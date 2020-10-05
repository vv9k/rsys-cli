use super::{
    common::{DataSeries, Monitor},
    events::{Config, Event},
    get_terminal,
};
use crate::util::{conv_fhz, conv_hz};
use anyhow::Result;
use rsys::linux::cpu::{processor, Core, Processor};
use std::time::Instant;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph},
    Frame,
};

const X_AXIS: (f64, f64) = (0., 30.0);
const Y_AXIS: (f64, f64) = (f64::MAX, 0.);
const TICK_RATE: u64 = 250;

struct CoreStat {
    name: String,
    color: Color,
    data: DataSeries,
}
impl CoreStat {
    fn from_core(core: &Core) -> CoreStat {
        Self {
            name: format!("cpu{}", core.id),
            color: Color::Indexed(core.id as u8),
            data: DataSeries::new(),
        }
    }
}

pub(crate) struct CpuMonitor {
    cpu: Processor,
    stats: Vec<CoreStat>,
    start_time: Instant,
    last_time: Instant,
    m: Monitor,
}

impl CpuMonitor {
    pub(crate) fn new(tick_rate: Option<u64>) -> Result<CpuMonitor> {
        let cpu = processor()?;
        let mut stats = Vec::new();
        for core in &cpu.cores {
            stats.push(CoreStat::from_core(&core));
        }

        Ok(CpuMonitor {
            cpu,
            stats,
            start_time: Instant::now(),
            last_time: Instant::now(),
            m: Monitor::new(X_AXIS, Y_AXIS, Config::new_or_default(tick_rate)),
        })
    }

    fn update(&mut self) {
        // Time since begining
        let elapsed = self.start_time.elapsed().as_secs_f64();

        // Time since last run
        self.m.add_time(self.last_time.elapsed().as_secs_f64());

        // Update frequencies on cores
        for core in &mut self.cpu.cores {
            core.update().unwrap();
        }

        // Set last_time to current time
        self.last_time = Instant::now();

        // Parse data for each core
        let cores = self.cpu.cores.clone();
        cores.iter().enumerate().for_each(|(i, c)| {
            let freq = c.cur_freq as f64;

            self.stats[i].data.add(elapsed, freq);
            self.m.set_if_y_max(freq + 100_000.);
            self.m.set_if_y_min(freq + 100_000.);
        });

        // Move x axis if time reached end
        if self.m.time() > self.m.max_x() {
            let removed = self.stats[0].data.pop();
            if let Some(point) = self.stats[0].data.first() {
                self.m.inc_x_axis(point.0 - removed.0);
            }
            self.stats.iter_mut().skip(1).for_each(|c| {
                c.data.pop();
            });
        }
    }

    fn datasets(&self) -> Vec<Dataset> {
        let mut data = Vec::new();
        for core in &self.stats {
            data.push(
                Dataset::default()
                    .name(&core.name)
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(core.color))
                    .data(&core.data.data()),
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
                        "CPU Frequencies",
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
                    .title("Core Frequency")
                    .style(Style::default().fg(Color::Gray))
                    .labels(vec![
                        Span::raw(conv_fhz(self.m.min_y())),
                        Span::raw(conv_fhz(
                            self.m.min_y() + ((self.m.max_y() - self.m.min_y()) * (1.0 / 4.0)),
                        )),
                        Span::raw(conv_fhz(
                            self.m.min_y() + ((self.m.max_y() - self.m.min_y()) * (2.0 / 4.0)),
                        )),
                        Span::raw(conv_fhz(
                            self.m.min_y() + ((self.m.max_y() - self.m.min_y()) * (3.0 / 4.0)),
                        )),
                        Span::styled(conv_fhz(self.m.max_y()), Style::default().add_modifier(Modifier::BOLD)),
                    ])
                    .bounds(self.m.y()),
            );
        f.render_widget(chart, area);
    }

    fn render_cores_info_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let count = self.cpu.core_count();

        let mut first = Vec::new();
        let mut second = Vec::new();

        for i in 0..(count / 2) {
            let core = &self.cpu.cores[i];
            first.push(Spans::from(vec![
                Span::raw(format!("cpu{}: ", core.id)),
                Span::styled(
                    conv_hz(core.cur_freq),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Indexed(i as u8)),
                ),
            ]));
        }
        for i in (count / 2)..count {
            let core = &self.cpu.cores[i];
            second.push(Spans::from(vec![
                Span::raw(format!("cpu{}: ", core.id)),
                Span::styled(
                    conv_hz(core.cur_freq),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Indexed(i as u8)),
                ),
            ]));
        }

        let first_col = Paragraph::new(first);
        let second_col = Paragraph::new(second);

        f.render_widget(first_col, chunks[0]);
        f.render_widget(second_col, chunks[1]);
    }

    pub(crate) fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(85), Constraint::Percentage(15)].as_ref())
            .split(area);

        self.render_graph_widget(f, chunks[0]);
        self.render_cores_info_widget(f, chunks[1]);
    }

    pub(crate) fn graph_loop() -> Result<()> {
        let mut terminal = get_terminal()?;
        let mut monitor = CpuMonitor::new(Some(TICK_RATE))?;

        loop {
            terminal.draw(|f| {
                let size = f.size();
                let layout = Layout::default().constraints([Constraint::Percentage(100)]).split(size);
                monitor.render_widget(f, layout[0]);
            })?;

            match monitor.m.next_event()? {
                Event::Input(input) => {
                    if input == monitor.m.exit_key() {
                        break;
                    }
                }
                Event::Tick => {
                    monitor.update();
                }
            }
        }

        Ok(())
    }
}
