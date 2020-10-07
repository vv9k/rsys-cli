use super::{
    common::{single_widget_loop, GraphWidget, Statistic},
    events::Config,
    monitor::CpuMonitor,
    DataSeries, Monitor,
};
use crate::util::conv_hz;
use crate::util::random_color;
use anyhow::{anyhow, Result};
use rsys::linux::cpu::{processor, Core};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Row, Table},
    Frame,
};

const X_AXIS: (f64, f64) = (0., 30.0);
const FREQUENCY_Y_AXIS: (f64, f64) = (f64::MAX, 0.);
const TICK_RATE: u64 = 250;
const CPU_INFO_HEADERS: &[&str] = &["core", "frequency"];

// Stats of a single core
pub struct CoreFrequencyStat {
    name: String,
    color: Color,
    frequency_data: DataSeries,
    core: Core,
}
impl From<Core> for CoreFrequencyStat {
    fn from(core: Core) -> Self {
        Self {
            name: format!("cpu{}", core.id),
            color: random_color(Some(20)),
            frequency_data: DataSeries::new(),
            core,
        }
    }
}
impl Statistic for CoreFrequencyStat {
    // Updates core and returns its new frequency
    fn update(&mut self, m: &mut Monitor) -> Result<()> {
        self.core
            .update()
            .map_err(|e| anyhow!("Failed to update core `{}` frequency - {}", self.name, e))?;
        let freq = self.core.cur_freq as f64;
        self.frequency_data.add(m.elapsed_since_start(), freq);

        m.set_if_y_max(freq + 100_000.);
        m.set_if_y_min(freq + 100_000.);

        Ok(())
    }
    fn data(&self) -> &DataSeries {
        &self.frequency_data
    }
    fn data_mut(&mut self) -> &mut DataSeries {
        &mut self.frequency_data
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn color(&self) -> Color {
        self.color
    }
}

impl CpuMonitor<CoreFrequencyStat> {
    pub fn new() -> Result<CpuMonitor<CoreFrequencyStat>> {
        Ok(CpuMonitor {
            stats: {
                let mut stats = processor()?
                    .cores
                    .into_iter()
                    .map(CoreFrequencyStat::from)
                    .collect::<Vec<CoreFrequencyStat>>();
                stats.sort_by(|s1, s2| s1.core.id.cmp(&s2.core.id));
                stats
            },
            m: Monitor::new(X_AXIS, FREQUENCY_Y_AXIS),
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
                Style::default().fg(s.color),
            )
        });

        let table =
            Table::new(CPU_INFO_HEADERS.iter(), data).widths(&[Constraint::Percentage(25), Constraint::Percentage(60)]);

        f.render_widget(table, chunks[1]);
    }

    pub fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Min(80)].as_ref())
            .split(area);

        self.render_cores_info_widget(f, chunks[0]);
        self.render_graph_widget(f, chunks[1]);
    }

    pub fn frequency_graph_loop() -> Result<()> {
        let mut monitor = CpuMonitor::<CoreFrequencyStat>::new()?;
        single_widget_loop(&mut monitor, Config::new(TICK_RATE))
    }
}
