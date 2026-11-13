//! Engine clocking: fixed-timestep accumulator and a stopwatch for stats.

use std::time::{Duration, Instant};

/// Fixed-timestep clock using the accumulator pattern.
///
/// `tick` is called once per rendered frame with the current instant; it
/// returns how many fixed simulation steps must run this frame. `alpha`
/// then gives the interpolation fraction for rendering.
///
/// A per-frame clamp (`max_frame_delta`) prevents the "spiral of death"
/// after long stalls (window drag, debugger break): extra time is dropped
/// rather than forcing unbounded catch-up steps.
#[derive(Debug)]
pub struct FixedClock {
    fixed_step: Duration,
    max_frame_delta: Duration,
    accumulator: Duration,
    last_tick: Option<Instant>,
}

impl FixedClock {
    /// Clock with a fixed step of `1/fps` seconds.
    pub fn with_fps(fps: u32) -> FixedClock {
        let fps = fps.max(1) as f32;
        FixedClock::with_step(Duration::from_secs_f32(1.0 / fps))
    }

    pub fn with_step(step: Duration) -> FixedClock {
        FixedClock {
            fixed_step: step,
            max_frame_delta: Duration::from_millis(250),
            accumulator: Duration::ZERO,
            last_tick: None,
        }
    }

    /// Advance the clock; returns the number of fixed steps to simulate.
    pub fn tick(&mut self, now: Instant) -> u32 {
        let Some(last) = self.last_tick else {
            self.last_tick = Some(now);
            return 0;
        };
        let mut frame_delta = now.saturating_duration_since(last);
        if frame_delta > self.max_frame_delta {
            frame_delta = self.max_frame_delta;
        }
        self.last_tick = Some(now);
        self.accumulator += frame_delta;

        let step = self.fixed_step;
        let mut steps = 0u32;
        while self.accumulator >= step {
            self.accumulator -= step;
            steps += 1;
        }
        steps
    }

    /// Progress `0.0..1.0` between the last simulated step and the next,
    /// for render interpolation.
    pub fn alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / self.fixed_step.as_secs_f32()
    }

    pub fn fixed_step_secs(&self) -> f32 {
        self.fixed_step.as_secs_f32()
    }
}

/// Measures elapsed wall time; used for per-phase stats.
#[derive(Debug)]
pub struct Stopwatch {
    start: Instant,
}

impl Stopwatch {
    pub fn start_now() -> Stopwatch {
        Stopwatch {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn elapsed_secs(&self) -> f32 {
        self.elapsed().as_secs_f32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Anchor instant offset far in the past so tests build a stable timeline.
    fn base_instant() -> Instant {
        Instant::now()
            .checked_sub(Duration::from_millis(1_000_000))
            .unwrap()
    }

    /// `ms` milliseconds after `base` — deltas stay exact within a test.
    fn at(base: Instant, ms: u64) -> Instant {
        base.checked_add(Duration::from_millis(ms)).unwrap()
    }

    #[test]
    fn first_tick_is_zero_steps() {
        let base = base_instant();
        let mut clock = FixedClock::with_fps(60);
        assert_eq!(clock.tick(at(base, 0)), 0);
    }

    #[test]
    fn accumulates_into_steps() {
        let base = base_instant();
        let step = Duration::from_millis(10);
        let mut clock = FixedClock::with_step(step);
        assert_eq!(clock.tick(at(base, 0)), 0);
        // 25ms after the first tick: two full 10ms steps + 5ms carry.
        assert_eq!(clock.tick(at(base, 25)), 2);
        assert!(monster_math::approx_eq(clock.alpha(), 0.5, 1e-5));
        // 5ms more: carry reaches 10ms -> exactly one more step.
        assert_eq!(clock.tick(at(base, 30)), 1);
        assert!(monster_math::approx_eq(clock.alpha(), 0.0, 1e-5));
    }

    #[test]
    fn max_frame_delta_clamps_spiral() {
        let base = base_instant();
        let mut clock = FixedClock::with_step(Duration::from_millis(10));
        clock.max_frame_delta = Duration::from_millis(100);
        clock.tick(at(base, 0));
        // 10 second stall -> clamped to 100ms -> 10 steps, not 1000.
        assert_eq!(clock.tick(at(base, 10_000)), 10);
    }

    #[test]
    fn stopwatch_measures() {
        let sw = Stopwatch::start_now();
        std::thread::sleep(Duration::from_millis(15));
        assert!(sw.elapsed_secs() > 0.0);
        assert!(sw.elapsed() >= Duration::from_millis(15));
    }
}
