//! Trivial slot impls shared across integration tests.
//!
//! Cargo treats every file in `tests/` as a separate binary; the standard
//! way to share code is `tests/common/mod.rs`, declared as `mod common;`
//! in each test file.
//!
//! The fixtures here are scaffolding-grade — identity dynamics, no-op
//! variation, always-survive viability — and exist purely to satisfy the
//! trait bounds of [`SimulatorInstance`](sim_core::SimulatorInstance) so
//! that integration tests can wire one of every slot together.

#![allow(dead_code)] // not every test file uses every fixture

use rand_core::{Infallible, Rng, TryRng};
use sim_core::{
    MeasurableSpace, Observer, Point, Population, Substrate, Topology, Trajectory, Variation,
    Viability,
};

// ── A trivial measurable space ────────────────────────────────────────────

pub struct Reals;
impl MeasurableSpace for Reals {
    type Point = f64;
}

// ── Trivial slots ─────────────────────────────────────────────────────────

pub struct IdentitySubstrate;
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

pub struct NoopVariation;
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

pub struct AlwaysSurvives;
impl Viability for AlwaysSurvives {
    type Space = Reals;
    fn survives(&self, _x: &Point<Reals>, _population: &Population<Reals>) -> bool {
        true
    }
}

pub struct NoNeighbors;
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

pub struct ZeroObserver;
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

pub struct DummyRng;
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
