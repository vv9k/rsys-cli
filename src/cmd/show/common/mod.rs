//! Module containing common functionality used across all widgets.
mod data;
mod display;
mod monitor;
mod rxtx;
mod screen;
mod widget;

pub use data::*;
pub use display::*;
pub use monitor::Monitor;
pub use rxtx::RxTx;
pub use screen::Screen;
pub use widget::*;

use super::{events, get_terminal};
