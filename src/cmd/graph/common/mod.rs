mod data;
mod display;
mod monitor;
mod rxtx;

pub(crate) use data::DataSeries;
pub(crate) use display::*;
pub(crate) use monitor::Monitor;
pub(crate) use rxtx::RxTx;

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
