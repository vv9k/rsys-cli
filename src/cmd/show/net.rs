use super::{
    common::{
        kv_span, single_widget_loop, spans_from, DataSeries, GraphSettings, GraphWidget, Monitor, RxTx, Screen,
        StatefulWidget, Statistic,
    },
    events::Config,
};
use crate::util::{conv_fbs, conv_t, random_color};
use anyhow::{anyhow, Result};
use rsys::linux::net::{ifaces, Interface};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{Dataset, Paragraph},
    Frame,
};

const X_AXIS: (f64, f64) = (0., 30.0);
const Y_AXIS: (f64, f64) = (0., 100.0);
const TICK_RATE: u64 = 300;

pub struct IfaceSpeedStat {
    iface: Interface,
    data: RxTx<DataSeries>,
    prev_bytes: RxTx<u64>,
    curr_speed: RxTx<f64>,
    total: RxTx<f64>,
}
impl Statistic for IfaceSpeedStat {
    // Updates core and returns its new frequency
    fn update(&mut self, m: &mut Screen) -> Result<()> {
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
    fn pop(&mut self) -> f64 {
        let removed = self.data.rx_mut().pop();
        self.data.tx_mut().pop();

        if let Some(point) = self.data.rx().first() {
            return point.0 - removed.0;
        }
        0.
    }
    fn name(&self) -> &str {
        &self.iface.name
    }
}
impl IfaceSpeedStat {
    fn new(iface: Interface) -> Self {
        let rx = iface.stat.rx_bytes;
        let tx = iface.stat.tx_bytes;
        Self {
            iface,
            data: RxTx((
                DataSeries::new(random_color(Some(20))),
                DataSeries::new(random_color(Some(20))),
            )),
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
                self.data.rx().color,
                true,
            )]),
            spans_from(vec![kv_span(
                " Σrx : ",
                &self.total.rx_bytes_str(),
                self.data.rx().color,
                true,
            )]),
            spans_from(vec![kv_span(
                " Vtx : ",
                &self.curr_speed.tx_speed_str(),
                self.data.tx().color,
                true,
            )]),
            spans_from(vec![kv_span(
                " Σtx : ",
                &self.total.tx_bytes_str(),
                self.data.tx().color,
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
}
impl From<Interface> for IfaceSpeedStat {
    fn from(iface: Interface) -> IfaceSpeedStat {
        IfaceSpeedStat::new(iface)
    }
}

impl GraphWidget for Monitor<IfaceSpeedStat> {
    fn datasets(&self) -> Vec<Dataset> {
        let mut data = Vec::new();
        for iface in &self.stats {
            data.push(
                Dataset::default()
                    .name("rx")
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(iface.data.rx().color))
                    .data(&iface.data.rx().dataset()),
            );
            data.push(
                Dataset::default()
                    .name("tx")
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(iface.data.tx().color))
                    .data(&iface.data.tx().dataset()),
            );
        }
        data
    }
    fn settings(&self) -> GraphSettings {
        GraphSettings::new()
            .title(
                "Network Speed",
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue),
            )
            .x_title("Time", Style::default().fg(Color::White))
            .y_title("Speed", Style::default().fg(Color::White))
            .x_labels(self.m.x_bounds_labels(conv_t, 4))
            .y_labels(self.m.y_bounds_labels(conv_fbs, 5))
    }
    fn monitor(&self) -> &Screen {
        &self.m
    }
}

impl Monitor<IfaceSpeedStat> {
    pub fn new(filter: Option<&[&str]>) -> Result<Monitor<IfaceSpeedStat>> {
        Ok(Monitor {
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
                .map(IfaceSpeedStat::from)
                .collect::<Vec<IfaceSpeedStat>>(),
            m: Screen::new(X_AXIS, Y_AXIS),
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

    pub fn graph_loop(filter: Option<&[&str]>) -> Result<()> {
        let mut monitor = Self::new(filter)?;
        single_widget_loop(&mut monitor, Config::new(TICK_RATE))
    }

    pub fn single_iface_loop(name: &str) -> Result<()> {
        Self::graph_loop(Some(&[name]))
    }
}

impl StatefulWidget for Monitor<IfaceSpeedStat> {
    // Override
    fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Min(80)].as_ref())
            .split(area);

        self.render_info_widget(f, chunks[0]);
        self.render_graph_widget(f, chunks[1]);
    }
}
