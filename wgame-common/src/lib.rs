#![forbid(unsafe_code)]

pub trait Frame {
    fn resized(&self) -> Option<(u32, u32)>;
    fn pre_present(&mut self);
}
