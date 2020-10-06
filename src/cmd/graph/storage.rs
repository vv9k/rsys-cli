use super::{
    common::{graph_loop, DataSeries, GraphWidget, Monitor},
    events::Config,
};
use crate::util::{conv_fb, random_color};
use anyhow::{anyhow, Result};
use rsys::linux::storage::{storage_devices_info, BlockStorageInfo};
use std::time::Instant;
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
const TICK_RATE: u64 = 1000;
const SECTOR_SIZE: f64 = 512.;

#[derive(Debug)]
// Stats of a single block storage device
struct BlockDeviceStat {
    name: String,
    color: Color,
    rx_data: DataSeries,
    wx_data: DataSeries,
    device: BlockStorageInfo,
    rx_speed: f64,
    wx_speed: f64,
    total_rx: f64,
    total_wx: f64,
}
impl BlockDeviceStat {
    fn from_storage_info(info: BlockStorageInfo) -> BlockDeviceStat {
        Self {
            name: info.dev.to_string(),
            color: random_color(Some(50)),
            rx_data: DataSeries::new(),
            wx_data: DataSeries::new(),
            device: info,
            rx_speed: 0.,
            wx_speed: 0.,
            total_rx: 0.,
            total_wx: 0.,
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

        self.total_rx += rx_delta;
        self.total_wx += wx_delta;

        self.rx_speed = rx_delta / time_delta;
        self.wx_speed = wx_delta / time_delta;
        Ok((self.rx_speed, self.wx_speed))
    }

    fn add_current(&mut self, time: f64) {
        self.rx_data.add(time, self.rx_speed);
        self.wx_data.add(time, self.wx_speed);
    }
}

pub(crate) struct StorageMonitor {
    stats: Vec<BlockDeviceStat>,
    start_time: Instant,
    last_time: Instant,
    m: Monitor,
}

impl GraphWidget for StorageMonitor {
    fn update(&mut self) {
        // Time since last run
        let elapsed_last = self.last_time.elapsed().as_secs_f64();
        let elapsed_start = self.start_time.elapsed().as_secs_f64();
        self.m.add_time(elapsed_last);

        for storage in &mut self.stats {
            let (rx, wx) = storage.update(elapsed_last).unwrap();
            storage.add_current(elapsed_start);
            self.m.set_if_y_max(rx + 100.);
            self.m.set_if_y_max(wx + 100.);
        }

        // Set last_time to current time
        self.last_time = Instant::now();

        // Move x axis if time reached end
        if self.m.time() > self.m.max_x() {
            let removed = self.stats[0].rx_data.pop();
            self.stats[0].wx_data.pop();
            if let Some(point) = self.stats[0].rx_data.first() {
                self.m.inc_x_axis(point.0 - removed.0);
            }
            self.stats.iter_mut().skip(1).for_each(|c| {
                c.rx_data.pop();
                c.wx_data.pop();
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
            start_time: Instant::now(),
            last_time: Instant::now(),
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
                    .data(&device.rx_data.data()),
            );
            data.push(
                Dataset::default()
                    .name(format!("wx {}", &device.name))
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(device.color))
                    .data(&device.wx_data.data()),
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
        let data = self.stats.iter().enumerate().map(|(i, s)| {
            Row::StyledData(
                vec![
                    s.name.to_string(),
                    format!("{}/s", conv_fb(s.rx_speed)),
                    format!("{}/s", conv_fb(s.wx_speed)),
                    conv_fb(s.total_rx),
                    conv_fb(s.total_wx),
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
