mod events;
mod interface;
use crate::RsysCli;
use interface::graph_net_interface;
use std::{
    error::Error,
    io::{self, stdout},
};
use structopt::StructOpt;
use termion::{
    input::MouseTerminal,
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};
use tui::{backend::TermionBackend, Terminal};

type Term = Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<io::Stdout>>>>>;

pub(crate) fn get_terminal() -> Result<Term, Box<dyn Error>> {
    let stdout = stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

#[derive(StructOpt, Clone)]
pub enum GraphCmd {
    Interface { name: String },
}

impl RsysCli {
    pub(crate) fn graph(&self, cmd: GraphCmd) {
        match cmd {
            GraphCmd::Interface { name } => graph_net_interface(&name).unwrap(),
        }
    }
}
