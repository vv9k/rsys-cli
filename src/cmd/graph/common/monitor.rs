use std::time::Instant;

#[derive(Debug)]
/// A helper struct for each monitor (cpu, storage, interface...) that
/// gives a more convienient api to x axis, y axis and time of measurement.
pub struct Monitor {
    x_axis: [f64; 2],
    y_axis: [f64; 2],
    /// Monitor initialization time
    start_time: Instant,
    /// Last measurement time
    last_time: Instant,
}
impl Monitor {
    /// Returns a new instance of Monitor given x and y axis ranges.
    pub fn new(x: (f64, f64), y: (f64, f64)) -> Self {
        Self {
            x_axis: [x.0, x.1],
            y_axis: [y.0, y.1],
            start_time: Instant::now(),
            last_time: Instant::now(),
        }
    }

    /// Returns time elapsed since start in seconds
    pub fn elapsed_since_start(&mut self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Returns time since last measurement in seconds
    pub fn elapsed_since_last(&mut self) -> f64 {
        self.last_time.elapsed().as_secs_f64()
    }

    /// Updates last measurement time to current time
    pub fn update_last_time(&mut self) {
        self.last_time = Instant::now();
    }

    /// Increment both ends of x axis by n
    pub fn inc_x_axis(&mut self, n: f64) {
        self.x_axis[0] += n;
        self.x_axis[1] += n;
    }

    /// Set second coordinate of y axis as y
    pub fn set_y_max(&mut self, y: f64) {
        self.y_axis[1] = y;
    }

    /// Set first coordinate of y axis as y
    pub fn set_y_min(&mut self, y: f64) {
        self.y_axis[0] = y;
    }

    /// Set second coordinate of y axis as y if y > current max
    pub fn set_if_y_max(&mut self, y: f64) {
        if y > self.max_y() {
            self.set_y_max(y)
        }
    }

    /// Set first coordinate of y axis as y if y < current min
    pub fn set_if_y_min(&mut self, y: f64) {
        if y < self.min_y() {
            self.set_y_min(y)
        }
    }

    /// Returns second coordinate of y
    pub fn max_y(&self) -> f64 {
        self.y_axis[1]
    }

    /// Returns first coordinate of y
    pub fn min_y(&self) -> f64 {
        self.y_axis[0]
    }

    /// Returns second coordinate of x
    pub fn max_x(&self) -> f64 {
        self.x_axis[1]
    }

    #[allow(dead_code)]
    /// Returns first coordinate of x
    pub fn min_x(&self) -> f64 {
        self.x_axis[0]
    }

    /// Returns y axis
    pub fn y(&self) -> [f64; 2] {
        self.y_axis
    }

    /// Returns x axis
    pub fn x(&self) -> [f64; 2] {
        self.x_axis
    }
}
