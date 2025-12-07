#![forbid(unsafe_code)]

use futures::future::FusedFuture;
use std::time::Duration;
use wgame_app::{
    runtime::sleep_until,
    sleep,
    time::{Instant, Timer},
};

pub struct PeriodicTimer {
    timer: Timer,
    period: Duration,
}

impl PeriodicTimer {
    pub fn new(period: Duration) -> Self {
        dbg!(period);
        Self {
            timer: sleep(period),
            period,
        }
    }

    pub fn period(&self) -> Duration {
        self.period
    }

    pub fn elapsed_periods(&mut self) -> Duration {
        if self.timer.is_terminated() {
            let now = Instant::now();
            let elapsed = now - self.timer.timestamp() + self.period;
            let n_periods = elapsed.div_duration_f32(self.period);
            let last_timestamp = self.timer.timestamp() + self.period.mul_f32(n_periods);
            let elapsed = last_timestamp - self.timer.timestamp();
            let next_timestamp = last_timestamp + self.period;
            self.timer = sleep_until(next_timestamp);
            elapsed
        } else {
            Duration::ZERO
        }
    }

    pub async fn wait_next(&mut self) -> Duration {
        (&mut self.timer).await;
        self.elapsed_periods()
    }
}
