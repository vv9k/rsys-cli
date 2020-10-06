use super::{
    common::{kv_span, single_widget_loop, spans_from, DataSeries, Monitor, RxTx, StatefulWidget},
    events::Config,
};
use crate::util::{conv_fb, random_color};
use anyhow::{anyhow, Result};
use rsys::linux::net::{ifaces, Interface};
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

struct IfaceStat {
    iface: Interface,
    rx_color: Color,
    tx_color: Color,
    data: RxTx<DataSeries>,
    prev_bytes: RxTx<u64>,
    curr_speed: RxTx<f64>,
    total: RxTx<f64>,
}
impl IfaceStat {
    fn new(iface: Interface) -> Self {
        let rx = iface.stat.rx_bytes;
        let tx = iface.stat.tx_bytes;
        Self {
            iface,
            rx_color: random_color(Some(20)),
            tx_color: random_color(Some(20)),
            data: RxTx((DataSeries::new(), DataSeries::new())),
            prev_bytes: RxTx((rx, tx)),
            curr_speed: RxTx::default(),
            total: RxTx::default(),
        }
    }
    fn delta(&mut self) -> (f64, f64) {
        (
            (self.iface.stat.rx_bytes - self.prev_bytes.rx()) as f64,
            (self.iface.stat.tx_bytes - self.prev_bytes.tx()) as f64,
        )
    }
    fn info(&self) -> Paragraph {
        Paragraph::new(vec![
            Spans::from(Span::styled(
                &self.iface.name,
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Green),
            )),
            spans_from(vec![kv_span(
                " Vrx : ",
                &self.curr_speed.rx_speed_str(),
                self.rx_color,
                true,
            )]),
            spans_from(vec![kv_span(
                " Σrx : ",
                &self.total.rx_bytes_str(),
                self.rx_color,
                true,
            )]),
            spans_from(vec![kv_span(
                " Vtx : ",
                &self.curr_speed.tx_speed_str(),
                self.tx_color,
                true,
            )]),
            spans_from(vec![kv_span(
                " Σtx : ",
                &self.total.tx_bytes_str(),
                self.tx_color,
                true,
            )]),
            spans_from(vec![kv_span(" ipv4: ", &self.iface.ipv4, Color::White, true)]),
            spans_from(vec![kv_span(" ipv6: ", &self.iface.ipv6, Color::White, true)]),
            spans_from(vec![kv_span(
                " mtu : ",
                &self.iface.mtu.to_string(),
                Color::White,
                true,
            )]),
            spans_from(vec![kv_span(" mac : ", &self.iface.mac_address, Color::White, true)]),
        ])
    }

    fn update(&mut self, m: &Monitor) -> Result<()> {
        self.iface
            .update()
            .map_err(|e| anyhow!("Failed to update interface `{}` - {}", self.iface.name, e.to_string()))?;

        let (delta_rx, delta_tx) = self.delta();

        self.total.inc(delta_rx, delta_tx);

        self.curr_speed = RxTx((delta_rx / m.elapsed_since_last(), delta_tx / m.elapsed_since_last()));
        self.data.rx_mut().add(m.elapsed_since_start(), *self.curr_speed.rx());
        self.data.tx_mut().add(m.elapsed_since_start(), *self.curr_speed.tx());

        self.prev_bytes = RxTx((self.iface.stat.rx_bytes, self.iface.stat.tx_bytes));
        Ok(())
    }
}
impl From<Interface> for IfaceStat {
    fn from(iface: Interface) -> IfaceStat {
        IfaceStat::new(iface)
    }
}

pub struct NetMonitor {
    stats: Vec<IfaceStat>,
    m: Monitor,
}

impl StatefulWidget for NetMonitor {
    fn update(&mut self) {
        for iface in &mut self.stats {
            iface.update(&mut self.m).unwrap();

            // If the values are bigger than current max y
            // update y axis
            self.m.set_if_y_max(iface.curr_speed.rx() + 100.);
            self.m.set_if_y_max(iface.curr_speed.tx() + 100.);
        }
        // If total time elapsed passed max x coordinate
        // pop first item of dataset and move x axis
        // by time difference of popped and last element
        if self.m.elapsed_since_start() > self.m.max_x() {
            let removed = self.stats[0].data.rx_mut().pop();
            self.stats[0].data.tx_mut().pop();
            if let Some(point) = self.stats[0].data.rx().first() {
                self.m.inc_x_axis(point.0 - removed.0);
            }
            self.stats.iter_mut().skip(1).for_each(|s| {
                s.data.rx_mut().pop();
                s.data.tx_mut().pop();
            });
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

impl NetMonitor {
    pub fn new(filter: Option<&[&str]>) -> Result<NetMonitor> {
        Ok(NetMonitor {
            stats: ifaces()?
                .0
                .into_iter()
                .filter(|s| {
                    if let Some(filters) = filter {
                        for f in filters {
                            if *f == &s.name {
                                return true;
                            } else {
                                continue;
                            }
                        }
                        false
                    } else {
                        true
                    }
                })
                .map(IfaceStat::from)
                .collect::<Vec<IfaceStat>>(),
            m: Monitor::new(X_AXIS, Y_AXIS),
        })
    }

    fn render_info_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let count = self.stats.len();
        let percentage = (100 / count) as u16;
        let constraints = (0..count)
            .into_iter()
            .map(|_| Constraint::Percentage(percentage))
            .collect::<Vec<Constraint>>();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);
        self.stats
            .iter()
            .enumerate()
            .for_each(|(i, s)| f.render_widget(s.info(), chunks[i]));
    }
    fn datasets(&self) -> Vec<Dataset> {
        let mut data = Vec::new();
        for iface in &self.stats {
            data.push(
                Dataset::default()
                    .name("rx")
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(iface.rx_color))
                    .data(&iface.data.rx().dataset()),
            );
            data.push(
                Dataset::default()
                    .name("tx")
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(iface.tx_color))
                    .data(&iface.data.tx().dataset()),
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

    pub fn graph_loop(filter: Option<&[&str]>) -> Result<()> {
        let mut monitor = NetMonitor::new(filter)?;
        single_widget_loop(&mut monitor, Config::new(TICK_RATE))
    }

    pub fn single_iface_loop(name: &str) -> Result<()> {
        Self::graph_loop(Some(&[name]))
    }
}
