use super::{
    common::{GraphWidget, Monitor, StatefulWidget},
    Statistic,
};
use crate::util::{conv_fhz, conv_t};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::Dataset,
    Frame,
};

pub struct CpuMonitor<S: Statistic> {
    pub stats: Vec<S>,
    pub m: Monitor,
}

impl<S: Statistic> StatefulWidget for CpuMonitor<S> {
    fn update(&mut self) {
        // Update frequencies on cores
        for core in &mut self.stats {
            // TODO: handle err here somehow
            core.update(&mut self.m).unwrap();
        }

        // Move x axis if time reached end
        if self.m.elapsed_since_start() > self.m.max_x() {
            let removed = self.stats[0].data_mut().pop();
            if let Some(point) = self.stats[0].data_mut().first() {
                self.m.inc_x_axis(point.0 - removed.0);
            }
            self.stats.iter_mut().skip(1).for_each(|c| {
                c.data_mut().pop();
            });
        }
    }
    fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Min(80)].as_ref())
            .split(area);

        self.render_graph_widget(f, chunks[1]);
    }
}

impl<S: Statistic> GraphWidget for CpuMonitor<S> {
    fn datasets(&self) -> Vec<Dataset> {
        let mut data = Vec::new();
        for core in &self.stats {
            data.push(
                Dataset::default()
                    .name(core.name())
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(core.color()))
                    .data(&core.data().dataset()),
            );
        }
        data
    }
    fn title(&self) -> Span {
        Span::styled(
            "Cpu Frequency",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue),
        )
    }
    fn x_axis(&self) -> Span {
        Span::styled("Time", Style::default().fg(Color::White))
    }
    fn y_axis(&self) -> Span {
        Span::styled("Frequency", Style::default().fg(Color::White))
    }
    fn y_labels(&self) -> Vec<Span> {
        self.m.y_bounds_labels(conv_fhz, 4)
    }
    fn x_labels(&self) -> Vec<Span> {
        self.m.x_bounds_labels(conv_t, 4)
    }
    fn monitor(&self) -> &Monitor {
        &self.m
    }
}
