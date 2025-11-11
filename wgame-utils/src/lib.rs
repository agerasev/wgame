#![forbid(unsafe_code)]

use std::{ops::Deref, time::Duration};

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
        self.count_ext().map(|guard| guard.per_second())
    }

    #[must_use]
    pub fn count_ext(&mut self) -> Option<CountGuard<'_>> {
        self.count += 1;
        let now = Instant::now();
        let elapsed = now - self.start;
        if elapsed > self.period {
            Some(CountGuard { owner: self, now })
        } else {
            None
        }
    }
}

pub struct CountGuard<'a> {
    owner: &'a mut FrameCounter,
    pub now: Instant,
}

impl CountGuard<'_> {
    pub fn elapsed(&self) -> Duration {
        self.now - self.start
    }
    pub fn per_second(&self) -> f32 {
        self.count as f32 / self.elapsed().as_secs_f32()
    }
}

impl Deref for CountGuard<'_> {
    type Target = FrameCounter;
    fn deref(&self) -> &Self::Target {
        self.owner
    }
}

impl Drop for CountGuard<'_> {
    fn drop(&mut self) {
        self.owner.start = self.now;
        self.owner.count = 0;
    }
}
