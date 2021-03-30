mod common;
mod cpu;
mod events;
mod net;
mod ps;
mod storage;

use crate::RsysCli;
use common::{err_popup, Monitor, StatefulWidget};
use cpu::{CoreFrequencyStat, CoreUsageStat};
use events::{Config, Event, Events};
use net::IfaceSpeedStat;
use ps::ProcessMonitor;
use storage::StorageSpeedStat;

use anyhow::Error;
use std::io::{self, stdout};
use structopt::StructOpt;
use termion::{
    event::Key,
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
    Interface { name: String },
    /// Draw cpu usage
    CpuUsage,
    /// Draw cpu core frequencies
    CpuFreq,
    /// Display I/O stats for storage devices
    Storage,
    /// Display network interfaces graphs
    Net,
    /// Display process list
    Ps,
    /// Display all graphs at once
    All,
}

impl RsysCli {
    pub fn show(&self, cmd: ShowCmd) {
        let result = match cmd {
            ShowCmd::Interface { name } => Monitor::<IfaceSpeedStat>::single_iface_loop(&name),
            ShowCmd::CpuFreq => Monitor::<CoreFrequencyStat>::graph_loop(),
            ShowCmd::CpuUsage => Monitor::<CoreUsageStat>::graph_loop(),
            ShowCmd::Storage => Monitor::<StorageSpeedStat>::graph_loop(),
            ShowCmd::Net => Monitor::<IfaceSpeedStat>::graph_loop(None),
            ShowCmd::Ps => ProcessMonitor::display_loop(),
            ShowCmd::All => show_all_loop(),
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
    let mut cpumon = Monitor::<CoreFrequencyStat>::new()?;
    let mut ifacemon = Monitor::<IfaceSpeedStat>::new(None)?;
    let mut stormon = Monitor::<StorageSpeedStat>::new()?;
    let mut errors: Vec<Error> = Vec::new();
    let mut show_errors = true;
    let mut was_error = false;
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

            if !errors.is_empty() && show_errors {
                let error = &errors[0];
                was_error = true;
                err_popup(f, &error.to_string(), "`q` - quit, `i` - ignore errors, `c` - continue")
            } else {
                cpumon.render_widget(f, layout[0]);
                ifacemon.render_widget(f, layout[1]);
                stormon.render_widget(f, layout[2]);
            }
        })?;

        match events.next()? {
            Event::Input(input) => {
                if input == events.exit_key() {
                    break;
                }

                match input {
                    Key::Char('c') if was_error => {
                        errors.pop();
                        was_error = false;
                    }
                    Key::Char('i') => show_errors = false,
                    _ => {}
                }
            }
            Event::Tick => {
                if let Err(e) = cpumon.update() {
                    errors.push(e);
                }
                if let Err(e) = ifacemon.update() {
                    errors.push(e);
                }
                if let Err(e) = stormon.update() {
                    errors.push(e);
                }
            }
        }
    }

    Ok(())
}
