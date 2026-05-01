//! Trajectories T_t = (x_0, x_1, …, x_t) ∈ 𝒳^{t+1}.
//!
//! Mirrors the notation `T_t` in §1 of the math doc. The observer functional
//! 𝒪 takes trajectories as input.

use crate::measure::{MeasurableSpace, Point};

/// A trajectory T_t = (x_0, x_1, …, x_t) over a measurable space X.
///
/// The points are stored in temporal order: `points[0]` is the earliest.
/// This is the time series an [`Observer`](crate::observer::Observer)
/// scores; for a population trajectory, instantiate with X being a
/// population-shaped space (e.g., `MeasurableSpace<Point = Vec<P>>`).
pub struct Trajectory<S: MeasurableSpace> {
    pub points: Vec<Point<S>>,
}

impl<S: MeasurableSpace> Trajectory<S> {
    /// An empty trajectory.
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    /// A trajectory pre-seeded with x_0.
    pub fn from_initial(x0: Point<S>) -> Self {
        Self { points: vec![x0] }
    }

    /// Append x_{t+1}.
    pub fn push(&mut self, x: Point<S>) {
        self.points.push(x);
    }

    /// Number of recorded states (= t + 1 for T_t).
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// True iff no states recorded.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// The most recent state x_t, if any.
    pub fn last(&self) -> Option<&Point<S>> {
        self.points.last()
    }

    /// Iterate over states in temporal order.
    pub fn iter(&self) -> std::slice::Iter<'_, Point<S>> {
        self.points.iter()
    }
}

impl<S: MeasurableSpace> Default for Trajectory<S> {
    fn default() -> Self {
        Self::new()
    }
}
