//! Utility types and functions for wgame.
//!
//! Provides periodic timer for handling regular intervals.

#![forbid(unsafe_code)]

use futures::future::FusedFuture;
use std::time::Duration;
use wgame_app::{
    runtime::sleep_until,
    sleep,
    time::{Instant, Timer},
};

/// A timer that preserves its deadline phase when ticks are missed.
/// [`Self::elapsed_periods`] and [`Self::wait_next`] return the duration of whole
/// elapsed periods, not an integer count. Use elapsed time for animation and
/// periodic timers for scheduled background work.
pub struct PeriodicTimer {
    timer: Timer,
    period: Duration,
}

impl PeriodicTimer {
    /// Creates a periodic timer in the current runtime.
    ///
    /// # Panics
    ///
    /// Panics for a zero period or when no runtime is active.
    pub fn new(period: Duration) -> Self {
        assert!(!period.is_zero(), "Timer period must be positive");
        Self {
            timer: sleep(period),
            period,
        }
    }

    /// Returns the period of the timer.
    pub fn period(&self) -> Duration {
        self.period
    }

    /// Returns the total duration of whole periods elapsed since the last wait.
    pub fn elapsed_periods(&mut self) -> Duration {
        if self.timer.is_terminated() {
            let now = Instant::now();
            let (next_timestamp, elapsed) = advance(self.timer.timestamp(), now, self.period);
            self.timer = sleep_until(next_timestamp);
            elapsed
        } else {
            Duration::ZERO
        }
    }

    /// Waits for the next timer fire and returns the elapsed periods.
    pub async fn wait_next(&mut self) -> Duration {
        (&mut self.timer).await;
        self.elapsed_periods()
    }
}

fn advance(deadline: Instant, now: Instant, period: Duration) -> (Instant, Duration) {
    let late = now - deadline;
    let rem = late.as_nanos() % period.as_nanos();
    let remainder = Duration::new((rem / 1_000_000_000) as u64, (rem % 1_000_000_000) as u32);
    let elapsed = late - remainder + period;
    (deadline + elapsed, elapsed)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn periodic_deadlines_keep_phase_and_count_missed_ticks() {
        let start = Instant::now();
        let period = Duration::from_millis(10);
        assert_eq!(advance(start, start, period), (start + period, period));
        assert_eq!(
            advance(start, start + Duration::from_millis(25), period),
            (start + Duration::from_millis(30), Duration::from_millis(30))
        );
    }
}
