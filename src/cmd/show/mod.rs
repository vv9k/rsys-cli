mod common;
mod cpu;
mod events;
mod net;
mod ps;
mod storage;

use crate::RsysCli;
use common::StatefulWidget;
use cpu::{CoreFrequencyStat, CoreUsageStat, CpuMonitor};
use events::{Config, Event, Events};
use net::NetMonitor;
use ps::ProcessMonitor;
use storage::StorageMonitor;

use std::io::{self, stdout};
use structopt::StructOpt;
use termion::{
    input::MouseTerminal,
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Layout},
    Terminal,
};

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
pub enum ShowCmd {
    /// Draw interface rx/tx speed
    Interface {
        name: String,
    },
    /// Draw cpu usage
    CpuUsage,
    /// Draw cpu core frequencies
    CpuFreq,
    /// Display I/O stats for storage devices
    Storage,
    /// Display all graphs at once
    All,
    Ps,
    Net,
}

impl RsysCli {
    pub fn show(&self, cmd: ShowCmd) {
        let result = match cmd {
            ShowCmd::Interface { name } => NetMonitor::single_iface_loop(&name),
            ShowCmd::CpuFreq => CpuMonitor::<CoreFrequencyStat>::graph_loop(),
            ShowCmd::CpuUsage => CpuMonitor::<CoreUsageStat>::graph_loop(),
            ShowCmd::Storage => StorageMonitor::graph_loop(),
            ShowCmd::All => show_all_loop(),
            ShowCmd::Ps => ProcessMonitor::display_loop(),
            ShowCmd::Net => NetMonitor::graph_loop(None),
        };

        if let Err(e) = result {
            eprintln!("Error: {}", e);
        }
    }
}

/// A loop with all graph widgets groupped together
pub fn show_all_loop() -> Result<()> {
    let mut terminal = get_terminal()?;
    let events = Events::with_config(Config::new(200));
    let mut cpumon = CpuMonitor::<CoreFrequencyStat>::new()?;
    let mut ifacemon = NetMonitor::new(None)?;
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
