//! Module containing common functionality used across all graphs widgets.
mod data;
mod display;
mod monitor;
mod rxtx;

pub use data::DataSeries;
pub use display::*;
pub use monitor::Monitor;
pub use rxtx::RxTx;

use super::{
    cpu::CpuMonitor,
    events::{Config, Event, Events},
    get_terminal,
    interface::IfaceMonitor,
    storage::StorageMonitor,
};
use anyhow::Result;
use rsys::linux;
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    Frame,
};

/// Trait grouping all graph widgets together providing functionality
/// like graph_loop.
pub trait StatefulWidget {
    fn update(&mut self);
    fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect);
}

/// Loop a single widget on full screen endlessly
pub fn graph_loop<W: StatefulWidget>(widget: &mut W, config: Config) -> Result<()> {
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

/// A loop with all graph widgets groupped together
pub fn graph_all_loop() -> Result<()> {
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
