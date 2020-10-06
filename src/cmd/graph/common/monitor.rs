use std::time::Instant;

#[derive(Debug)]
pub(crate) struct Monitor {
    x_axis: [f64; 2],
    y_axis: [f64; 2],
    start_time: Instant,
    last_time: Instant,
}
impl Monitor {
    pub fn new(x: (f64, f64), y: (f64, f64)) -> Self {
        Self {
            x_axis: [x.0, x.1],
            y_axis: [y.0, y.1],
            start_time: Instant::now(),
            last_time: Instant::now(),
        }
    }

    pub fn elapsed_since_start(&mut self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    pub fn elapsed_since_last(&mut self) -> f64 {
        self.last_time.elapsed().as_secs_f64()
    }

    pub fn update_last_time(&mut self) {
        self.last_time = Instant::now();
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
}
