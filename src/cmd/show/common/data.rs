use super::screen::Screen;
use anyhow::Result;
use tui::style::Color;

#[derive(Debug)]
/// Wrapper stuct for graph datapoints used by Datasets.
pub struct DataSeries {
    data: Vec<(f64, f64)>,
    pub color: Color,
    len: usize,
}

impl Default for DataSeries {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            color: Color::White,
            len: 0,
        }
    }
}

impl DataSeries {
    pub fn new(color: Color) -> Self {
        Self {
            data: Vec::new(),
            color,
            len: 0,
        }
    }
    /// Return self data as slice readable by tui's Dataset
    pub fn dataset(&self) -> &[(f64, f64)] {
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

pub trait Statistic {
    /// Updates the value of this stat
    fn update(&mut self, m: &mut Screen) -> Result<()>;
    /// Pops all datasets returning time delta between popped
    /// element and new first element of set
    fn pop(&mut self) -> f64;
    fn name(&self) -> &str;
}
