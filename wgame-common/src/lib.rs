#![forbid(unsafe_code)]

pub trait Window {
    type Handle;
    fn handle(&self) -> Self::Handle;

    fn size(&self) -> (u32, u32);

    type Frame<'a>: Frame
    where
        Self: 'a;
    fn next_frame(&mut self) -> impl Future<Output = Option<Self::Frame<'_>>>;
}

pub trait Frame {
    fn resized(&self) -> Option<(u32, u32)>;
    fn pre_present(&mut self);
}
