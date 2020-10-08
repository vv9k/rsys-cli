use super::{
    common::{single_widget_loop, DataSeries, GraphSettings, GraphWidget, Monitor, Screen, StatefulWidget, Statistic},
    events::Config,
};
use crate::util::random_color;
use crate::util::{conv_fhz, conv_hz, conv_t};
use anyhow::{anyhow, Result};
use rsys::linux::cpu::{processor, Core};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Dataset, Row, Table},
    Frame,
};

const X_AXIS: (f64, f64) = (0., 30.0);
const FREQUENCY_Y_AXIS: (f64, f64) = (f64::MAX, 0.);
const TICK_RATE: u64 = 200;
const CPU_INFO_HEADERS: &[&str] = &["core", "frequency"];

// Stats of a single core
pub struct CoreFrequencyStat {
    name: String,
    frequency_data: DataSeries,
    core: Core,
}
impl From<Core> for CoreFrequencyStat {
    fn from(core: Core) -> Self {
        Self {
            name: format!("cpu{}", core.id),
            frequency_data: DataSeries::new(random_color(Some(20))),
            core,
        }
    }
}
impl Statistic for CoreFrequencyStat {
    // Updates core and returns its new frequency
    fn update(&mut self, m: &mut Screen) -> Result<()> {
        self.core
            .update()
            .map_err(|e| anyhow!("Failed to update core `{}` frequency - {}", self.name, e))?;
        let freq = self.core.cur_freq as f64;
        self.frequency_data.add(m.elapsed_since_start(), freq);

        m.set_if_y_max(freq + 100_000.);
        m.set_if_y_min(freq + 100_000.);

        Ok(())
    }
    fn pop(&mut self) -> f64 {
        let removed = self.frequency_data.pop();
        if let Some(point) = self.frequency_data.first() {
            return point.0 - removed.0;
        }
        0.
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl Monitor<CoreFrequencyStat> {
    pub fn new() -> Result<Monitor<CoreFrequencyStat>> {
        Ok(Monitor {
            stats: {
                let mut stats = processor()?
                    .cores
                    .into_iter()
                    .map(CoreFrequencyStat::from)
                    .collect::<Vec<CoreFrequencyStat>>();
                stats.sort_by(|s1, s2| s1.core.id.cmp(&s2.core.id));
                stats
            },
            m: Screen::new(X_AXIS, FREQUENCY_Y_AXIS),
        })
    }

    fn render_cores_info_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(10), Constraint::Percentage(90)])
            .split(area);

        let data = self.stats.iter().map(|s| {
            Row::StyledData(
                vec![s.name.clone(), conv_hz(s.core.cur_freq)].into_iter(),
                Style::default().fg(s.frequency_data.color),
            )
        });

        let table =
            Table::new(CPU_INFO_HEADERS.iter(), data).widths(&[Constraint::Percentage(25), Constraint::Percentage(60)]);

        f.render_widget(table, chunks[1]);
    }

    pub fn graph_loop() -> Result<()> {
        let mut monitor = Self::new()?;
        single_widget_loop(&mut monitor, Config::new(TICK_RATE))
    }
}

impl GraphWidget for Monitor<CoreFrequencyStat> {
    fn datasets(&self) -> Vec<Dataset> {
        let mut data = Vec::new();
        for core in &self.stats {
            data.push(
                Dataset::default()
                    .name(core.name())
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(core.frequency_data.color))
                    .data(&core.frequency_data.dataset()),
            );
        }
        data
    }
    fn settings(&self) -> GraphSettings {
        GraphSettings::new()
            .title(
                "Cpu Frequency",
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue),
            )
            .x_title("Time", Style::default().fg(Color::White))
            .y_title("Frequency", Style::default().fg(Color::White))
            .x_labels(self.m.x_bounds_labels(conv_t, 4))
            .y_labels(self.m.y_bounds_labels(conv_fhz, 4))
    }
}

impl StatefulWidget for Monitor<CoreFrequencyStat> {
    // Override
    fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Min(80)].as_ref())
            .split(area);

        self.render_cores_info_widget(f, chunks[0]);
        self.render_graph_widget(f, chunks[1]);
    }
}
