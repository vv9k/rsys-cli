use super::{GraphSettings, GraphWidget, Screen, StatefulWidget, Statistic};
use anyhow::{anyhow, Result};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Dataset,
    Frame,
};

pub struct Monitor<S: Statistic> {
    pub stats: Vec<S>,
    pub m: Screen,
}

impl<S: Statistic> GraphWidget for Monitor<S> {
    default fn datasets(&self) -> Vec<Dataset> {
        Vec::new()
    }
    default fn monitor(&self) -> &Screen {
        &self.m
    }
    default fn settings(&self) -> GraphSettings {
        GraphSettings::default()
    }
}

impl<S: Statistic> StatefulWidget for Monitor<S> {
    fn update(&mut self) -> Result<()> {
        for stat in &mut self.stats {
            stat.update(&mut self.m)
                .map_err(|e| anyhow!("Failed to update widget statistics - `{}`", e))?;
        }
        self.m.update_last_time();

        // Move x axis if time reached end
        if self.m.elapsed_since_start() > self.m.max_x() {
            let delta = self.stats[0].pop();
            self.m.inc_x_axis(delta);

            self.stats.iter_mut().skip(1).for_each(|s| {
                s.pop();
            });
        }

        Ok(())
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
