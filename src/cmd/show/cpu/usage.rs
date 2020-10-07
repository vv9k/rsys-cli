use super::{
    common::{single_widget_loop, Statistic},
    events::Config,
    monitor::CpuMonitor,
    DataSeries, Monitor,
};
use crate::util::random_color;
use anyhow::Result;
use rsys::linux::cpu::{processor, Core};
use tui::style::Color;

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
