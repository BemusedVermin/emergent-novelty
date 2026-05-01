//! The viability slot F : 𝒳 × P(𝒳) → {0, 1}.
//!
//! Mirrors §5.1 of the math doc.

use crate::measure::{MeasurableSpace, Point};
use crate::population::Population;

/// A viability filter F : 𝒳 × P(𝒳) → {0, 1}. §5.1.
///
/// `survives(x, μ)` returns true iff x is viable in the population μ
/// (represented empirically as a [`Population`]).
///
/// **Non-prescriptive condition.** F is *non-prescriptive* iff it does
/// not factor as
///
/// ```text
///   F(x, μ) = 𝟙[ f(x) > τ ]
/// ```
///
/// for some scalar fitness f and threshold τ — equivalently, if it does
/// not reduce to a global objective. The simulator does not enforce this;
/// it is a discipline of the design.
///
/// The four canonical non-prescriptive viability formalisms are
/// [`MarkovBlanketClosure`](crate::closure::MarkovBlanketClosure),
/// [`AutopoieticClosure`](crate::closure::AutopoieticClosure),
/// [`RafClosure`](crate::closure::RafClosure), and
/// [`MinimalCriterionClosure`](crate::closure::MinimalCriterionClosure)
/// — each definable as `F(x, μ) = 𝟙[ supporting_set(x, μ) is closed ]`.
pub trait Viability {
    type Space: MeasurableSpace;

    fn survives(&self, x: &Point<Self::Space>, population: &Population<Self::Space>) -> bool;
}
