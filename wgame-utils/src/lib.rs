#![forbid(unsafe_code)]
#![no_std]

#[cfg(not(feature = "web"))]
extern crate std;

#[cfg(not(feature = "web"))]
use std::time::Instant;
#[cfg(feature = "web")]
use web_time::Instant;

pub struct FrameCounter {
    pub start: Instant,
    pub count: usize,
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            count: 0,
        }
    }
}

impl FrameCounter {
    pub fn count(&mut self) {
        self.count += 1;

        let now = Instant::now();
        let secs = (now - self.start).as_secs_f32();
        if secs > 10.0 {
            log::info!("FPS: {}", self.count as f32 / secs);
            self.start = now;
            self.count = 0;
        }
    }
}
