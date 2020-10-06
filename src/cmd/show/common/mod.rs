//! Module containing common functionality used across all widgets.
mod data;
mod display;
mod monitor;
mod rxtx;

pub use data::DataSeries;
pub use display::*;
pub use monitor::Monitor;
pub use rxtx::RxTx;

use super::{
    events::{Config, Event, Events},
    get_terminal,
};
use anyhow::Result;
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
    fn datasets(&self) -> Vec<Dataset>;
    fn title(&self) -> (&str, Style);
    // Name of x axis
    fn x_axis(&self) -> (&str, Style);
    // Name of y axis
    fn y_axis(&self) -> (&str, Style);
    fn labels(&self) -> Vec<Span>;
    fn monitor(&self) -> &Monitor;

    fn chart(&self) -> Chart {
        let (title, title_style) = self.title();
        let (x_name, x_style) = self.x_axis();
        let (y_name, y_style) = self.y_axis();
        Chart::new(self.datasets())
            .block(
                Block::default()
                    .title(Span::styled(title, title_style))
                    .borders(Borders::ALL),
            )
            .x_axis(Axis::default().title(x_name).style(x_style).bounds(self.monitor().x()))
            .y_axis(
                Axis::default()
                    .title(y_name)
                    .labels(self.labels())
                    .style(y_style)
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
