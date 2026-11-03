//! Coverage-guided plateau detection for MCTS rollout stagnation.
//! Tracks test execution samples and detects plateau conditions when coverage gains stagnate.

/// Point-in-time measurement of test suite coverage during a rollout.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::default::Default,
)]
pub struct CoverageSample {
    pub rollout: ::core::primitive::usize,
    pub total_tests: ::core::primitive::usize,
    pub passed_tests: ::core::primitive::usize,
    pub uncovered_count: ::core::primitive::usize,
}

/// Coverage-guided plateau detector tracking recent rollout samples.
///
/// Ported from MAgHARCM plateau detection architecture. Stagnation occurs
/// when the last K evaluation pairs all demonstrate insufficient progress
/// in both passed test gains and uncovered count reductions.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct PlateauDetector {
    samples: ::std::vec::Vec<CoverageSample>,
    window: ::core::primitive::usize,
    min_passed_gain: ::core::primitive::usize,
    min_uncovered_gain: ::core::primitive::usize,
}

/// Default consecutive stagnant pairs required to trigger plateau.
pub const DEFAULT_PLATEAU_WINDOW: ::core::primitive::usize = 5;
/// Default minimum passed test increment to count as progress.
pub const DEFAULT_MIN_PASSED_GAIN: ::core::primitive::usize = 2;
/// Default minimum uncovered count reduction to count as progress.
pub const DEFAULT_MIN_UNCOVERED_GAIN: ::core::primitive::usize = 1;

impl self::PlateauDetector {
    /// Create a plateau detector with standard defaults.
    pub fn new() -> Self {
        Self {
            samples: ::std::vec::Vec::new(),
            window: self::DEFAULT_PLATEAU_WINDOW,
            min_passed_gain: self::DEFAULT_MIN_PASSED_GAIN,
            min_uncovered_gain: self::DEFAULT_MIN_UNCOVERED_GAIN,
        }
    }

    /// Configure the evaluation window size.
    pub fn with_window(mut self, window: ::core::primitive::usize) -> Self {
        self.window = window;
        self
    }

    /// Configure the minimum passed tests increment.
    pub fn with_min_passed_gain(mut self, min_passed_gain: ::core::primitive::usize) -> Self {
        self.min_passed_gain = min_passed_gain;
        self
    }

    /// Configure the minimum uncovered tests reduction.
    pub fn with_min_uncovered_gain(
        mut self,
        min_uncovered_gain: ::core::primitive::usize,
    ) -> Self {
        self.min_uncovered_gain = min_uncovered_gain;
        self
    }

    /// Return recorded coverage samples.
    pub fn samples(&self) -> &[CoverageSample] {
        &self.samples
    }

    /// Append a coverage sample and return whether a plateau is currently detected.
    pub fn record(&mut self, sample: self::CoverageSample) -> ::core::primitive::bool {
        self.samples.push(sample);
        self.is_plateau()
    }

    /// Return a human-readable one-line summary of plateau detector status.
    pub fn summary(&self) -> ::std::string::String {
        if self.samples.is_empty() {
            return ::std::string::String::from("plateau detector: no samples");
        }
        let last_sample = match self.samples.last() {
            ::core::option::Option::Some(sample) => sample,
            ::core::option::Option::None => {
                return ::std::string::String::from("plateau detector: no samples");
            }
        };
        let status = match self.is_plateau() {
            true => "PLATEAU DETECTED",
            false => "progressing",
        };
        ::std::format!(
            "plateau detector: {} samples, last passed={}, uncovered={}, status: {}",
            self.samples.len(),
            last_sample.passed_tests,
            last_sample.uncovered_count,
            status,
        )
    }

    /// Check if the recent coverage samples exhibit plateau stagnation.
    pub fn is_plateau(&self) -> ::core::primitive::bool {
        if self.window == 0 || self.samples.len() < self.window + 1 {
            return false;
        }
        let start_index = self.samples.len() - self.window - 1;
        let mut stagnant_pairs: ::core::primitive::usize = 0;
        for pair_index in start_index..(self.samples.len() - 1) {
            let previous_sample = &self.samples[pair_index];
            let current_sample = &self.samples[pair_index + 1];
            let passed_gain = (current_sample.passed_tests as ::core::primitive::isize)
                - (previous_sample.passed_tests as ::core::primitive::isize);
            let uncovered_gain = (previous_sample.uncovered_count as ::core::primitive::isize)
                - (current_sample.uncovered_count as ::core::primitive::isize);
            if passed_gain < (self.min_passed_gain as ::core::primitive::isize)
                && uncovered_gain < (self.min_uncovered_gain as ::core::primitive::isize)
            {
                stagnant_pairs += 1;
            }
        }
        stagnant_pairs >= self.window
    }
}

impl ::core::default::Default for self::PlateauDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_and_summary() {
        let detector = self::PlateauDetector::new();
        ::std::assert!(!detector.is_plateau());
        ::std::assert_eq!(detector.summary(), "plateau detector: no samples");
    }

    #[test]
    fn test_insufficient_samples_does_not_plateau() {
        let mut detector = self::PlateauDetector::new();
        let sample_first = self::CoverageSample {
            rollout: 1,
            total_tests: 20,
            passed_tests: 10,
            uncovered_count: 10,
        };
        let sample_second = self::CoverageSample {
            rollout: 2,
            total_tests: 20,
            passed_tests: 10,
            uncovered_count: 10,
        };
        ::std::assert!(!detector.record(sample_first));
        ::std::assert!(!detector.record(sample_second));
        ::std::assert!(!detector.is_plateau());
    }

    #[test]
    fn test_plateau_detection_after_consecutive_stagnant_pairs() {
        let mut detector = self::PlateauDetector::new();
        for rollout in 1..=5 {
            detector.record(self::CoverageSample {
                rollout,
                total_tests: 20,
                passed_tests: 5,
                uncovered_count: 15,
            });
        }
        let reached_plateau = detector.record(self::CoverageSample {
            rollout: 6,
            total_tests: 20,
            passed_tests: 6,
            uncovered_count: 15,
        });
        ::std::assert!(reached_plateau);
        ::std::assert!(detector.is_plateau());
        let summary_text = detector.summary();
        ::std::assert!(summary_text.contains("PLATEAU DETECTED"));
    }

    #[test]
    fn test_progress_in_passed_tests_clears_plateau() {
        let mut detector = self::PlateauDetector::new().with_window(2);
        detector.record(self::CoverageSample {
            rollout: 1,
            total_tests: 20,
            passed_tests: 5,
            uncovered_count: 15,
        });
        detector.record(self::CoverageSample {
            rollout: 2,
            total_tests: 20,
            passed_tests: 6,
            uncovered_count: 15,
        });
        detector.record(self::CoverageSample {
            rollout: 3,
            total_tests: 20,
            passed_tests: 12,
            uncovered_count: 15,
        });
        ::std::assert!(!detector.is_plateau());
    }

    #[test]
    fn test_progress_in_uncovered_reduction_clears_plateau() {
        let mut detector = self::PlateauDetector::new().with_window(2);
        detector.record(self::CoverageSample {
            rollout: 1,
            total_tests: 20,
            passed_tests: 5,
            uncovered_count: 15,
        });
        detector.record(self::CoverageSample {
            rollout: 2,
            total_tests: 20,
            passed_tests: 5,
            uncovered_count: 14,
        });
        detector.record(self::CoverageSample {
            rollout: 3,
            total_tests: 20,
            passed_tests: 5,
            uncovered_count: 13,
        });
        ::std::assert!(!detector.is_plateau());
    }

    #[test]
    fn test_custom_window_configuration() {
        let detector = self::PlateauDetector::new().with_window(3);
        ::std::assert_eq!(detector.window, 3);
    }
}
