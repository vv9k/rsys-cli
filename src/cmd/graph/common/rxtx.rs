use crate::util::conv_fb;
use std::{fmt::Debug, ops::AddAssign};

#[derive(Default, Debug)]
// Struct for grouping read(rx) and transfered(tx) values together
pub struct RxTx<T: Default + Debug>(pub (T, T));
impl<T: Default + Debug> RxTx<T> {
    /// Returns a reference to rx value
    pub fn rx(&self) -> &T {
        &(self.0).0
    }
    /// Returns a reference to tx value
    pub fn tx(&self) -> &T {
        &(self.0).1
    }
    /// Returns a mutable reference to rx value
    pub fn rx_mut(&mut self) -> &mut T {
        &mut (self.0).0
    }
    /// Returns a mutable reference to tx value
    pub fn tx_mut(&mut self) -> &mut T {
        &mut (self.0).1
    }
}
impl<T: Default + Debug + AddAssign> RxTx<T> {
    /// Increments both rx and transfered elements by coresponding values
    pub fn inc(&mut self, r: T, t: T) {
        (self.0).0.add_assign(r);
        (self.0).1.add_assign(t);
    }
}
impl RxTx<f64> {
    /// Returns rx value in scaled bytes/s as display string.
    pub fn rx_speed_str(&self) -> String {
        format!("{}/s", conv_fb(*self.rx()))
    }
    /// Returns tx value in scaled bytes/s as display string.
    pub fn tx_speed_str(&self) -> String {
        format!("{}/s", conv_fb(*self.rx()))
    }
    /// Returns scaled total rx bytes as display string.
    pub fn rx_bytes_str(&self) -> String {
        conv_fb(*self.rx())
    }
    /// Returns scaled total tx bytes as display string
    pub fn tx_bytes_str(&self) -> String {
        conv_fb(*self.tx())
    }
}
