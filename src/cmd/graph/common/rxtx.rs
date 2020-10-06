use crate::util::conv_fb;
use std::{fmt::Debug, ops::AddAssign};

#[derive(Default, Debug)]
pub(crate) struct RxTx<T: Default + Debug>(pub (T, T));
impl<T: Default + Debug> RxTx<T> {
    pub(crate) fn rx(&self) -> &T {
        &(self.0).0
    }
    pub(crate) fn tx(&self) -> &T {
        &(self.0).1
    }
    pub(crate) fn rx_mut(&mut self) -> &mut T {
        &mut (self.0).0
    }
    pub(crate) fn tx_mut(&mut self) -> &mut T {
        &mut (self.0).1
    }
}
impl<T: Default + Debug + AddAssign> RxTx<T> {
    pub(crate) fn inc(&mut self, r: T, t: T) {
        (self.0).0 += r;
        (self.0).1 += t;
    }
}
impl RxTx<f64> {
    pub(crate) fn rx_speed_str(&self) -> String {
        format!("{}/s", conv_fb(*self.rx()))
    }
    pub(crate) fn tx_speed_str(&self) -> String {
        format!("{}/s", conv_fb(*self.rx()))
    }
    pub(crate) fn rx_bytes_str(&self) -> String {
        conv_fb(*self.rx())
    }
    pub(crate) fn tx_bytes_str(&self) -> String {
        conv_fb(*self.tx())
    }
}
