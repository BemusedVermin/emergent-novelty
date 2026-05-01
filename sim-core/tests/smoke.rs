//! Smoke test — wires one trivial in-test impl per slot into
//! `SimulatorInstance::step()` and runs one update. Proves the trait
//! bounds compose end-to-end.
//!
//! These trivial impls are scaffolding-grade fixtures, not production
//! types; they intentionally do nothing interesting.

use rand_core::{Infallible, Rng, TryRng};
use sim_core::{
    MeasurableSpace, Observer, Point, Population, SimulatorInstance, Substrate, Topology,
    Trajectory, Variation, Viability,
};

// ── A trivial measurable space ────────────────────────────────────────────

struct Reals;
impl MeasurableSpace for Reals {
    type Point = f64;
}

// ── Trivial slots ─────────────────────────────────────────────────────────

struct IdentitySubstrate;
impl Substrate for IdentitySubstrate {
    type Space = Reals;
    fn initial<R: Rng + ?Sized>(&self, _rng: &mut R) -> Point<Reals> {
        0.0
    }
    fn step<R: Rng + ?Sized>(
        &self,
        x: &Point<Reals>,
        _neighbors: &[&Point<Reals>],
        _rng: &mut R,
    ) -> Point<Reals> {
        *x
    }
}

struct NoopVariation;
impl Variation for NoopVariation {
    type Space = Reals;
    type Archive = ();
    fn perturb<R: Rng + ?Sized>(
        &self,
        x: &Point<Reals>,
        _archive: &Self::Archive,
        _rng: &mut R,
    ) -> Point<Reals> {
        *x
    }
}

struct AlwaysSurvives;
impl Viability for AlwaysSurvives {
    type Space = Reals;
    fn survives(&self, _x: &Point<Reals>, _population: &Population<Reals>) -> bool {
        true
    }
}

struct NoNeighbors;
impl Topology for NoNeighbors {
    type Space = Reals;
    fn neighbors<'a>(
        &self,
        _i: usize,
        _population: &'a Population<Reals>,
    ) -> Vec<&'a Point<Reals>> {
        Vec::new()
    }
}

struct ZeroObserver;
impl Observer for ZeroObserver {
    type Space = Reals;
    fn score(&self, _trajectory: &Trajectory<Reals>) -> f64 {
        0.0
    }
}

// ── A throwaway RNG (the trivial slots above never read it) ───────────────
//
// In rand_core 0.10 implementors target `TryRng`; the blanket
// `impl<R: TryRng<Error = Infallible>> Rng for R` auto-promotes us into
// the infallible-`Rng` world used by the lib's trait bounds.

struct DummyRng;
impl TryRng for DummyRng {
    type Error = Infallible;
    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        Ok(0)
    }
    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        Ok(0)
    }
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
        for b in dst.iter_mut() {
            *b = 0;
        }
        Ok(())
    }
}

#[test]
fn step_composes_all_five_slots() {
    let sim = SimulatorInstance {
        substrate: IdentitySubstrate,
        variation: NoopVariation,
        viability: AlwaysSurvives,
        topology: NoNeighbors,
        observer: ZeroObserver,
    };

    let mut pop: Population<Reals> = Population::from_members(vec![0.0, 1.0, 2.0, 3.0]);
    let mut rng = DummyRng;

    sim.step(&mut pop, &(), &mut rng);

    // Identity Φ + no-op V + always-true F should leave the population
    // intact in both cardinality and content.
    assert_eq!(pop.len(), 4);
    assert_eq!(pop.members, vec![0.0, 1.0, 2.0, 3.0]);

    // Observer is reachable but `step` did not invoke it; calling it
    // explicitly returns the trivial score.
    let traj: Trajectory<Reals> = Trajectory::from_initial(0.0);
    assert_eq!(sim.observer.score(&traj), 0.0);
}
