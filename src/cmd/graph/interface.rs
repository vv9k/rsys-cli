use super::{
    common::{DataSeries, Monitor},
    events::{Config, Event},
    get_terminal,
};
use crate::util::conv_fb;
use anyhow::{anyhow, Result};
use rsys::linux::net::{iface, Interface};
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

const X_AXIS: (f64, f64) = (0., 30.0);
const Y_AXIS: (f64, f64) = (0., 100.0);

pub(crate) struct IfaceMonitor {
    rx_data: DataSeries,
    tx_data: DataSeries,
    iface: Interface,
    prev_rx_bytes: u64,
    prev_tx_bytes: u64,
    prev_time: Instant,
    curr_rx_speed: f64,
    curr_tx_speed: f64,
    total_rx: f64,
    total_tx: f64,
    monitor: Monitor,
}

impl IfaceMonitor {
    pub(crate) fn new(name: &str) -> Result<IfaceMonitor> {
        let iface = iface(name)?.ok_or_else(|| anyhow!("Interface `{}` not found", name))?;
        let rx = iface.stat.rx_bytes;
        let tx = iface.stat.tx_bytes;
        Ok(IfaceMonitor {
            rx_data: DataSeries::new(),
            tx_data: DataSeries::new(),
            iface,
            prev_rx_bytes: rx,
            prev_tx_bytes: tx,
            prev_time: Instant::now(),
            curr_rx_speed: 0.,
            curr_tx_speed: 0.,
            total_rx: 0.,
            total_tx: 0.,
            monitor: Monitor::new(X_AXIS, Y_AXIS, Config::new(300)),
        })
    }

    fn delta(&mut self, time: f64) -> (f64, f64) {
        (
            (self.iface.stat.rx_bytes - self.prev_rx_bytes) as f64 / time,
            (self.iface.stat.tx_bytes - self.prev_tx_bytes) as f64 / time,
        )
    }

    fn update(&mut self) {
        // Update interface
        self.iface.update().unwrap();

        // time between previous run and now
        let time = self.prev_time.elapsed().as_secs_f64();

        let (delta_rx, delta_tx) = self.delta(time);

        self.prev_time = Instant::now();
        self.monitor.add_time(time);

        self.total_rx += delta_rx;
        self.total_tx += delta_tx;
        self.rx_data.add(self.monitor.time(), delta_rx);
        self.tx_data.add(self.monitor.time(), delta_tx);
        self.curr_rx_speed = delta_rx;
        self.curr_tx_speed = delta_tx;

        // If the values are bigger than current max y
        // update y axis
        self.monitor.set_if_y_max(delta_rx + 100.);
        self.monitor.set_if_y_max(delta_tx + 100.);

        self.prev_rx_bytes = self.iface.stat.rx_bytes;
        self.prev_tx_bytes = self.iface.stat.tx_bytes;

        // If total time elapsed passed max x coordinate
        // pop first item of dataset and move x axis
        // by time difference of popped and last element
        if self.monitor.time() > self.monitor.max_x() {
            let removed = self.rx_data.pop();
            self.tx_data.pop();
            if let Some(point) = self.rx_data.first() {
                self.monitor.inc_x_axis(point.0 - removed.0);
            }
        }
    }

    fn current_rx_speed(&self) -> String {
        format!("{}/s", conv_fb(self.curr_rx_speed))
    }
    fn current_tx_speed(&self) -> String {
        format!("{}/s", conv_fb(self.curr_tx_speed))
    }
    fn total_rx(&self) -> String {
        conv_fb(self.total_rx)
    }
    fn total_tx(&self) -> String {
        conv_fb(self.total_tx)
    }
    fn rx_info(&self) -> Spans {
        Spans::from(vec![
            Span::raw("  Current rx speed: "),
            Span::styled(
                self.current_rx_speed(),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
            ),
            Span::raw(", Total received: "),
            Span::styled(
                self.total_rx(),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
            ),
        ])
    }
    fn tx_info(&self) -> Spans {
        Spans::from(vec![
            Span::raw("  Current tx speed: "),
            Span::styled(
                self.current_tx_speed(),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue),
            ),
            Span::raw(", Total transferred: "),
            Span::styled(
                self.total_tx(),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue),
            ),
        ])
    }
    fn iface_stats(&self) -> Paragraph {
        Paragraph::new(vec![Spans::from(vec![]), self.rx_info(), self.tx_info()])
    }
    fn iface_info(&self) -> Paragraph {
        Paragraph::new(vec![
            Spans::from(Span::styled(
                &self.iface.name,
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Green),
            )),
            Spans::from(vec![
                Span::raw("  ipv4: "),
                Span::styled(&self.iface.ipv4, Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Spans::from(vec![
                Span::raw("  ipv6: "),
                Span::styled(&self.iface.ipv6, Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Spans::from(vec![
                Span::raw("  mtu:  "),
                Span::styled(
                    self.iface.mtu.to_string(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]),
            Spans::from(vec![
                Span::raw("  mac:  "),
                Span::styled(&self.iface.mac_address, Style::default().add_modifier(Modifier::BOLD)),
            ]),
        ])
    }
    fn render_info_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        f.render_widget(self.iface_stats(), chunks[1]);
        f.render_widget(self.iface_info(), chunks[0]);
    }
    fn datasets(&self) -> Vec<Dataset> {
        vec![
            Dataset::default()
                .name("rx")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .data(&self.rx_data.data()),
            Dataset::default()
                .name("tx")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Blue))
                .data(&self.tx_data.data()),
        ]
    }
    fn render_graph_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let datasets = self.datasets();
        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(Span::styled(
                        "Network speed",
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    .bounds(self.monitor.x()),
            )
            .y_axis(
                Axis::default()
                    .title("Speed")
                    .style(Style::default().fg(Color::Gray))
                    .labels(vec![
                        Span::raw("0"),
                        Span::raw(format!("{}/s", conv_fb(self.monitor.max_y() * (1.0 / 5.0)))),
                        Span::raw(format!("{}/s", conv_fb(self.monitor.max_y() * (2.0 / 5.0)))),
                        Span::raw(format!("{}/s", conv_fb(self.monitor.max_y() * (3.0 / 5.0)))),
                        Span::raw(format!("{}/s", conv_fb(self.monitor.max_y() * (4.0 / 5.0)))),
                        Span::styled(
                            format!("{}/s", conv_fb(self.monitor.max_y())),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                    ])
                    .bounds(self.monitor.y()),
            );
        f.render_widget(chart, area);
    }

    pub(crate) fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(85), Constraint::Percentage(15)].as_ref())
            .split(area);

        self.render_graph_widget(f, chunks[0]);
        self.render_info_widget(f, chunks[1]);
    }

    pub(crate) fn graph_loop(name: &str) -> Result<()> {
        let mut terminal = get_terminal()?;
        let mut monitor = IfaceMonitor::new(name)?;
        loop {
            terminal.draw(|f| {
                let size = f.size();
                let layout = Layout::default().constraints([Constraint::Percentage(100)]).split(size);
                monitor.render_widget(f, layout[0]);
            })?;

            match monitor.monitor.next_event()? {
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
}
