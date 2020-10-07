use super::{
    common::{single_widget_loop, DataSeries, GraphWidget, Monitor, RxTx, StatefulWidget},
    events::Config,
};
use crate::util::{conv_fbs, conv_t, random_color};
use anyhow::{anyhow, Result};
use rsys::linux::storage::{storage_devices_info, BlockStorageInfo};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Dataset, Row, Table},
    Frame,
};

const X_AXIS: (f64, f64) = (0., 30.0);
const Y_AXIS: (f64, f64) = (0., 100.);
const TICK_RATE: u64 = 200;
const SECTOR_SIZE: f64 = 512.;
const STORAGE_INFO_HEADERS: &[&str] = &["name", "rx/s", "wx/s", "Σrx", "Σwx"];

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
impl From<BlockStorageInfo> for BlockDeviceStat {
    fn from(info: BlockStorageInfo) -> Self {
        Self {
            name: info.dev.to_string(),
            color: random_color(Some(20)),
            device: info,
            data: RxTx((DataSeries::new(), DataSeries::new())),
            speed: RxTx::default(),
            total: RxTx::default(),
        }
    }
}
impl BlockDeviceStat {
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
    fn update(&mut self, monitor: &Monitor) -> Result<()> {
        let (rx_before, wx_before) = self.sectors();
        let time_delta = monitor.elapsed_since_last();

        self.device
            .update_stats()
            .map_err(|e| anyhow!("Failed to update block device `{}` stats - {}", self.name, e))?;

        let (rx_after, wx_after) = self.sectors();

        let rx_delta = rx_after - rx_before;
        let wx_delta = wx_after - wx_before;

        self.total.inc(rx_delta, wx_delta);
        self.speed = RxTx((rx_delta / time_delta, wx_delta / time_delta));

        self.add_current(monitor.elapsed_since_start());

        Ok(())
    }

    fn add_current(&mut self, time: f64) {
        self.data.rx_mut().add(time, *self.speed.rx());
        self.data.tx_mut().add(time, *self.speed.tx());
    }
}

pub struct StorageMonitor {
    stats: Vec<BlockDeviceStat>,
    m: Monitor,
}

impl StatefulWidget for StorageMonitor {
    fn update(&mut self) {
        for storage in &mut self.stats {
            storage.update(&mut self.m).unwrap();

            self.m.set_if_y_max(storage.speed.rx() + 100.);
            self.m.set_if_y_max(storage.speed.tx() + 100.);
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
}

impl GraphWidget for StorageMonitor {
    fn datasets(&self) -> Vec<Dataset> {
        let mut data = Vec::new();
        for device in &self.stats {
            data.push(
                Dataset::default()
                    .name(format!("rx {}", &device.name))
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(device.color))
                    .data(&device.data.rx().dataset()),
            );
            data.push(
                Dataset::default()
                    .name(format!("wx {}", &device.name))
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(device.color))
                    .data(&device.data.tx().dataset()),
            );
        }
        data
    }
    fn title(&self) -> Span {
        Span::styled(
            "Storage devices",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue),
        )
    }
    fn x_axis(&self) -> Span {
        Span::styled("Time", Style::default())
    }
    fn y_axis(&self) -> Span {
        Span::styled("r/w speed", Style::default())
    }
    fn y_labels(&self) -> Vec<Span> {
        self.m.y_bounds_labels(conv_fbs, 5)
    }
    fn x_labels(&self) -> Vec<Span> {
        self.m.x_bounds_labels(conv_t, 4)
    }
    fn monitor(&self) -> &Monitor {
        &self.m
    }
}

impl StorageMonitor {
    pub fn new() -> Result<StorageMonitor> {
        Ok(StorageMonitor {
            stats: {
                let mut stats = storage_devices_info()
                    .map_err(|e| anyhow!("Failed to get storage devices info - {}", e))?
                    .into_iter()
                    .map(BlockDeviceStat::from)
                    .collect::<Vec<BlockDeviceStat>>();
                stats.sort_by(|s1, s2| s1.name.cmp(&s2.name));
                stats
            },
            m: Monitor::new(X_AXIS, Y_AXIS),
        })
    }
    fn render_storage_info_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
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

        let table = Table::new(STORAGE_INFO_HEADERS.iter(), data)
            .widths(&[
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(16),
                Constraint::Percentage(16),
            ])
            .header_gap(1)
            .column_spacing(1);

        f.render_widget(table, area);
    }

    pub fn graph_loop() -> Result<()> {
        let mut monitor = StorageMonitor::new()?;
        single_widget_loop(&mut monitor, Config::new(TICK_RATE))
    }
}
