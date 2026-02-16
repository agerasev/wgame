//! Utility types and functions for wgame.
//!
//! This crate provides common utility types that are used across the wgame ecosystem.
//! Currently, it includes the [`PeriodicTimer`] for handling periodic tasks.

#![forbid(unsafe_code)]

use futures::future::FusedFuture;
use std::time::Duration;
use wgame_app::{
    runtime::sleep_until,
    sleep,
    time::{Instant, Timer},
};

/// A timer that fires at regular intervals.
///
/// This struct provides a convenient way to run code at a fixed frequency.
/// It tracks the number of periods that have elapsed and can be used to
/// synchronize code to a specific update rate.
///
/// # Examples
///
/// ```no_run
/// # use wgame_utils::PeriodicTimer;
/// # use std::time::Duration;
/// # async fn example() {
/// let mut timer = PeriodicTimer::new(Duration::from_secs_f32(1.0 / 60.0));
///
/// loop {
///     timer.wait_next().await;
///     // Update game logic here
/// }
/// # }
/// ```
pub struct PeriodicTimer {
    timer: Timer,
    period: Duration,
}

impl PeriodicTimer {
    /// Creates a new periodic timer with the given period.
    ///
    /// # Arguments
    ///
    /// * `period` - The duration between each timer fire.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use wgame_utils::PeriodicTimer;
    /// # use std::time::Duration;
    /// let timer = PeriodicTimer::new(Duration::from_secs(1));
    /// ```
    pub fn new(period: Duration) -> Self {
        Self {
            timer: sleep(period),
            period,
        }
    }

    /// Returns the period of the timer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use wgame_utils::PeriodicTimer;
    /// # use std::time::Duration;
    /// let timer = PeriodicTimer::new(Duration::from_secs(1));
    /// assert_eq!(timer.period(), Duration::from_secs(1));
    /// ```
    pub fn period(&self) -> Duration {
        self.period
    }

    /// Returns the number of periods that have elapsed since the last wait.
    ///
    /// This method calculates how many periods have passed since the timer
    /// was last reset. If the timer hasn't fired yet, it returns zero.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use wgame_utils::PeriodicTimer;
    /// # use std::time::Duration;
    /// # async fn example() {
    /// let mut timer = PeriodicTimer::new(Duration::from_secs(1));
    /// assert_eq!(timer.elapsed_periods(), Duration::ZERO);
    /// # }
    /// ```
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

    /// Waits for the next timer fire and returns the elapsed periods.
    ///
    /// This async method waits for the timer to fire and then calculates
    /// how many periods have elapsed. This is useful for updating game
    /// logic at a fixed rate while accounting for time dilation.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use wgame_utils::PeriodicTimer;
    /// # use std::time::Duration;
    /// # async fn example() {
    /// let mut timer = PeriodicTimer::new(Duration::from_secs(1));
    /// let elapsed = timer.wait_next().await;
    /// println!("Elapsed periods: {:?}", elapsed);
    /// # }
    /// ```
    pub async fn wait_next(&mut self) -> Duration {
        (&mut self.timer).await;
        self.elapsed_periods()
    }
}
