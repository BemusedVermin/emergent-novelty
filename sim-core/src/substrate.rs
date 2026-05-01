//! The substrate slot S = (𝒳, Φ, 𝒩, μ_0).
//!
//! Mirrors §3 of the math doc.

use crate::measure::{MeasurableSpace, Point};
use rand_core::Rng;

/// A substrate S = (𝒳, Φ, 𝒩, μ_0). §3.1.
///
/// - `Space` is the measurable space 𝒳 supporting single-entity states.
/// - `initial` samples from the initial distribution μ_0.
/// - `step` applies Φ at point x given a slice of neighbour states; the
///   neighbour selection rule lives in the [`Topology`](crate::topology::Topology)
///   slot, which is why the substrate takes neighbours as an argument
///   rather than reaching into the population itself.
///
/// Φ is presented as a (deterministic-or-stochastic) point→point map; the
/// stochastic case uses the RNG argument, the deterministic case ignores
/// it. This matches the math doc's "Φ : 𝒳 → 𝒳 (deterministic) or
/// Φ : 𝒳 → P(𝒳) (stochastic)" — both factor through this signature, with
/// the deterministic case being the [`Dirac`](crate::measure::Dirac) lift.
pub trait Substrate {
    type Space: MeasurableSpace;

    /// Sample x_0 ~ μ_0.
    fn initial<R: Rng + ?Sized>(&self, rng: &mut R) -> Point<Self::Space>;

    /// Apply Φ at x given the topology-supplied neighbours.
    fn step<R: Rng + ?Sized>(
        &self,
        x: &Point<Self::Space>,
        neighbors: &[&Point<Self::Space>],
        rng: &mut R,
    ) -> Point<Self::Space>;
}

/// Mass functional m : 𝒳 → ℝ_+. §3.1.
///
/// A substrate "supports a mass functional if conservation is to be
/// enforced" — Flow-Lenia is the canonical case where m is mass-conserved
/// by the dynamics. The trait is separated from `Substrate` so that
/// substrates with no native mass (Avida tape, LLM prompt) simply do not
/// implement it.
///
/// The return type is `f64`; non-negativity is a documented expectation,
/// not a type-level constraint.
pub trait MassFunctional {
    type Space: MeasurableSpace;

    fn mass(&self, x: &Point<Self::Space>) -> f64;
}
