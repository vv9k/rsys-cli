mod frequency;
mod monitor;
mod usage;

use super::{
    common::{self, DataSeries, Monitor},
    events,
};
use anyhow::Result;
use tui::style::Color;
