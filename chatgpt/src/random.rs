//! Deterministic random testing and benchmarking support.

use std::time::{Duration, Instant};

use crate::algorithms::{solve, Algorithm};
use crate::{validate_sort_plan, MachineError};

/// Small, reproducible SplitMix64 generator.
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Creates a generator from an explicit seed.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Returns the next uniformly mixed 64-bit value.
    #[must_use]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    /// Produces a uniformly shuffled permutation of `1..=n`.
    #[must_use]
    pub fn permutation(&mut self, n: usize) -> Vec<usize> {
        let mut values: Vec<_> = (1..=n).collect();
        // Rejection sampling avoids modulo bias.
        for upper in (1..values.len()).rev() {
            let range = (upper + 1) as u64;
            let zone = u64::MAX - u64::MAX % range;
            let index = loop {
                let value = self.next_u64();
                if value < zone {
                    break (value % range) as usize;
                }
            };
            values.swap(upper, index);
        }
        values
    }
}

/// Online sample statistics for move counts.
#[derive(Clone, Debug, Default)]
pub struct SampleStats {
    /// Number of observations.
    pub count: usize,
    /// Arithmetic sample mean.
    pub mean: f64,
    m2: f64,
    /// Smallest observation.
    pub minimum: usize,
    /// Largest observation.
    pub maximum: usize,
}

impl SampleStats {
    /// Adds one observation using Welford's stable recurrence.
    pub fn add(&mut self, value: usize) {
        if self.count == 0 {
            self.minimum = value;
            self.maximum = value;
        } else {
            self.minimum = self.minimum.min(value);
            self.maximum = self.maximum.max(value);
        }
        self.count += 1;
        let delta = value as f64 - self.mean;
        self.mean += delta / self.count as f64;
        self.m2 += delta * (value as f64 - self.mean);
    }

    /// Unbiased sample standard deviation, or zero for fewer than two samples.
    #[must_use]
    pub fn standard_deviation(&self) -> f64 {
        if self.count < 2 {
            0.0
        } else {
            (self.m2 / (self.count - 1) as f64).sqrt()
        }
    }

    /// Standard error of the sample mean.
    #[must_use]
    pub fn standard_error(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.standard_deviation() / (self.count as f64).sqrt()
        }
    }
}

/// Result of a random correctness campaign.
#[derive(Clone, Debug, Default)]
pub struct RandomTestReport {
    /// Number of permutations tested.
    pub samples: usize,
    /// Number of algorithm-plan pairs replayed.
    pub plans_checked: usize,
    /// Total campaign duration.
    pub elapsed: Duration,
}

/// Generates permutations, solves with every requested algorithm, and replays
/// every returned plan through a fresh simulator.
pub fn random_test(
    n: usize,
    samples: usize,
    seed: u64,
    algorithms: &[Algorithm],
) -> Result<RandomTestReport, MachineError> {
    let began = Instant::now();
    let mut rng = Rng::new(seed);
    let mut checked = 0;
    for _ in 0..samples {
        let deck = rng.permutation(n);
        for &algorithm in algorithms {
            let result = solve(algorithm, &deck)?;
            validate_sort_plan(&deck, &result.plan)?;
            checked += 1;
        }
    }
    Ok(RandomTestReport {
        samples,
        plans_checked: checked,
        elapsed: began.elapsed(),
    })
}

/// Measurements for one algorithm in a benchmark campaign.
#[derive(Clone, Debug)]
pub struct BenchmarkResult {
    /// Algorithm measured.
    pub algorithm: Algorithm,
    /// Primitive move-count distribution.
    pub moves: SampleStats,
    /// Total solver and replay time.
    pub elapsed: Duration,
}

/// Benchmarks algorithms on the same deterministic set of random inputs.
pub fn random_benchmark(
    n: usize,
    samples: usize,
    seed: u64,
    algorithms: &[Algorithm],
) -> Result<Vec<BenchmarkResult>, MachineError> {
    let mut rng = Rng::new(seed);
    let decks: Vec<_> = (0..samples).map(|_| rng.permutation(n)).collect();
    let mut reports = Vec::with_capacity(algorithms.len());
    for &algorithm in algorithms {
        let began = Instant::now();
        let mut moves = SampleStats::default();
        for deck in &decks {
            let result = solve(algorithm, deck)?;
            validate_sort_plan(deck, &result.plan)?;
            moves.add(result.cost());
        }
        reports.push(BenchmarkResult {
            algorithm,
            moves,
            elapsed: began.elapsed(),
        });
    }
    Ok(reports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_permutations_are_reproducible() {
        let mut first = Rng::new(42);
        let mut second = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(first.permutation(52), second.permutation(52));
        }
    }

    #[test]
    fn random_framework_replays_plans() {
        let report = random_test(20, 50, 7, &Algorithm::ALL).unwrap();
        assert_eq!(report.samples, 50);
        assert_eq!(report.plans_checked, 800);
    }

    #[test]
    fn documented_random_means_are_within_sampling_tolerance() {
        let algorithms = [
            Algorithm::Selection,
            Algorithm::AdaptiveSelection,
            Algorithm::LookaheadSelection,
            Algorithm::TwoKPartitionLookaheadSelection(1),
            Algorithm::TwoKPartitionLookaheadSelection(2),
            Algorithm::TwoKPartitionLookaheadSelection(3),
            Algorithm::TwoKPartitionLookaheadSelection(4),
            Algorithm::BinaryPresortAdaptiveSelection,
            Algorithm::Natural,
        ];
        let reports = random_benchmark(52, 2_000, 0x5eed, &algorithms).unwrap();
        // The handoff's binary-presort value (504) omits the expected search
        // from the exposed top of each freshly partitioned bucket to its
        // maximum. The literal specified pseudocode pays another
        // `(a-1) + (b-1)` legal moves in expectation: 554 for n=52.
        let expected = [
            1_854.960_768_916_4,
            952.980_384_458_2,
            810.586,
            457.627,
            385.342,
            394.401,
            401.068,
            554.0,
            520.195_560_629_6,
        ];
        for (report, expected_mean) in reports.iter().zip(expected) {
            let tolerance = 5.0 * report.moves.standard_error().max(1.0);
            assert!(
                (report.moves.mean - expected_mean).abs() <= tolerance,
                "{} mean {}, expected {expected_mean}, tolerance {tolerance}",
                report.algorithm.name(),
                report.moves.mean
            );
        }
    }
}
