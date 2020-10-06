use super::{
    cpu::CpuMonitor,
    events::{Config, Event, Events},
    get_terminal,
    interface::IfaceMonitor,
    storage::StorageMonitor,
};
use crate::util::conv_fb;
use anyhow::Result;
use rsys::linux;
use std::{
    ops::AddAssign,
    fmt::Debug,
    time::Instant
};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    Frame,
};

type KvSpan<'a> = [Span<'a>; 2];

pub(crate) fn kv_span<'a, T: Into<String>>(k: T, v: T, color: Color, bold: bool) -> KvSpan<'a> {
    let val = if bold {
        Span::styled(v.into(), Style::default().fg(color).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(v.into(), Style::default().fg(color))
    };
    [Span::raw(k.into()), val]
}

pub(crate) fn spans_from<'a>(kvspans: Vec<KvSpan<'a>>) -> Spans<'a> {
    Spans::from(kvspans.concat())
}

/// Trait grouping all graph widgets together providing functionality
/// like graph_loop.
pub(crate) trait GraphWidget {
    fn update(&mut self);
    fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect);
    fn monitor(&mut self) -> &mut Monitor;
}
pub(crate) fn graph_loop<W: GraphWidget>(widget: &mut W, config: Config) -> Result<()> {
    let mut terminal = get_terminal()?;
    let events = Events::with_config(config);
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let layout = Layout::default().constraints([Constraint::Percentage(100)]).split(size);
            widget.render_widget(f, layout[0]);
        })?;

        match events.next()? {
            Event::Input(input) => {
                if input == events.exit_key() {
                    break;
                }
            }
            Event::Tick => {
                widget.update();
            }
        }
    }
    Ok(())
}

pub(crate) fn graph_all_loop() -> Result<()> {
    let mut terminal = get_terminal()?;
    let events = Events::with_config(Config::new(200));
    let mut cpumon = CpuMonitor::new()?;
    let mut ifacemon = IfaceMonitor::new(&linux::net::default_iface()?)?;
    let mut stormon = StorageMonitor::new()?;
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let layout = Layout::default()
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ])
                .split(size);
            cpumon.render_widget(f, layout[0]);
            ifacemon.render_widget(f, layout[1]);
            stormon.render_widget(f, layout[2]);
        })?;

        match events.next()? {
            Event::Input(input) => {
                if input == events.exit_key() {
                    break;
                }
            }
            Event::Tick => {
                cpumon.update();
                ifacemon.update();
                stormon.update();
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
/// Wrapper stuct for graph datapoints used by Datasets.
pub(crate) struct DataSeries {
    data: Vec<(f64, f64)>,
    len: usize,
}
impl DataSeries {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            len: 0,
        }
    }
    pub fn data(&self) -> &[(f64, f64)] {
        &self.data
    }

    /// Add a data point
    pub fn add(&mut self, time: f64, value: f64) {
        self.data.push((time, value));
        self.len += 1;
    }

    /// Pop first point returning it. If data vector is empty
    /// return (0., 0.)
    pub fn pop(&mut self) -> (f64, f64) {
        if self.len > 0 {
            self.len -= 1;
            return self.data.remove(0);
        }
        (0., 0.)
    }

    /// Return nth element of data set if such exists.
    pub fn nth(&self, n: usize) -> Option<(f64, f64)> {
        if n < self.len {
            return Some(self.data[n]);
        }
        None
    }

    /// Return first element of data set if such exists.
    pub fn first(&self) -> Option<(f64, f64)> {
        self.nth(0)
    }
}

#[derive(Debug)]
pub(crate) struct Monitor {
    x_axis: [f64; 2],
    y_axis: [f64; 2],
    start_time: Instant,
    last_time: Instant,
}
impl Monitor {
    pub fn new(x: (f64, f64), y: (f64, f64)) -> Self {
        Self {
            x_axis: [x.0, x.1],
            y_axis: [y.0, y.1],
            start_time: Instant::now(),
            last_time: Instant::now(),
        }
    }

    pub fn elapsed_since_start(&mut self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    pub fn elapsed_since_last(&mut self) -> f64 {
        self.last_time.elapsed().as_secs_f64()
    }

    pub fn update_last_time(&mut self) {
        self.last_time = Instant::now();
    }

    pub fn inc_x_axis(&mut self, n: f64) {
        self.x_axis[0] += n;
        self.x_axis[1] += n;
    }

    pub fn set_y_max(&mut self, y: f64) {
        self.y_axis[1] = y;
    }

    pub fn set_y_min(&mut self, y: f64) {
        self.y_axis[0] = y;
    }

    pub fn set_if_y_max(&mut self, y: f64) {
        if y > self.max_y() {
            self.set_y_max(y)
        }
    }

    pub fn set_if_y_min(&mut self, y: f64) {
        if y < self.min_y() {
            self.set_y_min(y)
        }
    }

    pub fn max_y(&self) -> f64 {
        self.y_axis[1]
    }

    pub fn min_y(&self) -> f64 {
        self.y_axis[0]
    }

    pub fn max_x(&self) -> f64 {
        self.x_axis[1]
    }

    #[allow(dead_code)]
    pub fn min_x(&self) -> f64 {
        self.x_axis[0]
    }

    pub fn y(&self) -> [f64; 2] {
        self.y_axis
    }

    pub fn x(&self) -> [f64; 2] {
        self.x_axis
    }
}

#[derive(Default, Debug)]
pub(crate) struct RxTx<T: Default + Debug + AddAssign>(pub (T, T));
impl<T: Default + Debug + AddAssign> RxTx<T> {
    pub(crate) fn rx(&self) -> &T {
        &self.0.0
    }
    pub(crate) fn tx(&self) -> &T {
        &self.0.1
    }
    pub(crate) fn inc(&mut self, r: T, t: T) {
        self.0.0 += r;
        self.0.1 += t;
    }
}
impl RxTx<f64> {
    pub(crate) fn rx_speed_str(&self) -> String {
        format!("{}/s", conv_fb(*self.rx()))
    }
    pub(crate) fn tx_speed_str(&self) -> String {
        format!("{}/s", conv_fb(*self.rx()))
    }
    pub(crate) fn rx_bytes_str(&self) -> String {
        conv_fb(*self.rx())
    }
    pub(crate) fn tx_bytes_str(&self) -> String {
        conv_fb(*self.tx())
    }
}
