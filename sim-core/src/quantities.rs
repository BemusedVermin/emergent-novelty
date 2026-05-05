//! Validated numeric newtypes for constraints the math doc imposes pointwise.
//!
//! The math doc prescribes range constraints on a handful of scalar
//! quantities — mass m : 𝒳 → ℝ_+ (§3.1), Langton's λ ∈ [0, 1] (§3.3),
//! probabilities, etc. Rust's type system can't express semantic laws like
//! idempotence or conservation, but it can enforce *pointwise* constraints
//! at construction time. This module exposes the two wrappers that pay for
//! their own noise: [`NonNeg`] and [`Probability`].
//!
//! Both reject NaN as well as out-of-range values, which is the more
//! load-bearing guarantee in practice — `f64` arithmetic propagates NaN
//! silently, and a stray NaN mass or NaN probability will eat through any
//! downstream comparison or summation without complaint.
//!
//! [`ProbabilityMeasure::density`](crate::measure::ProbabilityMeasure::density)
//! and [`Kernel::density`](crate::measure::Kernel::density) both return
//! [`Option<NonNeg>`]: a Radon–Nikodym density is non-negative and finite
//! wherever it's defined, and pushing that constraint into the type
//! catches NaN / negative-density bugs at the implementation site rather
//! than during a downstream comparison. Densities against Lebesgue can
//! exceed 1 — [`Probability`] would be too tight — but [`NonNeg`] is the
//! right codomain.

/// ℝ_+ — non-negative finite reals.
///
/// Constructed via [`NonNeg::new`], which rejects NaN and negative values.
/// The math doc's "supports a mass functional" condition (§3.1) requires
/// `m : 𝒳 → ℝ_+`; [`MassFunctional::mass`](crate::substrate::MassFunctional::mass)
/// returns `NonNeg` so that condition is checked at construction rather than
/// merely documented.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct NonNeg(f64);

impl NonNeg {
    /// Wrap `x` if it is finite and `≥ 0`. Returns `None` for NaN, negative
    /// values, or ±∞.
    pub fn new(x: f64) -> Option<Self> {
        if x.is_finite() && x >= 0.0 {
            Some(NonNeg(x))
        } else {
            None
        }
    }

    pub const ZERO: Self = NonNeg(0.0);

    /// Project back to `f64`.
    pub fn get(self) -> f64 {
        self.0
    }
}

impl From<NonNeg> for f64 {
    fn from(x: NonNeg) -> f64 {
        x.0
    }
}

/// [0, 1] — a probability.
///
/// Constructed via [`Probability::new`], which rejects NaN and values outside
/// [0, 1]. Used by [`LangtonLambda`](crate::regime::LangtonLambda) to make
/// the math doc's "0.0 ≤ λ ≤ 1.0" convention type-checked at construction.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Probability(f64);

impl Probability {
    /// Wrap `p` if it is finite and lies in `[0, 1]`. Returns `None`
    /// otherwise.
    pub fn new(p: f64) -> Option<Self> {
        if p.is_finite() && (0.0..=1.0).contains(&p) {
            Some(Probability(p))
        } else {
            None
        }
    }

    pub const ZERO: Self = Probability(0.0);
    pub const ONE: Self = Probability(1.0);

    /// Project back to `f64`.
    pub fn get(self) -> f64 {
        self.0
    }
}

impl From<Probability> for f64 {
    fn from(p: Probability) -> f64 {
        p.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonneg_rejects_nan_and_negatives() {
        assert!(NonNeg::new(f64::NAN).is_none());
        assert!(NonNeg::new(-0.001).is_none());
        assert!(NonNeg::new(f64::INFINITY).is_none());
        assert!(NonNeg::new(0.0).is_some());
        assert!(NonNeg::new(1e9).is_some());
    }

    #[test]
    fn probability_rejects_out_of_range_and_nan() {
        assert!(Probability::new(f64::NAN).is_none());
        assert!(Probability::new(-0.001).is_none());
        assert!(Probability::new(1.0001).is_none());
        assert!(Probability::new(0.0).is_some());
        assert!(Probability::new(1.0).is_some());
        assert!(Probability::new(0.5).is_some());
    }
}
