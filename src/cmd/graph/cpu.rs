use super::{
    events::{Config, Event, Events},
    get_terminal, DataSeries,
};
use crate::util::{conv_fhz, conv_hz};
use anyhow::Result;
use rsys::linux::cpu::{processor, Core, Processor};
use std::time::Instant;
use termion::event::Key;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph},
    Frame,
};

const X_AXIS_TIME_MAX: f64 = 30.0;
const X_AXIS_GRAPH_MAX: f64 = X_AXIS_TIME_MAX - 1.6;

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

struct CpuMonitor {
    cpu: Processor,
    stats: Vec<CoreStat>,
    start_time: Instant,
    last_time: Instant,
    total_time: f64,
    current_max_y: f64,
    current_min_y: f64,
    window: [f64; 2],
}

impl CpuMonitor {
    fn new() -> Result<CpuMonitor> {
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
            total_time: 0.,
            current_max_y: 0.,
            current_min_y: f64::MAX,
            window: [0., X_AXIS_TIME_MAX],
        })
    }

    fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        self.total_time += self.last_time.elapsed().as_secs_f64();
        self.last_time = Instant::now();
        let cores = self.cpu.cores.clone();
        cores.iter().enumerate().for_each(|(i, c)| {
            self.stats[i].data.add(elapsed, c.cur_freq as f64);
            if c.cur_freq as f64 >= self.current_max_y {
                self.current_max_y = c.cur_freq as f64 + 100.
            }

            if c.cur_freq as f64 <= self.current_min_y {
                self.current_min_y = c.cur_freq as f64 - 100.
            }
        });

        for core in &mut self.cpu.cores {
            core.update().unwrap();
        }

        if self.total_time > X_AXIS_GRAPH_MAX {
            let removed = self.stats[0].data.pop();
            if let Some(point) = self.stats[0].data.nth(0) {
                self.window[0] += point.0 - removed.1;
                self.window[1] += point.0 - removed.1;
            }
            self.stats.iter_mut().skip(1).for_each(|c| {
                c.data.pop();
            });
        }
    }

    fn y_bounds(&self) -> [f64; 2] {
        [self.current_min_y, self.current_max_y]
    }
    fn datasets(&self) -> Vec<Dataset> {
        let mut data = Vec::new();
        for core in &self.stats {
            data.push(
                Dataset::default()
                    .name(&core.name)
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(core.color))
                    .data(&core.data.data),
            );
        }
        data
    }
    fn render_graph_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let datasets = self.datasets();
        let top_freq = self.current_max_y;
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
                    .bounds(self.window),
            )
            .y_axis(
                Axis::default()
                    .title("Core Frequency")
                    .style(Style::default().fg(Color::Gray))
                    .labels(vec![
                        Span::raw(conv_fhz(self.current_min_y)),
                        Span::raw(conv_fhz(
                            self.current_min_y + ((top_freq - self.current_min_y) * (1.0 / 4.0)),
                        )),
                        Span::raw(conv_fhz(
                            self.current_min_y + ((top_freq - self.current_min_y) * (2.0 / 4.0)),
                        )),
                        Span::raw(conv_fhz(
                            self.current_min_y + ((top_freq - self.current_min_y) * (3.0 / 4.0)),
                        )),
                        Span::styled(conv_fhz(top_freq), Style::default().add_modifier(Modifier::BOLD)),
                    ])
                    .bounds(self.y_bounds()),
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

    fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(85), Constraint::Percentage(15)].as_ref())
            .split(area);

        self.render_graph_widget(f, chunks[0]);
        self.render_cores_info_widget(f, chunks[1]);
    }
}

pub fn graph_cpu() -> Result<()> {
    let mut terminal = get_terminal()?;
    let cfg = Config::new(200);
    let events = Events::with_config(cfg);
    let mut monitor = CpuMonitor::new()?;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let layout = Layout::default().constraints([Constraint::Percentage(100)]).split(size);
            monitor.render_widget(f, layout[0]);
        })?;

        match events.next()? {
            Event::Input(input) => {
                if input == Key::Char('q') {
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
