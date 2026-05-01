//! Smoke test — wires one trivial in-test impl per slot into
//! `SimulatorInstance::step()` and runs one update. Proves the trait
//! bounds compose end-to-end.
//!
//! The trivial impls live in `tests/common/mod.rs` so that other
//! integration tests (notably `tests/laws.rs`) can reuse them.

mod common;

use common::{
    AlwaysSurvives, DummyRng, IdentitySubstrate, NoNeighbors, NoopVariation, Reals, ZeroObserver,
};
use sim_core::{Observer, Population, SimulatorInstance, Trajectory};

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
