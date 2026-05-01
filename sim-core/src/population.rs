//! Populations: empirical approximation of μ ∈ P(Ω) where Ω = 𝒳^N.
//!
//! Mirrors the notation `μ_t ∈ P(Ω)` in §2 of the math doc. The full
//! population distribution μ is intractable for general state spaces; we
//! approximate it by the empirical measure μ̂ = (1/N) Σ_i δ_{x_i}, which is
//! parameterised by the multiset of population members. [`Population`] is
//! the newtype that documents this empirical approximation.

use crate::measure::{MeasurableSpace, Point};
use crate::quantities::NonNeg;
use crate::substrate::{MassFunctional, Substrate};
use rand_core::Rng;

/// A population of N entities, each a `Point` of the substrate's
/// measurable space S.
///
/// Empirically represents the measure μ̂ = (1/N) Σ_i δ_{x_i} ∈ P(Ω) with
/// Ω = S^N. Population size N is variable across simulator steps because
/// the selection operator 𝒮_F may cull or replicate entities.
pub struct Population<S: MeasurableSpace> {
    pub members: Vec<Point<S>>,
}

impl<S: MeasurableSpace> Population<S> {
    /// Empty population.
    pub fn new() -> Self {
        Self {
            members: Vec::new(),
        }
    }

    /// Population from a list of members.
    pub fn from_members(members: Vec<Point<S>>) -> Self {
        Self { members }
    }

    /// Population of `n` entities, each independently drawn from the
    /// substrate's initial distribution μ_0.
    ///
    /// Convenience constructor for the common opening move "give me N
    /// fresh entities to start a run with"; equivalent to calling
    /// [`Substrate::initial`] in a loop.
    pub fn sampled_from<Sub, R>(substrate: &Sub, n: usize, rng: &mut R) -> Self
    where
        Sub: Substrate<Space = S>,
        R: Rng + ?Sized,
    {
        let members = (0..n).map(|_| substrate.initial(rng)).collect();
        Self { members }
    }

    /// N — the current cardinality of the population.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Append a new entity x_{N+1}.
    pub fn push(&mut self, x: Point<S>) {
        self.members.push(x);
    }

    /// Cull entities for which `keep(x)` returns false.
    ///
    /// Used by the selection operator 𝒮_F to apply a viability predicate.
    pub fn retain<P: FnMut(&Point<S>) -> bool>(&mut self, keep: P) {
        self.members.retain(keep);
    }

    /// Iterate over members in storage order.
    pub fn iter(&self) -> std::slice::Iter<'_, Point<S>> {
        self.members.iter()
    }

    /// Iterate over members with mutable access.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Point<S>> {
        self.members.iter_mut()
    }

    /// Total mass `Σᵢ m(xᵢ)` under a [`MassFunctional`].
    ///
    /// The population-level aggregate that Flow-Lenia and other
    /// mass-conserving substrates preserve under Φ; the per-entity
    /// `m(xᵢ)` is rarely the interesting invariant on its own. Pair with
    /// [`laws::substrate::total_mass_conservation_under_phi`](crate::laws::substrate::total_mass_conservation_under_phi)
    /// to check the conservation law in tests.
    ///
    /// # Panics
    ///
    /// If the sum overflows to ±∞ (only reachable for adversarial mass
    /// functionals or astronomical N). Per-entity masses are already
    /// guaranteed finite by the [`NonNeg`] return type of
    /// [`MassFunctional::mass`].
    pub fn total_mass<MF>(&self, mf: &MF) -> NonNeg
    where
        MF: MassFunctional<Space = S>,
    {
        let total: f64 = self.members.iter().map(|x| mf.mass(x).get()).sum();
        NonNeg::new(total).expect("total mass overflowed to ±∞")
    }
}

impl<S: MeasurableSpace> Default for Population<S> {
    fn default() -> Self {
        Self::new()
    }
}
