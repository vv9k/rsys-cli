use super::{
    events::{Event, Events},
    get_terminal,
};
use anyhow::{anyhow, Result};
use rsys::linux::net::{iface, Interface};
use std::time::Instant;
use termion::event::Key;
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph},
};

const X_AXIS_TIME_MAX: f64 = 30.0;
const X_AXIS_GRAPH_MAX: f64 = X_AXIS_TIME_MAX - 2.;

struct IfaceMonitor {
    rx_speed: Vec<(f64, f64)>,
    tx_speed: Vec<(f64, f64)>,
    iface: Interface,
    prev_rx_bytes: u64,
    prev_tx_bytes: u64,
    prev_time: Instant,
    total_time: f64,
    current_max_y: f64,
    curr_rx_speed: f64,
    curr_tx_speed: f64,
    total_rx: f64,
    total_tx: f64,
    window: [f64; 2],
}

impl IfaceMonitor {
    fn new(name: &str) -> Result<IfaceMonitor> {
        let iface = iface(name)?.ok_or_else(|| anyhow!("Interface `{}` not found", name))?;
        let rx = iface.stat.rx_bytes;
        let tx = iface.stat.tx_bytes;
        Ok(IfaceMonitor {
            rx_speed: vec![(0., 0.)],
            tx_speed: vec![(0., 0.)],
            iface,
            prev_rx_bytes: rx,
            prev_tx_bytes: tx,
            prev_time: Instant::now(),
            current_max_y: 100.,
            curr_rx_speed: 0.,
            curr_tx_speed: 0.,
            total_time: 0.,
            total_rx: 0.,
            total_tx: 0.,
            window: [0., X_AXIS_TIME_MAX],
        })
    }

    fn update(&mut self) {
        self.iface.update().unwrap();
        let time = self.prev_time.elapsed().as_secs_f64();
        self.prev_time = Instant::now();
        let delta_rx = ((self.iface.stat.rx_bytes - self.prev_rx_bytes) as f64 / time) / 1024.;
        let delta_tx = ((self.iface.stat.tx_bytes - self.prev_tx_bytes) as f64 / time) / 1024.;
        self.total_rx += delta_rx;
        self.total_tx += delta_tx;
        self.rx_speed.push((time + self.total_time, delta_rx));
        self.tx_speed.push((time + self.total_time, delta_tx));
        self.curr_rx_speed = delta_rx;
        self.curr_tx_speed = delta_tx;
        if self.current_max_y < delta_rx {
            self.current_max_y = delta_rx + 100.;
        }
        if self.current_max_y < delta_tx {
            self.current_max_y = delta_tx + 100.;
        }
        self.prev_rx_bytes = self.iface.stat.rx_bytes;
        self.prev_tx_bytes = self.iface.stat.tx_bytes;
        self.total_time += time;
        if self.total_time > X_AXIS_GRAPH_MAX {
            let removed = self.rx_speed.remove(0);
            self.tx_speed.remove(0);
            self.window[0] += self.rx_speed[0].0 - removed.0;
            self.window[1] += self.rx_speed[0].0 - removed.0;
        }
    }

    fn y_bounds(&self) -> [f64; 2] {
        [0., self.current_max_y]
    }

    fn current_rx_speed(&self) -> String {
        format!("{:.2} kb/s", self.curr_rx_speed)
    }
    fn current_tx_speed(&self) -> String {
        format!("{:.2} kb/s", self.curr_tx_speed)
    }
    fn total_rx(&self) -> String {
        format!("{:.0} kb", self.total_rx)
    }
    fn total_tx(&self) -> String {
        format!("{:.0} kb", self.total_tx)
    }

    fn info_widget(&self) -> Paragraph {
        let rx_info = Spans::from(vec![
            Span::raw("Current rx speed: "),
            Span::styled(
                self.current_rx_speed(),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
            ),
            Span::raw(", Total received: "),
            Span::styled(
                self.total_rx(),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
            ),
        ]);
        let tx_info = Spans::from(vec![
            Span::raw("Current tx speed: "),
            Span::styled(
                self.current_tx_speed(),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue),
            ),
            Span::raw(", Total transferred: "),
            Span::styled(
                self.total_tx(),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue),
            ),
        ]);
        Paragraph::new(vec![
            Spans::from(vec![
                Span::styled(&self.iface.name, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(" {}", &self.iface.ipv4)),
            ]),
            rx_info,
            tx_info,
        ])
    }
    fn datasets(&self) -> Vec<Dataset> {
        vec![
            Dataset::default()
                .name("rx")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .data(&self.rx_speed),
            Dataset::default()
                .name("tx")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Blue))
                .data(&self.tx_speed),
        ]
    }
    fn graph_widget(&self) -> Chart {
        let datasets = self.datasets();
        let top_speed = self.current_max_y;
        Chart::new(datasets)
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
                    .bounds(self.window),
            )
            .y_axis(
                Axis::default()
                    .title("Speed")
                    .style(Style::default().fg(Color::Gray))
                    .labels(vec![
                        Span::raw("0"),
                        Span::raw(format!("{:.0} kb/s", top_speed * (1.0 / 5.0))),
                        Span::raw(format!("{:.0} kb/s", top_speed * (2.0 / 5.0))),
                        Span::raw(format!("{:.0} kb/s", top_speed * (3.0 / 5.0))),
                        Span::raw(format!("{:.0} kb/s", top_speed * (4.0 / 5.0))),
                        Span::styled(
                            format!("{:.2} kb/s", top_speed),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                    ])
                    .bounds(self.y_bounds()),
            )
    }
}

pub fn graph_net_interface(name: &str) -> Result<()> {
    let mut terminal = get_terminal()?;
    let events = Events::new();
    let mut monitor = IfaceMonitor::new(name)?;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                .split(size);
            let chart = monitor.graph_widget();

            f.render_widget(chart, chunks[0]);
            let info = monitor.info_widget();
            f.render_widget(info, chunks[1]);
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
