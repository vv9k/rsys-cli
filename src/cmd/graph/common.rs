use super::events::{Config, Event, Events};
use anyhow::{anyhow, Result};
use termion::event::Key;

/// Wrapper stuct for graph datapoints used by Datasets.
pub(crate) struct DataSeries {
    data: Vec<(f64, f64)>,
    len: usize,
}
impl DataSeries {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            len: 0,
        }
    }
    pub fn data(&self) -> &[(f64, f64)] {
        &self.data
    }

    /// Add a data point
    pub fn add(&mut self, time: f64, value: f64) {
        self.data.push((time, value));
        self.len += 1;
    }

    /// Pop first point returning it. If data vector is empty
    /// return (0., 0.)
    pub fn pop(&mut self) -> (f64, f64) {
        if self.len > 0 {
            self.len -= 1;
            return self.data.remove(0);
        }
        (0., 0.)
    }

    /// Return nth element of data set if such exists.
    pub fn nth(&self, n: usize) -> Option<(f64, f64)> {
        if n < self.len {
            return Some(self.data[n]);
        }
        None
    }

    /// Return first element of data set if such exists.
    pub fn first(&self) -> Option<(f64, f64)> {
        self.nth(0)
    }
}

#[derive(Debug)]
pub(crate) struct Monitor {
    x_axis: [f64; 2],
    y_axis: [f64; 2],
    total_time: f64,
    events: Events,
}
impl Monitor {
    pub fn new(x: (f64, f64), y: (f64, f64), config: Config) -> Self {
        Self {
            x_axis: [x.0, x.1],
            y_axis: [y.0, y.1],
            total_time: 0.,
            events: Events::with_config(config),
        }
    }

    pub fn add_time(&mut self, time: f64) {
        self.total_time += time;
    }

    pub fn time(&self) -> f64 {
        self.total_time
    }

    pub fn next_event(&mut self) -> Result<Event<Key>> {
        self.events
            .next()
            .map_err(|e| anyhow!("Failed to get next key event - {}", e))
    }

    pub fn inc_x_axis(&mut self, n: f64) {
        self.x_axis[0] += n;
        self.x_axis[1] += n;
    }

    pub fn set_y_max(&mut self, y: f64) {
        self.y_axis[1] = y;
    }

    pub fn set_y_min(&mut self, y: f64) {
        self.y_axis[0] = y;
    }

    pub fn set_if_y_max(&mut self, y: f64) {
        if y > self.max_y() {
            self.set_y_max(y)
        }
    }

    pub fn set_if_y_min(&mut self, y: f64) {
        if y < self.min_y() {
            self.set_y_min(y)
        }
    }

    pub fn max_y(&self) -> f64 {
        self.y_axis[1]
    }

    pub fn min_y(&self) -> f64 {
        self.y_axis[0]
    }

    pub fn max_x(&self) -> f64 {
        self.x_axis[1]
    }

    #[allow(dead_code)]
    pub fn min_x(&self) -> f64 {
        self.x_axis[0]
    }

    pub fn y(&self) -> [f64; 2] {
        self.y_axis
    }

    pub fn x(&self) -> [f64; 2] {
        self.x_axis
    }

    pub fn exit_key(&self) -> Key {
        self.events.exit_key()
    }
}
