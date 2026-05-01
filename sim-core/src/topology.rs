//! The interaction-topology slot T = (G, K) and its stigmergic variant.
//!
//! Mirrors §6 of the math doc.
//!
//! Topology is a separate slot from substrate (§6.3). The same substrate
//! supports multiple topologies (a swarm of LLM-agent particles can use
//! Euclidean range, social-network range, or a stigmergic field), and
//! topology determines who acts as whose viability filter under MCC / POET.

use crate::measure::{MeasurableSpace, Point};
use crate::population::Population;

/// Direct interaction topology — a graph G on population indices that says
/// which entities can causally influence which others within one step.
///
/// `neighbors(i, μ)` returns the entities causally adjacent to entity i in
/// the current population μ.
///
/// Lattice topologies (CA), range-based topologies (Boids, Particle Life,
/// Particle Lenia), social graphs, and pairwise-game topologies all fit
/// here. For *indirect* coupling through a shared environmental field,
/// see [`StigmergicField`].
pub trait Topology {
    type Space: MeasurableSpace;

    /// Neighbours of entity i in the population.
    fn neighbors<'a>(
        &self,
        i: usize,
        population: &'a Population<Self::Space>,
    ) -> Vec<&'a Point<Self::Space>>;
}

/// Stigmergic / niche-construction topology. §6.2.
///
/// Agents do not directly couple; they read and write to a shared
/// environmental field e ∈ ℰ:
///
/// ```text
///   e_{t+1} = e_t + Σ_i deposit_i(x_i)            (write + decay)
///   x_i^{(t+1)} = Φ(x_i^{(t)}, read_i(e_t))       (read + step)
/// ```
///
/// This is the formal expression of niche construction (Odling-Smee–
/// Laland–Feldman) and stigmergy (Grassé). For a coupled
/// evolution-and-culture simulator, stigmergic topology is load-bearing:
/// it is how agents alter the selective environment for genetic evolution.
///
/// The trait deliberately does not require any particular relationship
/// between `Field`, `Point`, `Observation`, and `Deposit`. Niche-
/// construction-aware viability filters can read the field; observers can
/// score it.
///
/// **Orchestration is user-driven.**
/// [`SimulatorInstance::step`](crate::SimulatorInstance::step) does *not*
/// drive stigmergic fields — it only consults [`Topology::neighbors`] (the
/// direct-coupling `G` half of the math doc's `T = (G, K)`). The kernel
/// `K` half is exposed as this trait, but the [`read`](Self::read) /
/// [`write`](Self::write) / [`decay`](Self::decay) calls are the
/// downstream runner's responsibility: a stigmergic substrate's main loop
/// calls `read` to gather observations before Φ, `write` to deposit after,
/// and `decay` between rounds. This split keeps `SimulatorInstance` field-
/// agnostic and avoids committing the scaffold to a particular ordering of
/// field updates relative to Φ, V, and 𝒮_F.
pub trait StigmergicField {
    /// The shared environmental field e.
    type Field;
    /// The agent state from which reads/deposits originate.
    type Point;
    /// The local view an agent receives from `read`.
    type Observation;
    /// The increment an agent contributes via `write`.
    type Deposit;

    /// Local read: e × x ↦ obs.
    fn read(&self, field: &Self::Field, x: &Self::Point) -> Self::Observation;

    /// Local write: e ← e + deposit.
    fn write(&self, field: &mut Self::Field, x: &Self::Point, deposit: Self::Deposit);

    /// Global decay step applied between rounds. The minus-sign in
    /// `e_{t+1} = e_t + Σ_i deposit_i − decay` lives here.
    fn decay(&self, field: &mut Self::Field);
}
