#![forbid(unsafe_code)]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::time::Duration;
#[cfg(feature = "std")]
use std::time::Instant;
#[cfg(feature = "web")]
use web_time::Instant;

pub struct FrameCounter {
    pub start: Instant,
    pub count: usize,
    pub period: Duration,
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self::new(Duration::from_secs(10))
    }
}

impl FrameCounter {
    pub fn new(period: Duration) -> Self {
        Self {
            start: Instant::now(),
            count: 0,
            period,
        }
    }

    #[must_use]
    pub fn count(&mut self) -> Option<f32> {
        self.count += 1;
        let mut value = None;
        let now = Instant::now();
        let elapsed = now - self.start;
        if elapsed > self.period {
            value = Some(self.count as f32 / elapsed.as_secs_f32());
            self.start = now;
            self.count = 0;
        }
        value
    }
}
