//! Gated hierarchical reward calculation.
//! Assigns zero reward to uncompilable code and scales with parity and hardening.

/// Compute gated scalar reward for candidate target code.
/// When compilation fails, reward is strictly zero.
pub fn compute_gated_reward(
    weights: &crate::types::RewardWeights,
    compiles: ::core::primitive::bool,
    char_pass_rate: ::core::primitive::f64,
    harden_score: ::core::primitive::f64,
) -> ::core::primitive::f64 {
    if !compiles {
        return 0.0;
    }

    let clamped_pass = self::clamp_unit(char_pass_rate);
    let clamped_harden = self::clamp_unit(harden_score);

    let w_comp = weights.compile;
    let w_char = weights.parity;
    let w_hard = weights.harden;

    let test_part = w_char * clamped_pass;
    let hard_part = w_hard * clamped_harden;
    let sum_score = test_part + hard_part;
    let total = w_comp + sum_score;

    self::clamp_unit(total)
}

fn clamp_unit(val: ::core::primitive::f64) -> ::core::primitive::f64 {
    val.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uncompilable_gives_zero() {
        let weights = crate::types::RewardWeights::default();
        let reward = self::compute_gated_reward(&weights, false, 1.0, 1.0);
        ::std::assert_eq!(reward, 0.0);
    }

    #[test]
    fn test_full_parity_gives_unit() {
        let weights = crate::types::RewardWeights::default();
        let reward = self::compute_gated_reward(&weights, true, 1.0, 1.0);
        ::std::assert!((reward - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_partial_parity_calculation() {
        let weights = crate::types::RewardWeights::default();
        let reward = self::compute_gated_reward(&weights, true, 0.5, 0.0);
        let expected = 0.20 + (0.60 * 0.5);
        ::std::assert!((reward - expected).abs() < 1e-6);
    }
}
