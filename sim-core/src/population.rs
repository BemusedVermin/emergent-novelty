//! Populations: empirical approximation of μ ∈ P(Ω) where Ω = 𝒳^N.
//!
//! Mirrors the notation `μ_t ∈ P(Ω)` in §2 of the math doc. The full
//! population distribution μ is intractable for general state spaces; we
//! approximate it by the empirical measure μ̂ = (1/N) Σ_i δ_{x_i}, which is
//! parameterised by the multiset of population members. [`Population`] is
//! the newtype that documents this empirical approximation.

use crate::measure::{MeasurableSpace, Point};

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
}

impl<S: MeasurableSpace> Default for Population<S> {
    fn default() -> Self {
        Self::new()
    }
}
