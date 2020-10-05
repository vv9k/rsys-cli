mod common;
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