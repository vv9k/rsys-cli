mod common;
mod cpu;
mod events;
mod interface;
mod storage;
use crate::RsysCli;
use common::graph_all_loop;
use cpu::CpuMonitor;
use interface::IfaceMonitor;
use std::io::{self, stdout};
use storage::StorageMonitor;
use structopt::StructOpt;
use termion::{
    input::MouseTerminal,
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};
use tui::{backend::TermionBackend, Terminal};

type Term = Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<io::Stdout>>>>>;
use anyhow::{anyhow, Result};

pub fn get_terminal() -> Result<Term> {
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
    /// Display I/O stats for storage devices
    Storage,
    /// Display all graphs at once
    All,
}

impl RsysCli {
    pub fn graph(&self, cmd: GraphCmd) {
        let result = match cmd {
            GraphCmd::Interface { name } => IfaceMonitor::graph_loop(&name),
            GraphCmd::Cpu => CpuMonitor::graph_loop(),
            GraphCmd::Storage => StorageMonitor::graph_loop(),
            GraphCmd::All => graph_all_loop(),
        };

        if let Err(e) = result {
            eprintln!("Error: {}", e);
        }
    }
}
