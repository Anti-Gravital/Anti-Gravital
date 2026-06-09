//! Bounded dynamic-pool scaling decisions.
//!
//! Dynamic pools scale within explicit bounds based on operational pressure — never
//! unbounded. The decision itself is a pure function of the configuration and the
//! observed queue depth, so it is deterministic and unit-testable; the controller that
//! applies it owns the cooldown and the spawning.

use std::time::Duration;

/// Configuration for a bounded dynamic worker pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaleConfig {
    /// Minimum number of workers (the pool never drops below this).
    pub min_workers: usize,
    /// Maximum number of workers (the pool never grows above this).
    pub max_workers: usize,
    /// Target number of ready jobs per worker; drives the scale-up target.
    pub queue_depth_per_worker: u64,
    /// Minimum time between scaling adjustments.
    pub cooldown: Duration,
}

impl Default for ScaleConfig {
    fn default() -> Self {
        Self {
            min_workers: 1,
            max_workers: 8,
            queue_depth_per_worker: 100,
            cooldown: Duration::from_secs(30),
        }
    }
}

impl ScaleConfig {
    /// Computes the desired worker count for the observed `ready_depth`, clamped to
    /// `[min_workers, max_workers]`. A higher backlog asks for more workers; an empty
    /// queue settles back to `min_workers`.
    pub fn desired_workers(&self, ready_depth: u64) -> usize {
        let per = self.queue_depth_per_worker.max(1);
        // Ceiling division so any backlog asks for at least one worker.
        let target = ready_depth.div_ceil(per) as usize;
        target.clamp(self.min_workers, self.max_workers.max(self.min_workers))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ScaleConfig {
        ScaleConfig {
            min_workers: 1,
            max_workers: 8,
            queue_depth_per_worker: 100,
            cooldown: Duration::from_secs(30),
        }
    }

    #[test]
    fn empty_queue_settles_to_min() {
        assert_eq!(cfg().desired_workers(0), 1);
    }

    #[test]
    fn backlog_scales_up_with_ceiling() {
        assert_eq!(cfg().desired_workers(1), 1);
        assert_eq!(cfg().desired_workers(100), 1);
        assert_eq!(cfg().desired_workers(101), 2);
        assert_eq!(cfg().desired_workers(350), 4);
    }

    #[test]
    fn never_exceeds_max() {
        assert_eq!(cfg().desired_workers(10_000), 8);
    }

    #[test]
    fn respects_min_floor() {
        let c = ScaleConfig {
            min_workers: 3,
            ..cfg()
        };
        assert_eq!(c.desired_workers(0), 3);
        assert_eq!(c.desired_workers(50), 3);
    }
}
