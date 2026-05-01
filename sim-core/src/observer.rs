//! The observer-evaluator slot 𝒪.
//!
//! Mirrors §7 of the math doc.
//!
//! 𝒪 is *outside* the dynamics: nothing in (Φ, V, F, T) consults 𝒪 during
//! simulation. Keeping 𝒪 outside the population update is the formal
//! expression of non-prescriptive design — the simulator does not consult
//! its own evaluator while running.
//!
//! The unification claim of §7.3 is that the named observer families are
//! all instances of one structure: a divergence-from-prior-history
//! functional 𝒪(T) = 𝔼_t[ D(p_t ‖ q_{<t}) ] for some divergence D and
//! choice of present/past representation. The three named stubs below
//! pick different (D, representation) pairs.

use crate::measure::{MeasurableSpace, Point};
use crate::trajectory::Trajectory;
use std::marker::PhantomData;

/// An observer-evaluator 𝒪 : (trajectories, prior knowledge) → ℝ. §7.1.
///
/// The "prior knowledge" component lives on `&self` (concrete observers
/// hold their own learner state, archive, embedding model, etc.). The
/// trajectory is the only externally supplied input.
///
/// Implementations are pluggable and disagree by design — the math doc's
/// recommendation (§7.4 implementation note) is to run multiple observers
/// in parallel and report each separately rather than averaging them.
pub trait Observer {
    type Space: MeasurableSpace;

    /// Score the trajectory T_t.
    fn score(&self, trajectory: &Trajectory<Self::Space>) -> f64;
}

/// Behaviour characterisation β : 𝒳 → ℬ. §1.
///
/// Maps a substrate point to a descriptor in a behaviour space ℬ.
/// Used by behaviour-archive novelty observers (Lehman–Stanley) and as
/// the goal space for IMGEP-style variation.
///
/// Mathematically a [`Map`](crate::measure::Map) from `Space` to
/// `Behavior`; we name it separately because it appears as a distinct
/// object in the math (denoted β) with a distinct role in observer and
/// variation slots.
pub trait BehaviorMap {
    type Space: MeasurableSpace;
    type Behavior: MeasurableSpace;

    fn behavior(&self, x: &Point<Self::Space>) -> Point<Self::Behavior>;
}

// ─────────────────────────────────────────────────────────────────────────
// §7.2 named observer families — stubs
// ─────────────────────────────────────────────────────────────────────────

/// Lehman–Stanley behaviour-archive kNN novelty observer
/// (§7.2 density-distance family).
///
/// 𝒪_nov(x_t; T_{<t}) = (1/k) Σ_{j ∈ kNN(x_t, T_{<t})} d(β(x_t), β(x_j))
///
/// **Implementation status.** Stub. Requires a concrete distance d on
/// behaviour space, a behaviour map β, and a kNN data structure over the
/// archive — all deferred.
pub struct KnnNoveltyObserver<S: MeasurableSpace> {
    /// k in the kNN query.
    pub k: usize,
    _space: PhantomData<S>,
}

impl<S: MeasurableSpace> KnnNoveltyObserver<S> {
    pub fn new(k: usize) -> Self {
        Self {
            k,
            _space: PhantomData,
        }
    }
}

impl<S: MeasurableSpace> Observer for KnnNoveltyObserver<S> {
    type Space = S;

    fn score(&self, _trajectory: &Trajectory<S>) -> f64 {
        unimplemented!(
            "KnnNoveltyObserver::score — Lehman–Stanley novelty; \
             not yet implemented"
        )
    }
}

/// Foundation-model embedding-distance observer
/// (§7.2 ASAL family; Kumar et al. 2024).
///
/// 𝒪_ASAL(T) = 𝔼_t[ min_{s<t} ‖ E_φ(x_t) − E_φ(x_s) ‖ ]
///
/// Replaces β with an FM encoder E_φ (e.g., CLIP).
///
/// **Implementation status.** Stub. Requires an embedding model and an
/// embedding-archive distance — deferred.
pub struct FmEmbeddingObserver<S: MeasurableSpace> {
    _space: PhantomData<S>,
}

impl<S: MeasurableSpace> FmEmbeddingObserver<S> {
    pub fn new() -> Self {
        Self {
            _space: PhantomData,
        }
    }
}

impl<S: MeasurableSpace> Default for FmEmbeddingObserver<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: MeasurableSpace> Observer for FmEmbeddingObserver<S> {
    type Space = S;

    fn score(&self, _trajectory: &Trajectory<S>) -> f64 {
        unimplemented!(
            "FmEmbeddingObserver::score — ASAL (Kumar–Lu–Kirsch–Tang–\
             Stanley–Isola–Ha 2024); not yet implemented"
        )
    }
}

/// Ω metric — residence-time-weighted attractor cycle length
/// (§7.2 Ω family; López-Díaz–Rivera Torres–Febres–Gershenson 2025).
///
/// Ω = (Σ_i τ_i ℓ_i) / (Σ_i τ_i) over the sequence of attractors a_i with
/// cycle lengths ℓ_i and residence times τ_i.
///
/// Ω = 0 iff dynamics settle into a single fixed point; Ω → ∞ as the
/// system visits long-period attractors persistently. Substrate-agnostic
/// — requires only the ability to detect attractors over time.
///
/// **Implementation status.** Stub. Requires an attractor-detection
/// routine on trajectories — deferred. Recent (Dec 2025), not yet widely
/// replicated; treat as a candidate diagnostic.
pub struct OmegaResidenceObserver<S: MeasurableSpace> {
    _space: PhantomData<S>,
}

impl<S: MeasurableSpace> OmegaResidenceObserver<S> {
    pub fn new() -> Self {
        Self {
            _space: PhantomData,
        }
    }
}

impl<S: MeasurableSpace> Default for OmegaResidenceObserver<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: MeasurableSpace> Observer for OmegaResidenceObserver<S> {
    type Space = S;

    fn score(&self, _trajectory: &Trajectory<S>) -> f64 {
        unimplemented!(
            "OmegaResidenceObserver::score — López-Díaz et al. 2025 Ω metric; \
             not yet implemented"
        )
    }
}
