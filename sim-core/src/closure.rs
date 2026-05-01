//! Closure operators on substrate-specific lattices.
//!
//! Mirrors §5.2 of the math doc. The unification claim there is that the
//! four "viability" notions —
//!
//! - Markov blanket (active inference; Kirchhoff–Parr–Palacios–Friston–
//!   Kiverstein 2018),
//! - autopoietic closure (Maturana–Varela; Beer's Game-of-Life work),
//! - RAF set (Kauffman; Hordijk–Steel detection algorithm),
//! - minimal-criterion coevolution (Brant–Stanley; POET extends),
//!
//! — are instances of the *same* abstract structure: each defines a
//! [`ClosureOperator`] on a substrate-specific lattice (subsets of joint
//! state, subsets of components, subsets of reactions, pairs of populations)
//! and "an entity exists" iff its supporting set is a fixed point of that
//! operator.
//!
//! This module defines the abstract trait and the four named stubs. The
//! algorithms behind `close()` are deferred to a later pass — only RAF
//! detection (Hordijk–Steel) has a known polynomial-time algorithm; the
//! others are NP-hard or unsolved on general substrates.

use crate::measure::{MeasurableSpace, Point};
use std::marker::PhantomData;

/// A closure operator cl on a poset (𝒞, ⊆).
///
/// **Required laws** (documented; not enforced by the type system):
///
/// 1. *Extensivity:* `A ⊆ cl(A)`.
/// 2. *Monotonicity:* `A ⊆ B ⇒ cl(A) ⊆ cl(B)`.
/// 3. *Idempotence:* `cl(cl(A)) = cl(A)`.
///
/// A set is *closed* iff `A = cl(A)`. The viability slot consults
/// `is_closed` on an entity's supporting set to decide existence.
///
/// Rust cannot enforce the three laws in the type system. A property-test
/// helper `closure_laws<C: ClosureOperator>(c, samples)` can be added later
/// that exercises them on representative inputs.
pub trait ClosureOperator {
    /// The lattice element type. Conventionally a set, multiset, or tuple
    /// of sets; the trait does not require any particular representation.
    type Element;

    /// Compute cl(A), the smallest closed superset of A.
    fn close(&self, a: &Self::Element) -> Self::Element;

    /// Decide A = cl(A). Equivalent to `cl(A) == A` when `Element: Eq`.
    fn is_closed(&self, a: &Self::Element) -> bool;
}

// ─────────────────────────────────────────────────────────────────────────
// §5.2.1 Markov blanket closure
// ─────────────────────────────────────────────────────────────────────────

/// Markov blanket closure (§5.2.1).
///
/// Lattice: subsets of the joint state space, partitioned into hidden
/// states ℋ, sensory states 𝒮, active states 𝒜, and external states ℰ.
/// A blanket B = 𝒮 ∪ 𝒜 satisfies `p(h, e | b) = p(h | b) p(e | b)`
/// — equivalently, `ℋ ⊥ ℰ | B`.
///
/// `cl_MB(A)` is the smallest superset of A whose complement is
/// conditionally independent of A given the boundary.
///
/// **Implementation status.** Stub. Detection of MBs on arbitrary
/// stochastic systems is generally hard; existence of MBs for non-
/// equilibrium steady-state systems is contested (Aguilera–Millidge–
/// Tschantz–Buckley 2022; see §10 of the math doc).
pub struct MarkovBlanketClosure<S: MeasurableSpace> {
    _space: PhantomData<S>,
}

impl<S: MeasurableSpace> MarkovBlanketClosure<S> {
    pub fn new() -> Self {
        Self {
            _space: PhantomData,
        }
    }
}

impl<S: MeasurableSpace> Default for MarkovBlanketClosure<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: MeasurableSpace> ClosureOperator for MarkovBlanketClosure<S> {
    /// A subset of joint states, represented as a multiset of points.
    type Element = Vec<Point<S>>;

    fn close(&self, _a: &Self::Element) -> Self::Element {
        unimplemented!(
            "MarkovBlanketClosure::close — see Kirchhoff–Parr–Palacios–\
             Friston–Kiverstein 2018; not yet implemented"
        )
    }

    fn is_closed(&self, _a: &Self::Element) -> bool {
        unimplemented!("MarkovBlanketClosure::is_closed — not yet implemented")
    }
}

// ─────────────────────────────────────────────────────────────────────────
// §5.2.2 Autopoietic closure
// ─────────────────────────────────────────────────────────────────────────

/// Autopoietic closure (§5.2.2; Maturana–Varela; Beer's CA work).
///
/// Lattice: subsets of components 𝒞 with an auxiliary "produces" /
/// "requires" bipartite relation against a set of production processes 𝒫.
/// A subset C ⊆ 𝒞 is *organisationally closed* iff there exists P ⊆ 𝒫
/// such that every c ∈ C is produced by some p ∈ P, every p ∈ P requires
/// only components in C, and P produces a topological boundary that
/// distinguishes C from its environment.
///
/// `cl_AP(C)` is the smallest closed superset of C.
///
/// **Implementation status.** Stub. Beer's 2014–2015 Game-of-Life work
/// gives a concrete algorithm on a discrete CA substrate; general-substrate
/// detection is hard.
pub struct AutopoieticClosure<C> {
    _component: PhantomData<C>,
}

impl<C> AutopoieticClosure<C> {
    pub fn new() -> Self {
        Self {
            _component: PhantomData,
        }
    }
}

impl<C> Default for AutopoieticClosure<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> ClosureOperator for AutopoieticClosure<C> {
    /// A subset of components, represented as a multiset.
    type Element = Vec<C>;

    fn close(&self, _a: &Self::Element) -> Self::Element {
        unimplemented!(
            "AutopoieticClosure::close — Maturana–Varela; see Beer 2014/2015 \
             for the discrete-CA case; not yet implemented"
        )
    }

    fn is_closed(&self, _a: &Self::Element) -> bool {
        unimplemented!("AutopoieticClosure::is_closed — not yet implemented")
    }
}

// ─────────────────────────────────────────────────────────────────────────
// §5.2.3 RAF closure (Reflexively Autocatalytic and Food-generated)
// ─────────────────────────────────────────────────────────────────────────

/// RAF closure (§5.2.3; Kauffman; Hordijk–Steel detection).
///
/// Lattice: subsets of reactions ℛ with a food set F ⊆ 𝒳 and a per-
/// reaction (reactants, products, catalyst) specification.
/// R' ⊆ ℛ is RAF iff:
///
/// - *Reflexive autocatalysis:* every r ∈ R' is catalysed by some product
///   of R' or by an element of F.
/// - *Food-generated:* every reactant of every r ∈ R' is producible by
///   R' ∪ F.
///
/// `cl_RAF(R')` is the maximal RAF subset of R' (computable in polynomial
/// time by Hordijk–Steel).
///
/// **Implementation status.** Stub. RAF detection is the cleanest
/// computational-complexity treatment of autopoiesis; the algorithm is
/// known but not yet ported.
pub struct RafClosure<Reaction, Molecule> {
    /// The food set F ⊆ 𝒳 — molecules assumed available without
    /// production, the substrate against which the RAF property is
    /// evaluated.
    pub food_set: Vec<Molecule>,
    _reaction: PhantomData<Reaction>,
}

impl<Reaction, Molecule> RafClosure<Reaction, Molecule> {
    pub fn new(food_set: Vec<Molecule>) -> Self {
        Self {
            food_set,
            _reaction: PhantomData,
        }
    }
}

impl<Reaction, Molecule> ClosureOperator for RafClosure<Reaction, Molecule> {
    /// A subset of reactions, represented as a multiset.
    type Element = Vec<Reaction>;

    fn close(&self, _a: &Self::Element) -> Self::Element {
        unimplemented!(
            "RafClosure::close — Hordijk–Steel polynomial-time RAF detection; \
             not yet implemented"
        )
    }

    fn is_closed(&self, _a: &Self::Element) -> bool {
        unimplemented!("RafClosure::is_closed — not yet implemented")
    }
}

// ─────────────────────────────────────────────────────────────────────────
// §5.2.4 Minimal-criterion coevolution closure
// ─────────────────────────────────────────────────────────────────────────

/// Minimal-criterion coevolution closure (§5.2.4; Brant–Stanley; POET).
///
/// Lattice: pairs (P_A, P_B) of populations. The pair is closed under
/// mutual minimal criteria iff every a ∈ P_A satisfies the relational
/// predicate φ_A(a, P_B) and every b ∈ P_B satisfies φ_B(b, P_A).
///
/// `cl_MCC(P_A, P_B)` is the largest pair satisfying both predicates — a
/// fixed point of the two-population predicate.
///
/// **Implementation status.** Stub. Each predicate evaluation is a single
/// pass over the opposing population; the full closure is iterative
/// elimination until stable.
pub struct MinimalCriterionClosure<A: MeasurableSpace, B: MeasurableSpace> {
    _spaces: PhantomData<(A, B)>,
}

impl<A: MeasurableSpace, B: MeasurableSpace> MinimalCriterionClosure<A, B> {
    pub fn new() -> Self {
        Self {
            _spaces: PhantomData,
        }
    }
}

impl<A: MeasurableSpace, B: MeasurableSpace> Default for MinimalCriterionClosure<A, B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: MeasurableSpace, B: MeasurableSpace> ClosureOperator for MinimalCriterionClosure<A, B> {
    /// A pair of populations (P_A, P_B).
    type Element = (Vec<Point<A>>, Vec<Point<B>>);

    fn close(&self, _a: &Self::Element) -> Self::Element {
        unimplemented!(
            "MinimalCriterionClosure::close — Brant–Stanley GECCO 2017; \
             not yet implemented"
        )
    }

    fn is_closed(&self, _a: &Self::Element) -> bool {
        unimplemented!("MinimalCriterionClosure::is_closed — not yet implemented")
    }
}
