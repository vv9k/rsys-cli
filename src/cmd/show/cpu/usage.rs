use super::{
    common::{single_widget_loop, DataSeries, GraphWidget, Monitor, StatefulWidget, Statistic},
    events::Config,
    CpuMonitor,
};
use crate::util::{conv_p, conv_t, random_color};
use anyhow::Result;
use rsys::linux::cpu::{processor, Core};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::Dataset,
    Frame,
};

const X_AXIS: (f64, f64) = (0., 30.0);
const USAGE_Y_AXIS: (f64, f64) = (0., 100.);
const TICK_RATE: u64 = 250;

pub struct CoreUsageStat {
    name: String,
    color: Color,
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
            color: random_color(Some(20)),
            data: DataSeries::new(),
            last_total_time: 0.,
            last_idle_time: 0.,
            last_usage: 0.,
            core,
        }
    }
}
impl Statistic for CoreUsageStat {
    fn update(&mut self, m: &mut Monitor) -> Result<()> {
        if let Some(times) = self.core.cpu_time()? {
            let total_time = (times.user + times.nice + times.system + times.iowait + times.irq + times.softirq) as f64;
            let idle_time = times.idle as f64;
            self.last_usage = (1. - (idle_time - self.last_idle_time) / (total_time - self.last_total_time)) * 100.;
            self.data.add(m.elapsed_since_start(), self.last_usage);
            self.last_total_time = total_time;
            self.last_idle_time = idle_time;
        }

        Ok(())
    }
    fn data(&self) -> &DataSeries {
        &self.data
    }
    fn data_mut(&mut self) -> &mut DataSeries {
        &mut self.data
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn color(&self) -> Color {
        self.color
    }
}

impl CpuMonitor<CoreUsageStat> {
    pub fn new() -> Result<CpuMonitor<CoreUsageStat>> {
        Ok(CpuMonitor {
            stats: {
                let mut stats = processor()?
                    .cores
                    .into_iter()
                    .map(CoreUsageStat::from)
                    .collect::<Vec<CoreUsageStat>>();
                stats.sort_by(|s1, s2| s1.core.id.cmp(&s2.core.id));
                stats
            },
            m: Monitor::new(X_AXIS, USAGE_Y_AXIS),
        })
    }
    pub fn usage_graph_loop() -> Result<()> {
        let mut monitor = CpuMonitor::<CoreUsageStat>::new()?;
        single_widget_loop(&mut monitor, Config::new(TICK_RATE))
    }
}

impl GraphWidget for CpuMonitor<CoreUsageStat> {
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
        self.m.y_bounds_labels(conv_p, 4)
    }
    fn x_labels(&self) -> Vec<Span> {
        self.m.x_bounds_labels(conv_t, 4)
    }
    fn monitor(&self) -> &Monitor {
        &self.m
    }
}

impl StatefulWidget for CpuMonitor<CoreUsageStat> {
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
    // By default widget is rendered on full area. If a monitor of some
    // statistic wants to display more widgets it has to override this method
    fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(100)].as_ref())
            .split(area);

        self.render_graph_widget(f, chunks[0]);
    }
}
