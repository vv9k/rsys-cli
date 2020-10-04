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
