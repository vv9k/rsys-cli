use super::{
    common::{graph_loop, kv_span, spans_from, DataSeries, GraphWidget, Monitor},
    events::Config,
};
use crate::util::conv_fb;
use anyhow::{anyhow, Result};
use rsys::linux::net::{iface, Interface};
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
const Y_AXIS: (f64, f64) = (0., 100.0);
const TICK_RATE: u64 = 300;

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
    m: Monitor,
}

impl GraphWidget for IfaceMonitor {
    fn update(&mut self) {
        // Update interface
        self.iface.update().unwrap();

        // time between previous run and now
        let time = self.prev_time.elapsed().as_secs_f64();

        let (delta_rx, delta_tx) = self.delta(time);

        self.prev_time = Instant::now();
        self.m.add_time(time);

        self.total_rx += delta_rx;
        self.total_tx += delta_tx;
        self.rx_data.add(self.m.time(), delta_rx);
        self.tx_data.add(self.m.time(), delta_tx);
        self.curr_rx_speed = delta_rx;
        self.curr_tx_speed = delta_tx;

        // If the values are bigger than current max y
        // update y axis
        self.m.set_if_y_max(delta_rx + 100.);
        self.m.set_if_y_max(delta_tx + 100.);

        self.prev_rx_bytes = self.iface.stat.rx_bytes;
        self.prev_tx_bytes = self.iface.stat.tx_bytes;

        // If total time elapsed passed max x coordinate
        // pop first item of dataset and move x axis
        // by time difference of popped and last element
        if self.m.time() > self.m.max_x() {
            let removed = self.rx_data.pop();
            self.tx_data.pop();
            if let Some(point) = self.rx_data.first() {
                self.m.inc_x_axis(point.0 - removed.0);
            }
        }
    }
    fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(85), Constraint::Percentage(15)].as_ref())
            .split(area);

        self.render_graph_widget(f, chunks[0]);
        self.render_info_widget(f, chunks[1]);
    }
    fn monitor(&mut self) -> &mut Monitor {
        &mut self.m
    }
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
            m: Monitor::new(X_AXIS, Y_AXIS),
        })
    }

    fn delta(&mut self, time: f64) -> (f64, f64) {
        (
            (self.iface.stat.rx_bytes - self.prev_rx_bytes) as f64 / time,
            (self.iface.stat.tx_bytes - self.prev_tx_bytes) as f64 / time,
        )
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
        spans_from(vec![
            kv_span("  Current rx speed: ", &self.current_rx_speed(), Color::Cyan, true),
            kv_span(", Total received: ", &self.total_rx(), Color::Cyan, true),
        ])
    }
    fn tx_info(&self) -> Spans {
        spans_from(vec![
            kv_span("  Current tx speed: ", &self.current_tx_speed(), Color::Blue, true),
            kv_span(", Total received: ", &self.total_tx(), Color::Blue, true),
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
            spans_from(vec![kv_span("  ipv4: ", &self.iface.ipv4, Color::White, true)]),
            spans_from(vec![kv_span("  ipv6: ", &self.iface.ipv6, Color::White, true)]),
            spans_from(vec![kv_span(
                "  mtu: ",
                &self.iface.mtu.to_string(),
                Color::White,
                true,
            )]),
            spans_from(vec![kv_span("  mac: ", &self.iface.mac_address, Color::White, true)]),
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
                    .bounds(self.m.x()),
            )
            .y_axis(
                Axis::default()
                    .title("Speed")
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

    pub(crate) fn graph_loop(name: &str) -> Result<()> {
        let mut monitor = IfaceMonitor::new(name)?;
        graph_loop(&mut monitor, Config::new(TICK_RATE))
    }
}
