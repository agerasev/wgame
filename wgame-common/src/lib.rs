pub trait Window {
    type Inner;
    fn inner(&self) -> Self::Inner;
    fn size(&self) -> (u32, u32);
}

pub trait Frame {
    fn resized(&self) -> Option<(u32, u32)>;
    fn pre_present(&mut self);
}
