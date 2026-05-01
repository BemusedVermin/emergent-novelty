//! Substrate-agnostic emergent-systems scaffold.
//!
//! One-to-one Rust translation of
//! `docs/notes/emergent_systems_mathematical_treatment.md`.
//!
//! ## The six-tuple
//!
//! A simulator instance is
//!
//! ```text
//!   M = (𝒳, Φ, V, F, 𝒯, 𝒪)
//! ```
//!
//! over a measurable space 𝒳, with five interchangeable slots and an
//! observer-evaluator that lives outside the dynamics:
//!
//! | Math | Trait |
//! |---|---|
//! | (𝒳, Φ, μ_0) substrate | [`Substrate`] |
//! | V : 𝒳 → P(𝒳) variation | [`Variation`] |
//! | F : 𝒳 × P(𝒳) → {0,1} viability | [`Viability`] |
//! | 𝒯 = (G, K) topology | [`Topology`] (and [`StigmergicField`]) |
//! | 𝒪 : trajectories → ℝ observer | [`Observer`] |
//!
//! ## The population update
//!
//! ```text
//!   μ_{t+1} = 𝒮_F ∘ V ∘ Φ_𝒯 (μ_t)
//! ```
//!
//! is implemented by [`SimulatorInstance::step`] as a literal three-stage
//! pipeline. The observer is reachable via [`SimulatorInstance::observer`]
//! but never invoked by `step` — this is the formal expression of
//! non-prescriptive design (§2 of the math doc).
//!
//! ## Foundations
//!
//! Every operator above is typed in terms of [`MeasurableSpace`],
//! [`Map`], [`Kernel`], and [`ProbabilityMeasure`] from the [`measure`]
//! module — see there for the measure-theoretic foundations.

pub mod closure;
pub mod laws;
pub mod measure;
pub mod niche;
pub mod observer;
pub mod population;
pub mod quantities;
pub mod regime;
pub mod substrate;
pub mod topology;
pub mod trajectory;
pub mod variation;
pub mod viability;

pub use crate::closure::{
    AutopoieticClosure, ClosureOperator, MarkovBlanketClosure, MinimalCriterionClosure, RafClosure,
};
pub use crate::measure::{Dirac, Kernel, Map, MeasurableSpace, Point, ProbabilityMeasure};
pub use crate::niche::NicheConstruction;
pub use crate::observer::{
    BehaviorMap, FmEmbeddingObserver, KnnNoveltyObserver, Observer, OmegaResidenceObserver,
};
pub use crate::population::Population;
pub use crate::quantities::{NonNeg, Probability};
pub use crate::regime::{LangtonLambda, WolframClass};
pub use crate::substrate::{MassFunctional, Substrate};
pub use crate::topology::{StigmergicField, Topology};
pub use crate::trajectory::Trajectory;
pub use crate::variation::Variation;
pub use crate::viability::Viability;

use rand_core::Rng;

/// A simulator instance — the six-tuple M = (𝒳, Φ, V, F, 𝒯, 𝒪) of §2 of
/// the math doc.
///
/// The `where` bounds enforce that every slot speak the same substrate
/// measurable space. Wiring a slot whose `Space` differs from the
/// substrate's is a compile error.
///
/// The `observer` field is reachable but is **not** invoked by
/// [`step`](Self::step). The simulator does not consult its own evaluator
/// while running — that is what makes the design non-prescriptive.
pub struct SimulatorInstance<S, V, F, T, O>
where
    S: Substrate,
    V: Variation<Space = S::Space>,
    F: Viability<Space = S::Space>,
    T: Topology<Space = S::Space>,
    O: Observer<Space = S::Space>,
{
    pub substrate: S,
    pub variation: V,
    pub viability: F,
    pub topology: T,
    pub observer: O,
}

impl<S, V, F, T, O> SimulatorInstance<S, V, F, T, O>
where
    S: Substrate,
    V: Variation<Space = S::Space>,
    F: Viability<Space = S::Space>,
    T: Topology<Space = S::Space>,
    O: Observer<Space = S::Space>,
{
    /// One population update: μ_{t+1} = 𝒮_F ∘ V ∘ Φ_𝒯 (μ_t). §2.
    ///
    /// Three sequential stages, in math-doc order:
    ///
    /// 1. **Φ_𝒯** — each entity x_i steps under the substrate dynamics,
    ///    consulting the topology for its neighbour set.
    /// 2. **V** — each post-step entity is perturbed by the variation
    ///    operator, conditioned on `archive`.
    /// 3. **𝒮_F** — the resulting candidate population is filtered by the
    ///    viability predicate. *Cull-only*: an entity that fails F is
    ///    removed; the population may shrink, and extinction is a
    ///    possible outcome. The "possibly producing offspring" half of
    ///    the math doc's 𝒮_F description (§2) is left to higher-level
    ///    orchestration so that the scaffold does not commit to a
    ///    replication policy.
    pub fn step<R: Rng + ?Sized>(
        &self,
        pop: &mut Population<S::Space>,
        archive: &V::Archive,
        rng: &mut R,
    ) {
        // Φ_𝒯 : substrate dynamics with topology coupling
        let mut next: Vec<Point<S::Space>> = Vec::with_capacity(pop.len());
        for i in 0..pop.len() {
            let neighbors = self.topology.neighbors(i, pop);
            let stepped = self.substrate.step(&pop.members[i], &neighbors, rng);
            next.push(stepped);
        }

        // V : variation
        for point in &mut next {
            let perturbed = self.variation.perturb(point, archive, rng);
            *point = perturbed;
        }

        // 𝒮_F : selection (cull-only)
        let candidate = Population::from_members(next);
        let survivors: Vec<Point<S::Space>> = candidate
            .members
            .iter()
            .filter(|x| self.viability.survives(*x, &candidate))
            .cloned()
            .collect();

        pop.members = survivors;
    }
}
