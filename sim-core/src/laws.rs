//! Public test-harness for the laws documented in the math doc.
//!
//! The trait signatures in this crate enforce *structural* matching
//! (every slot speaks the same `Space`), but cannot enforce the *semantic*
//! laws the math doc imposes — closure idempotence (§5.2), one-step mass
//! conservation (§3.1), the Dirac lift's collapse to its underlying map
//! (§1), 𝒮_F's cull-only invariant (§2). Those are properties of how a
//! concrete impl behaves on actual inputs, and Rust's type system does not
//! reach them.
//!
//! This module is the bridge: pure functions that take a slot impl plus a
//! sample point/element and return a [`LawResult`] describing whether the
//! law holds at that sample. Downstream crates implementing the slot
//! traits can call these from their own integration tests, paired with
//! whatever generator strategy they prefer (proptest, quickcheck,
//! hand-rolled). The harness has *no* test-framework dependency of its
//! own — it adds nothing to the crate's runtime deps.
//!
//! ## Laws not covered (and why)
//!
//! - **Extensivity / monotonicity** of
//!   [`ClosureOperator`](crate::closure::ClosureOperator) —
//!   [`ClosureOperator::Element`](crate::closure::ClosureOperator::Element)
//!   has no subset-or-order bound, so the harness cannot express
//!   `A ⊆ cl(A)` generically. Downstream code with a concrete `Element`
//!   that *does* support inclusion can write its own predicate.
//! - **Variation reachability (§4.3)** — irreducibility of `V ∘ Φ` is an
//!   ergodic claim that needs Monte Carlo over many steps, not a
//!   single-call helper. Out of scope for this iteration.
//! - **Viability non-prescriptivity (§5.1)** — a meta-property of how `F`
//!   is *defined*, not testable from an opaque impl. Stays doc-only.

use crate::measure::Point;

/// A single law's failure on a specific input.
///
/// Carries the law's name (a `&'static str` so messages are stable across
/// versions) and a free-text `detail` describing the counterexample
/// concretely enough for a test report to be actionable.
#[derive(Debug, Clone)]
pub struct LawViolation {
    pub law: &'static str,
    pub detail: String,
}

impl std::fmt::Display for LawViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "law `{}` violated: {}", self.law, self.detail)
    }
}

impl std::error::Error for LawViolation {}

/// Result type for every helper in this module.
pub type LawResult = Result<(), LawViolation>;

/// Panic with the violation's `Display` form. Convenience for hand-written
/// tests that prefer a panicking style; programmatic callers should match
/// on the [`LawResult`] directly.
pub fn assert_law(r: LawResult) {
    if let Err(v) = r {
        panic!("{v}");
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Closure operator laws (§5.2)
// ─────────────────────────────────────────────────────────────────────────

pub mod closure {
    //! Laws for [`ClosureOperator`].

    use super::{LawResult, LawViolation};
    use crate::closure::ClosureOperator;

    /// Idempotence: `cl(cl(a)) == cl(a)`. §5.2.
    ///
    /// Tested at the single sample `a`; downstream callers should drive
    /// the helper across a generator of representative samples.
    pub fn idempotence<C>(c: &C, a: &C::Element) -> LawResult
    where
        C: ClosureOperator,
        C::Element: Eq,
    {
        let once = c.close(a);
        let twice = c.close(&once);
        if twice == once {
            Ok(())
        } else {
            Err(LawViolation {
                law: "closure::idempotence",
                detail: "cl(cl(a)) != cl(a) at sample".into(),
            })
        }
    }

    /// `is_closed(a) ⇔ close(a) == a`.
    ///
    /// The trait's rustdoc states this equivalence; a custom impl that
    /// makes one cheaper than the other can drift, and this helper
    /// catches the drift.
    pub fn is_closed_consistent<C>(c: &C, a: &C::Element) -> LawResult
    where
        C: ClosureOperator,
        C::Element: Eq,
    {
        let closed_pred = c.is_closed(a);
        let fixed_point = c.close(a) == *a;
        if closed_pred == fixed_point {
            Ok(())
        } else {
            Err(LawViolation {
                law: "closure::is_closed_consistent",
                detail: format!(
                    "is_closed reported {closed_pred}, but close(a) == a is {fixed_point}"
                ),
            })
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Measure-theoretic laws (§1)
// ─────────────────────────────────────────────────────────────────────────

pub mod measure {
    //! Laws for [`Map`], [`Kernel`], and [`ProbabilityMeasure`].
    //!
    //! `K(y|x) ≥ 0` and `μ(y) ≥ 0` used to live here as runtime helpers;
    //! they've moved into the type system — both
    //! [`Kernel::density`](crate::measure::Kernel::density) and
    //! [`ProbabilityMeasure::density`](crate::measure::ProbabilityMeasure::density)
    //! now return [`Option<NonNeg>`](crate::quantities::NonNeg), so a
    //! negative or NaN density cannot be constructed at all.

    use super::{LawResult, LawViolation, Point};
    use crate::measure::{Dirac, Map};
    use rand_core::Rng;

    /// `Dirac(M).sample(x, _) == M.apply(x)`. §1.
    ///
    /// The Dirac lift is supposed to embed deterministic maps as
    /// degenerate kernels; this helper checks the embedding is faithful
    /// at the sample `x`.
    pub fn dirac_consistent<M, R>(m: &M, x: &Point<M::Source>, rng: &mut R) -> LawResult
    where
        M: Map,
        M: Clone,
        Point<M::Target>: Eq + std::fmt::Debug,
        R: Rng + ?Sized,
    {
        use crate::measure::Kernel as _;
        let direct = m.apply(x);
        let lifted = Dirac(m.clone()).sample(x, rng);
        if direct == lifted {
            Ok(())
        } else {
            Err(LawViolation {
                law: "measure::dirac_consistent",
                detail: format!("Dirac(M).sample(x) = {lifted:?} but M.apply(x) = {direct:?}"),
            })
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Substrate laws (§3)
// ─────────────────────────────────────────────────────────────────────────

pub mod substrate {
    //! Laws for [`Substrate`] and [`MassFunctional`].

    use super::{LawResult, LawViolation, Point};
    use crate::population::Population;
    use crate::substrate::{MassFunctional, Substrate};
    use crate::topology::Topology;
    use rand_core::Rng;

    /// One-step mass conservation: `m(Φ(x; N)) == m(x)`. §3.1.
    ///
    /// Strict equality on `f64`. Substrates whose conservation is only
    /// expected up to numerical error (or in expectation, for stochastic
    /// dynamics) should wrap this with their own tolerance, or call it
    /// across many samples and check the empirical mean. The helper
    /// stays strict to avoid baking a tolerance choice into the harness.
    pub fn mass_conservation<S, MF, R>(
        s: &S,
        mf: &MF,
        x: &Point<S::Space>,
        neighbors: &[&Point<S::Space>],
        rng: &mut R,
    ) -> LawResult
    where
        S: Substrate,
        MF: MassFunctional<Space = S::Space>,
        R: Rng + ?Sized,
    {
        let m_before = mf.mass(x).get();
        let next = s.step(x, neighbors, rng);
        let m_after = mf.mass(&next).get();
        if m_before == m_after {
            Ok(())
        } else {
            Err(LawViolation {
                law: "substrate::mass_conservation",
                detail: format!(
                    "m(x) = {m_before}, m(Φ(x; N)) = {m_after}; difference {}",
                    m_after - m_before
                ),
            })
        }
    }

    /// Population-level conservation: `Σᵢ m(Φ(xᵢ; Nᵢ)) == Σᵢ m(xᵢ)`. §3.1.
    ///
    /// The aggregate-level form of the conservation law — the one that's
    /// actually load-bearing for substrates like Flow-Lenia, where mass
    /// flows between cells but the global integral is preserved.
    /// Pointwise [`mass_conservation`] would fail there even on a
    /// correct impl; this helper is the right tool.
    ///
    /// Drives Φ across every member of `pop` with the topology-supplied
    /// neighbour set, then compares totals. `pop` is borrowed
    /// immutably — no V, no cull, just Φ. Strict equality on `f64`;
    /// callers whose substrates conserve only up to numerical tolerance
    /// should wrap with their own slack budget.
    pub fn total_mass_conservation_under_phi<S, MF, T, R>(
        s: &S,
        mf: &MF,
        topology: &T,
        pop: &Population<S::Space>,
        rng: &mut R,
    ) -> LawResult
    where
        S: Substrate,
        MF: MassFunctional<Space = S::Space>,
        T: Topology<Space = S::Space>,
        R: Rng + ?Sized,
    {
        let before = pop.total_mass(mf).get();
        let mut after: f64 = 0.0;
        for i in 0..pop.len() {
            let neighbors = topology.neighbors(i, pop);
            let stepped = s.step(&pop.members[i], &neighbors, rng);
            after += mf.mass(&stepped).get();
        }
        if before == after {
            Ok(())
        } else {
            Err(LawViolation {
                law: "substrate::total_mass_conservation_under_phi",
                detail: format!(
                    "Σ m(x) = {before}, Σ m(Φ(x; N)) = {after}; difference {}",
                    after - before
                ),
            })
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Simulator-level laws (§2)
// ─────────────────────────────────────────────────────────────────────────

pub mod simulator {
    //! Laws for [`SimulatorInstance`].

    use super::{LawResult, LawViolation};
    use crate::SimulatorInstance;
    use crate::observer::Observer;
    use crate::population::Population;
    use crate::substrate::Substrate;
    use crate::topology::Topology;
    use crate::variation::Variation;
    use crate::viability::Viability;
    use rand_core::Rng;

    /// `𝒮_F` is cull-only: `|pop_{t+1}| ≤ |pop_t|`. §2.
    ///
    /// The math doc allows `𝒮_F` to "possibly produce offspring", but the
    /// scaffold deliberately leaves replication to higher-level
    /// orchestration — see `sim-core/CLAUDE.md`. This helper checks the
    /// scaffold's own contract: a single [`step`](SimulatorInstance::step)
    /// never grows the population.
    pub fn step_does_not_grow_population<S, V, F, T, O, R>(
        sim: &SimulatorInstance<S, V, F, T, O>,
        pop: &mut Population<S::Space>,
        archive: &V::Archive,
        rng: &mut R,
    ) -> LawResult
    where
        S: Substrate,
        V: Variation<Space = S::Space>,
        F: Viability<Space = S::Space>,
        T: Topology<Space = S::Space>,
        O: Observer<Space = S::Space>,
        R: Rng + ?Sized,
    {
        let before = pop.len();
        sim.step(pop, archive, rng);
        let after = pop.len();
        if after <= before {
            Ok(())
        } else {
            Err(LawViolation {
                law: "simulator::step_does_not_grow_population",
                detail: format!("|pop| grew from {before} to {after}"),
            })
        }
    }
}
