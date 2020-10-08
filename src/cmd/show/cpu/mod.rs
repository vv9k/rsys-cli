mod frequency;
mod usage;

use super::{
    common::{self, GraphSettings, GraphWidget, Monitor, Statistic},
    events, StatefulWidget,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Dataset,
    Frame,
};

pub use frequency::CoreFrequencyStat;
pub use usage::CoreUsageStat;

pub struct CpuMonitor<S: Statistic> {
    pub stats: Vec<S>,
    pub m: Monitor,
}

impl<S: Statistic> GraphWidget for CpuMonitor<S> {
    default fn datasets(&self) -> Vec<Dataset> {
        Vec::new()
    }
    default fn monitor(&self) -> &Monitor {
        &self.m
    }
    default fn settings(&self) -> GraphSettings {
        GraphSettings::default()
    }
}

impl<S: Statistic> StatefulWidget for CpuMonitor<S> {
    fn update(&mut self) {
        // Update frequencies on cores
        for core in &mut self.stats {
            // TODO: handle err here somehow
            core.update(&mut self.m).unwrap();
        }
        self.m.update_last_time();

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
    // By default widget is rendered on full area. If a monitor of some
    // statistic wants to display more widgets it has to override this method
    default fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(area);

        self.render_graph_widget(f, chunks[0]);
    }
}
