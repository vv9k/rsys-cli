mod cpu;
mod events;
mod interface;
use crate::RsysCli;
use cpu::graph_cpu;
use interface::graph_net_interface;
use std::io::{self, stdout};
use structopt::StructOpt;
use termion::{
    input::MouseTerminal,
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};
use tui::{backend::TermionBackend, Terminal};

type Term = Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<io::Stdout>>>>>;
use anyhow::{anyhow, Result};

pub(crate) fn get_terminal() -> Result<Term> {
    let stdout = stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    Terminal::new(backend).map_err(|e| anyhow!("Failed to get terminal handle - {}", e))
}

#[derive(StructOpt, Clone)]
pub enum GraphCmd {
    /// Draw interface rx/tx speed
    Interface { name: String },
    /// Draw core frequencies
    Cpu,
}

impl RsysCli {
    pub(crate) fn graph(&self, cmd: GraphCmd) {
        let result = match cmd {
            GraphCmd::Interface { name } => graph_net_interface(&name),
            GraphCmd::Cpu => graph_cpu(),
        };

        if let Err(e) = result {
            eprintln!("Error: {}", e);
        }
    }
}

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
    #[inline]
    /// Add a data point
    pub fn add(&mut self, time: f64, value: f64) {
        self.data.push((time, value));
        self.len += 1;
    }
    #[inline]
    /// Pop first point returning it. If data vector is empty
    /// return (0., 0.)
    pub fn pop(&mut self) -> (f64, f64) {
        if self.len > 0 {
            self.len -= 1;
            return self.data.remove(0);
        }
        (0., 0.)
    }
    #[inline]
    /// Return nth element of data set if such exists.
    pub fn nth(&self, n: usize) -> Option<(f64, f64)> {
        if n < self.len {
            return Some(self.data[n]);
        }
        None
    }
    #[inline]
    /// Return first element of data set if such exists.
    pub fn first(&self) -> Option<(f64, f64)> {
        self.nth(0)
    }
}

#[derive(Debug)]
pub(crate) struct Monitor {
    x_axis: [f64; 2],
    y_axis: [f64; 2],
    total_time: f64,
}
impl Monitor {
    pub fn new(x: (f64, f64), y: (f64, f64)) -> Self {
        Self {
            x_axis: [x.0, x.1],
            y_axis: [y.0, y.1],
            total_time: 0.,
        }
    }
    #[inline]
    pub fn add_time(&mut self, time: f64) {
        self.total_time += time;
    }
    #[inline]
    pub fn time(&self) -> f64 {
        self.total_time
    }
    #[inline]
    pub fn inc_x_axis(&mut self, n: f64) {
        self.x_axis[0] += n;
        self.x_axis[1] += n;
    }
    #[inline]
    pub fn set_y_max(&mut self, y: f64) {
        self.y_axis[1] = y;
    }
    #[inline]
    pub fn set_y_min(&mut self, y: f64) {
        self.y_axis[0] = y;
    }
    #[inline]
    pub fn set_if_y_max(&mut self, y: f64) {
        if y > self.max_y() {
            self.set_y_max(y)
        }
    }
    #[inline]
    pub fn set_if_y_min(&mut self, y: f64) {
        if y < self.min_y() {
            self.set_y_min(y)
        }
    }
    #[inline]
    pub fn max_y(&self) -> f64 {
        self.y_axis[1]
    }
    #[inline]
    pub fn min_y(&self) -> f64 {
        self.y_axis[0]
    }
    #[inline]
    pub fn max_x(&self) -> f64 {
        self.x_axis[1]
    }
    #[inline]
    #[allow(dead_code)]
    pub fn min_x(&self) -> f64 {
        self.x_axis[0]
    }
    #[inline]
    pub fn y(&self) -> [f64; 2] {
        self.y_axis
    }
    #[inline]
    pub fn x(&self) -> [f64; 2] {
        self.x_axis
    }
}
