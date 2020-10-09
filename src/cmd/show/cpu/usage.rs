use super::{
    common::{single_widget_loop, DataSeries, GraphSettings, GraphWidget, Monitor, Screen, Statistic},
    events::Config,
};
use crate::util::{conv_p, conv_t, random_color};
use anyhow::Result;
use rsys::linux::cpu::{processor, Core};
use tui::{
    style::{Color, Modifier, Style},
    symbols,
    widgets::Dataset,
};

const X_AXIS: (f64, f64) = (0., 30.0);
const USAGE_Y_AXIS: (f64, f64) = (0., 100.);
const TICK_RATE: u64 = 250;

#[derive(Debug)]
pub struct CoreUsageStat {
    name: String,
    data: DataSeries,
    last_total_time: f64,
    last_idle_time: f64,
    last_usage: f64,
    core: Core,
}
impl From<Core> for CoreUsageStat {
    fn from(core: Core) -> Self {
        Self {
            name: format!("cpu{}", core.id),
            data: DataSeries::new(random_color(Some(20))),
            last_total_time: 0.,
            last_idle_time: 0.,
            last_usage: 0.,
            core,
        }
    }
}
impl Statistic for CoreUsageStat {
    fn update(&mut self, m: &mut Screen) -> Result<()> {
        if let Some(times) = self.core.cpu_time()? {
            let total_time =
                (times.user + times.nice + times.system + times.iowait + times.irq + times.softirq + times.idle) as f64;
            let idle_time = times.idle as f64;
            let idle_delta = idle_time - self.last_idle_time;
            let total_delta = total_time - self.last_total_time;
            self.last_usage = 100. * (1.0 - idle_delta / total_delta);

            self.data.add(m.elapsed_since_start(), self.last_usage);
            self.last_total_time = total_time;
            self.last_idle_time = idle_time;
        }

        Ok(())
    }
    fn pop(&mut self) -> f64 {
        let removed = self.data.pop();
        if let Some(point) = self.data.first() {
            return point.0 - removed.0;
        }
        0.
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl Monitor<CoreUsageStat> {
    pub fn new() -> Result<Monitor<CoreUsageStat>> {
        Ok(Monitor {
            stats: {
                let mut stats = processor()?
                    .cores
                    .into_iter()
                    .map(CoreUsageStat::from)
                    .collect::<Vec<CoreUsageStat>>();
                stats.sort_by(|s1, s2| s1.core.id.cmp(&s2.core.id));
                stats
            },
            m: Screen::new(X_AXIS, USAGE_Y_AXIS),
        })
    }
    pub fn graph_loop() -> Result<()> {
        let mut monitor = Monitor::<CoreUsageStat>::new()?;
        single_widget_loop(&mut monitor, Config::new(TICK_RATE))
    }
}

impl GraphWidget for Monitor<CoreUsageStat> {
    fn datasets(&self) -> Vec<Dataset> {
        let mut data = Vec::new();
        for core in &self.stats {
            data.push(
                Dataset::default()
                    .name(core.name())
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(core.data.color))
                    .data(&core.data.dataset()),
            );
        }
        data
    }
    fn settings(&self) -> GraphSettings {
        GraphSettings::new()
            .title(
                "Cpu Usage",
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
            )
            .x_title("Time", Style::default().fg(Color::White))
            .y_title("Usage", Style::default().fg(Color::White))
            .x_labels(self.m.x_bounds_labels(conv_t, 4))
            .y_labels(self.m.y_bounds_labels(conv_p, 4))
    }
    fn monitor(&self) -> &Screen {
        &self.m
    }
}
