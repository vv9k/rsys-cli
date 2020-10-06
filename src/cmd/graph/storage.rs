use super::{
    common::{graph_loop, DataSeries, GraphWidget, Monitor, RxTx},
    events::Config,
};
use crate::util::{conv_fb, random_color};
use anyhow::{anyhow, Result};
use rsys::linux::storage::{storage_devices_info, BlockStorageInfo};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, Row, Table},
    Frame,
};

const X_AXIS: (f64, f64) = (0., 30.0);
const Y_AXIS: (f64, f64) = (0., 100.);
const TICK_RATE: u64 = 200;
const SECTOR_SIZE: f64 = 512.;

#[derive(Debug)]
// Stats of a single block storage device
struct BlockDeviceStat {
    name: String,
    color: Color,
    device: BlockStorageInfo,
    data: RxTx<DataSeries>,
    speed: RxTx<f64>,
    total: RxTx<f64>,
}
impl BlockDeviceStat {
    fn from_storage_info(info: BlockStorageInfo) -> BlockDeviceStat {
        Self {
            name: info.dev.to_string(),
            color: random_color(Some(50)),
            device: info,
            data: RxTx((DataSeries::new(), DataSeries::new())),
            speed: RxTx::default(),
            total: RxTx::default(),
        }
    }
    fn sectors(&mut self) -> (f64, f64) {
        if let Some(stat) = &self.device.stat {
            (
                stat.read_sectors as f64 * SECTOR_SIZE,
                stat.write_sectors as f64 * SECTOR_SIZE,
            )
        } else {
            (0., 0.)
        }
    }
    // Updates core and returns its new frequency
    fn update(&mut self, time_delta: f64) -> Result<(f64, f64)> {
        let (rx_before, wx_before) = self.sectors();

        self.device
            .update_stats()
            .map_err(|e| anyhow!("Failed to update block device `{}` stats - {}", self.name, e))?;

        let (rx_after, wx_after) = self.sectors();

        let rx_delta = rx_after - rx_before;
        let wx_delta = wx_after - wx_before;

        self.total.inc(rx_delta, wx_delta);
        self.speed = RxTx((rx_delta / time_delta, wx_delta / time_delta));

        Ok(self.speed.0)
    }

    fn add_current(&mut self, time: f64) {
        self.data.rx_mut().add(time, *self.speed.rx());
        self.data.tx_mut().add(time, *self.speed.tx());
    }
}

pub(crate) struct StorageMonitor {
    stats: Vec<BlockDeviceStat>,
    m: Monitor,
}

impl GraphWidget for StorageMonitor {
    fn update(&mut self) {
        for storage in &mut self.stats {
            let (rx, wx) = storage.update(self.m.elapsed_since_last()).unwrap();
            storage.add_current(self.m.elapsed_since_start());
            self.m.set_if_y_max(rx + 100.);
            self.m.set_if_y_max(wx + 100.);
        }
        self.m.update_last_time();

        // Move x axis if time reached end
        if self.m.elapsed_since_start() > self.m.max_x() {
            let removed = self.stats[0].data.rx_mut().pop();
            self.stats[0].data.tx_mut().pop();
            if let Some(point) = self.stats[0].data.rx().first() {
                self.m.inc_x_axis(point.0 - removed.0);
            }
            self.stats.iter_mut().skip(1).for_each(|c| {
                c.data.rx_mut().pop();
                c.data.tx_mut().pop();
            });
        }
    }
    fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(area);

        self.render_storage_info_widget(f, chunks[0]);
        self.render_graph_widget(f, chunks[1]);
    }
    fn monitor(&mut self) -> &mut Monitor {
        &mut self.m
    }
}

impl StorageMonitor {
    pub(crate) fn new() -> Result<StorageMonitor> {
        let infos = storage_devices_info().map_err(|e| anyhow!("Failed to get storage devices info - {}", e))?;
        let mut stats = Vec::new();
        for info in infos.into_iter() {
            stats.push(BlockDeviceStat::from_storage_info(info));
        }

        stats.sort_by(|s1, s2| s1.name.cmp(&s2.name));

        Ok(StorageMonitor {
            stats,
            m: Monitor::new(X_AXIS, Y_AXIS),
        })
    }

    fn datasets(&self) -> Vec<Dataset> {
        let mut data = Vec::new();
        for device in &self.stats {
            data.push(
                Dataset::default()
                    .name(format!("rx {}", &device.name))
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(device.color))
                    .data(&device.data.rx().data()),
            );
            data.push(
                Dataset::default()
                    .name(format!("wx {}", &device.name))
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(device.color))
                    .data(&device.data.tx().data()),
            );
        }
        data
    }

    fn render_graph_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let datasets = self.datasets();
        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(Span::styled(
                        "Storage devices",
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    .bounds(self.m.x()),
            )
            .y_axis(
                Axis::default()
                    .title("r/w speed")
                    .style(Style::default().fg(Color::Gray))
                    .labels(vec![
                        Span::raw("0"),
                        Span::raw(format!("{}/s", conv_fb(self.m.max_y() * (1.0 / 5.0)))),
                        Span::raw(format!("{}/s", conv_fb(self.m.max_y() * (2.0 / 5.0)))),
                        Span::raw(format!("{}/s", conv_fb(self.m.max_y() * (3.0 / 5.0)))),
                        Span::raw(format!("{}/s", conv_fb(self.m.max_y() * (4.0 / 5.0)))),
                        Span::styled(
                            format!("{}/s", conv_fb(self.m.max_y())),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                    ])
                    .bounds(self.m.y()),
            );
        f.render_widget(chart, area);
    }

    fn render_storage_info_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(area);

        let headers = ["name", "rx/s", "wx/s", "Σrx", "Σwx"];
        let data = self.stats.iter().map(|s| {
            Row::StyledData(
                vec![
                    s.name.to_string(),
                    s.speed.rx_speed_str(),
                    s.speed.tx_speed_str(),
                    s.total.rx_bytes_str(),
                    s.total.tx_bytes_str(),
                ]
                .into_iter(),
                Style::default().fg(s.color),
            )
        });

        let table = Table::new(headers.iter(), data).widths(&[
            Constraint::Percentage(10),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ]);

        f.render_widget(table, chunks[0]);
    }

    pub(crate) fn graph_loop() -> Result<()> {
        let mut monitor = StorageMonitor::new()?;
        graph_loop(&mut monitor, Config::new(TICK_RATE))
    }
}
