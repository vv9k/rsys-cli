//! Module containing common functionality used across all widgets.
mod data;
mod display;
mod monitor;
mod rxtx;

pub use data::*;
pub use display::*;
pub use monitor::Monitor;
pub use rxtx::RxTx;

use super::{
    events::{Config, Event, Events},
    get_terminal,
};
use anyhow::Result;
use std::borrow::Cow;
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset},
    Frame,
};

/// Trait grouping all widgets with state that needs updating
/// together providing functionality like single_widget_loop.
pub trait StatefulWidget {
    fn update(&mut self);
    fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect);
}

/// Trait providing more readable way of creating graph widgets
pub trait GraphWidget {
    fn datasets(&self) -> Vec<Dataset> {
        Vec::new()
    }
    fn settings(&self) -> GraphSettings {
        GraphSettings::default()
    }
    fn monitor(&self) -> &Monitor;

    fn chart(&self) -> Chart {
        Chart::new(self.datasets())
            .block(Block::default().title(self.settings().title).borders(Borders::ALL))
            .x_axis(
                Axis::default()
                    .title(self.settings().x_title)
                    .labels(self.settings().x_labels)
                    .bounds(self.monitor().x()),
            )
            .y_axis(
                Axis::default()
                    .title(self.settings().y_title)
                    .labels(self.settings().y_labels)
                    .bounds(self.monitor().y()),
            )
    }
    fn render_graph_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chart = self.chart();
        f.render_widget(chart, area);
    }
}

/// Loop a single widget on full screen endlessly
pub fn single_widget_loop<W: StatefulWidget>(widget: &mut W, config: Config) -> Result<()> {
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

pub struct GraphSettings<'t, 'l> {
    pub title: Span<'t>,
    pub x_title: Span<'t>,
    pub y_title: Span<'t>,
    pub x_labels: Vec<Span<'l>>,
    pub y_labels: Vec<Span<'l>>,
}
impl<'t, 'l> Default for GraphSettings<'t, 'l> {
    fn default() -> Self {
        Self {
            title: Span::raw(""),
            x_title: Span::raw(""),
            y_title: Span::raw(""),
            x_labels: Vec::new(),
            y_labels: Vec::new(),
        }
    }
}
impl<'t, 'l> GraphSettings<'t, 'l> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn title<S: Into<Cow<'t, str>>>(mut self, title: S, style: Style) -> Self {
        self.title = Span::styled(title.into(), style);
        self
    }
    pub fn x_title<S: Into<Cow<'t, str>>>(mut self, x_axis: S, style: Style) -> Self {
        self.x_title = Span::styled(x_axis.into(), style);
        self
    }
    pub fn y_title<S: Into<Cow<'t, str>>>(mut self, y_axis: S, style: Style) -> Self {
        self.y_title = Span::styled(y_axis.into(), style);
        self
    }
    pub fn x_labels(mut self, x_labels: Vec<Span<'l>>) -> Self {
        self.x_labels = x_labels;
        self
    }
    pub fn y_labels(mut self, y_labels: Vec<Span<'l>>) -> Self {
        self.y_labels = y_labels;
        self
    }
}
