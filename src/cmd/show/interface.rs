use super::{
    common::{kv_span, single_widget_loop, spans_from, DataSeries, Monitor, RxTx, StatefulWidget},
    events::Config,
};
use crate::util::conv_fb;
use anyhow::{anyhow, Result};
use rsys::linux::net::{self, iface, Interface};
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

pub struct IfaceMonitor {
    iface: Interface,
    m: Monitor,
    data: RxTx<DataSeries>,
    prev_bytes: RxTx<u64>,
    curr_speed: RxTx<f64>,
    total: RxTx<f64>,
}

impl StatefulWidget for IfaceMonitor {
    fn update(&mut self) {
        // Update interface
        self.iface.update().unwrap();

        let (delta_rx, delta_tx) = self.delta();

        self.total.inc(delta_rx, delta_tx);

        self.curr_speed = RxTx((
            delta_rx / self.m.elapsed_since_last(),
            delta_tx / self.m.elapsed_since_last(),
        ));
        self.data
            .rx_mut()
            .add(self.m.elapsed_since_start(), *self.curr_speed.rx());
        self.data
            .tx_mut()
            .add(self.m.elapsed_since_start(), *self.curr_speed.tx());

        // If the values are bigger than current max y
        // update y axis
        self.m.set_if_y_max(self.curr_speed.rx() + 100.);
        self.m.set_if_y_max(self.curr_speed.tx() + 100.);

        self.prev_bytes = RxTx((self.iface.stat.rx_bytes, self.iface.stat.tx_bytes));

        // If total time elapsed passed max x coordinate
        // pop first item of dataset and move x axis
        // by time difference of popped and last element
        if self.m.elapsed_since_start() > self.m.max_x() {
            let removed = self.data.rx_mut().pop();
            self.data.tx_mut().pop();
            if let Some(point) = self.data.rx().first() {
                self.m.inc_x_axis(point.0 - removed.0);
            }
        }
    }
    fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(area);

        self.render_info_widget(f, chunks[0]);
        self.render_graph_widget(f, chunks[1]);
    }
}

impl IfaceMonitor {
    pub fn new(name: &str) -> Result<IfaceMonitor> {
        let iface = iface(name)?.ok_or_else(|| anyhow!("Interface `{}` not found", name))?;
        let rx = iface.stat.rx_bytes;
        let tx = iface.stat.tx_bytes;
        Ok(IfaceMonitor {
            data: RxTx((DataSeries::new(), DataSeries::new())),
            iface,
            prev_bytes: RxTx((rx, tx)),
            curr_speed: RxTx::default(),
            total: RxTx::default(),
            m: Monitor::new(X_AXIS, Y_AXIS),
        })
    }

    pub fn default() -> Result<IfaceMonitor> {
        Self::new(&net::default_iface()?)
    }

    fn delta(&mut self) -> (f64, f64) {
        (
            (self.iface.stat.rx_bytes - self.prev_bytes.rx()) as f64,
            (self.iface.stat.tx_bytes - self.prev_bytes.tx()) as f64,
        )
    }

    fn iface_info(&self) -> Paragraph {
        Paragraph::new(vec![
            Spans::from(Span::styled(
                &self.iface.name,
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Green),
            )),
            spans_from(vec![kv_span(" ipv4: ", &self.iface.ipv4, Color::White, true)]),
            spans_from(vec![kv_span(" ipv6: ", &self.iface.ipv6, Color::White, true)]),
            spans_from(vec![kv_span(
                " mtu : ",
                &self.iface.mtu.to_string(),
                Color::White,
                true,
            )]),
            spans_from(vec![kv_span(" mac : ", &self.iface.mac_address, Color::White, true)]),
            spans_from(vec![kv_span(
                " Vrx : ",
                &self.curr_speed.rx_speed_str(),
                Color::Cyan,
                true,
            )]),
            spans_from(vec![kv_span(" Σrx : ", &self.total.rx_bytes_str(), Color::Cyan, true)]),
            spans_from(vec![kv_span(
                " Vtx : ",
                &self.curr_speed.tx_speed_str(),
                Color::Blue,
                true,
            )]),
            spans_from(vec![kv_span(" Σtx : ", &self.total.tx_bytes_str(), Color::Blue, true)]),
        ])
    }
    fn render_info_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(area);
        f.render_widget(self.iface_info(), chunks[0]);
    }
    fn datasets(&self) -> Vec<Dataset> {
        vec![
            Dataset::default()
                .name("rx")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .data(&self.data.rx().dataset()),
            Dataset::default()
                .name("tx")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Blue))
                .data(&self.data.rx().dataset()),
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

    pub fn graph_loop(name: &str) -> Result<()> {
        let mut monitor = IfaceMonitor::new(name)?;
        single_widget_loop(&mut monitor, Config::new(TICK_RATE))
    }
}
