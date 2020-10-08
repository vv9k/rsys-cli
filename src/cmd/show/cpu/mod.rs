mod frequency;
mod usage;

use super::{
    common::{self, Monitor, Statistic},
    events,
};

pub use frequency::CoreFrequencyStat;
pub use usage::CoreUsageStat;

pub struct CpuMonitor<S: Statistic> {
    pub stats: Vec<S>,
    pub m: Monitor,
}
