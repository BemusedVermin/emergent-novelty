//! Exercises the public laws harness on hand-rolled fixtures.
//!
//! Two roles:
//! 1. *Self-test.* Confirm each helper returns `Ok` on a correct impl and
//!    `Err` with a useful detail on a deliberately-broken one.
//! 2. *Worked example.* Show downstream developers the shape of a test
//!    that uses `sim_core::laws::*` against their own slot impls.

mod common;

use common::{
    AlwaysSurvives, DummyRng, IdentitySubstrate, NoNeighbors, NoopVariation, Reals, ZeroObserver,
};
use sim_core::closure::ClosureOperator;
use sim_core::laws::{self, assert_law};
use sim_core::measure::{Map, MeasurableSpace, Point};
use sim_core::quantities::NonNeg;
use sim_core::substrate::MassFunctional;
use sim_core::{Population, SimulatorInstance};

// ── Closure laws ──────────────────────────────────────────────────────────

/// Always returns the input unchanged. Trivially closed; trivially
/// idempotent.
struct IdentityClosure;
impl ClosureOperator for IdentityClosure {
    type Element = Vec<u32>;
    fn close(&self, a: &Self::Element) -> Self::Element {
        a.clone()
    }
    fn is_closed(&self, _a: &Self::Element) -> bool {
        true
    }
}

/// `close` is idempotent (returns input) but `is_closed` lies (always
/// false). Drives the `is_closed_consistent` helper into reporting an
/// `Err`.
struct LyingClosure;
impl ClosureOperator for LyingClosure {
    type Element = Vec<u32>;
    fn close(&self, a: &Self::Element) -> Self::Element {
        a.clone()
    }
    fn is_closed(&self, _a: &Self::Element) -> bool {
        false
    }
}

#[test]
fn identity_closure_passes_idempotence_and_consistency() {
    let c = IdentityClosure;
    let sample = vec![1, 2, 3];
    assert_law(laws::closure::idempotence(&c, &sample));
    assert_law(laws::closure::is_closed_consistent(&c, &sample));
}

#[test]
fn lying_closure_is_caught_by_is_closed_consistent() {
    let c = LyingClosure;
    let sample = vec![1, 2, 3];

    // `close` is honestly idempotent — that helper should pass.
    assert_law(laws::closure::idempotence(&c, &sample));

    // …but `is_closed` contradicts `close(a) == a`.
    let result = laws::closure::is_closed_consistent(&c, &sample);
    let violation = result.expect_err("expected a violation");
    assert_eq!(violation.law, "closure::is_closed_consistent");
    assert!(
        violation.detail.contains("is_closed reported false"),
        "detail should describe the contradiction; got {:?}",
        violation.detail
    );
}

// ── Measure laws ──────────────────────────────────────────────────────────

#[derive(Clone)]
struct DoubleMap;
impl Map for DoubleMap {
    type Source = Reals;
    type Target = Reals;
    fn apply(&self, x: &Point<Reals>) -> Point<Reals> {
        x * 2.0
    }
}

// `Reals::Point = f64`, which doesn't implement `Eq`. Wrap in a tiny
// struct so the harness's `Eq` bound is satisfied.
#[derive(Debug, PartialEq, Eq, Clone)]
struct Bits(u64);

struct BitsSpace;
impl MeasurableSpace for BitsSpace {
    type Point = Bits;
}

#[derive(Clone)]
struct ShiftBits;
impl Map for ShiftBits {
    type Source = BitsSpace;
    type Target = BitsSpace;
    fn apply(&self, x: &Bits) -> Bits {
        Bits(x.0 << 1)
    }
}

#[test]
fn dirac_lift_is_consistent_with_underlying_map() {
    let m = ShiftBits;
    let mut rng = DummyRng;
    let x = Bits(42);
    assert_law(laws::measure::dirac_consistent(&m, &x, &mut rng));
}

// ── Substrate / mass laws ─────────────────────────────────────────────────

/// A mass functional that returns the absolute value of `x`. Conserved
/// under `IdentitySubstrate`.
struct AbsMass;
impl MassFunctional for AbsMass {
    type Space = Reals;
    fn mass(&self, x: &Point<Reals>) -> NonNeg {
        NonNeg::new(x.abs()).expect("|x| is finite & non-negative for finite x")
    }
}

#[test]
fn identity_substrate_conserves_abs_mass() {
    let s = IdentitySubstrate;
    let mf = AbsMass;
    let mut rng = DummyRng;
    let x: f64 = 3.5;
    let neighbors: Vec<&f64> = Vec::new();
    assert_law(laws::substrate::mass_conservation(
        &s, &mf, &x, &neighbors, &mut rng,
    ));
}

/// Φ that doubles `x` — explicitly *not* mass-conserving against
/// [`AbsMass`]. Drives the helper into reporting an `Err`.
struct DoubleSubstrate;
impl sim_core::Substrate for DoubleSubstrate {
    type Space = Reals;
    fn initial<R: rand_core::Rng + ?Sized>(&self, _rng: &mut R) -> Point<Reals> {
        0.0
    }
    fn step<R: rand_core::Rng + ?Sized>(
        &self,
        x: &Point<Reals>,
        _neighbors: &[&Point<Reals>],
        _rng: &mut R,
    ) -> Point<Reals> {
        x * 2.0
    }
}

#[test]
fn doubling_substrate_violates_mass_conservation() {
    let s = DoubleSubstrate;
    let mf = AbsMass;
    let mut rng = DummyRng;
    let x: f64 = 3.5;
    let neighbors: Vec<&f64> = Vec::new();
    let result = laws::substrate::mass_conservation(&s, &mf, &x, &neighbors, &mut rng);
    let v = result.expect_err("expected a conservation violation");
    assert_eq!(v.law, "substrate::mass_conservation");
}

// ── Simulator-level laws ──────────────────────────────────────────────────

#[test]
fn step_does_not_grow_population_under_trivial_slots() {
    let sim = SimulatorInstance {
        substrate: IdentitySubstrate,
        variation: NoopVariation,
        viability: AlwaysSurvives,
        topology: NoNeighbors,
        observer: ZeroObserver,
    };
    let mut pop: Population<Reals> = Population::from_members(vec![0.0, 1.0, 2.0]);
    let mut rng = DummyRng;
    assert_law(laws::simulator::step_does_not_grow_population(
        &sim,
        &mut pop,
        &(),
        &mut rng,
    ));
}

// `DoubleMap` is unused above; reference it here so the lint stays
// quiet without an `#[allow(dead_code)]` on the type itself.
#[test]
fn double_map_compiles() {
    let _ = DoubleMap;
}
