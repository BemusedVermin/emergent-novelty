//! Niche construction — agents writing to a shared substrate field.
//!
//! Mirrors §8.3 of the math doc.
//!
//! Augments a [`Substrate`](crate::substrate::Substrate) with a shared
//! environmental field e ∈ ℰ that agents both read (via the substrate's
//! `step`, indirectly through observations) and write to (via per-step
//! deposits). The viability filter then becomes
//!
//! ```text
//!   F(x_i, μ, e_t)
//! ```
//!
//! — explicitly dependent on the constructed environment. Without
//! niche construction, a simulator cannot host the
//! Boyd–Richerson–Laland coupling between cultural and genetic evolution.

use crate::measure::{MeasurableSpace, Point};

/// A niche-construction layer on top of a substrate.
///
/// `deposit(x)` produces the per-agent contribution at the agent's
/// location; `accumulate(field, deposit)` writes it into the field;
/// `decay(field)` applies the global decay step
/// `e_{t+1} = e_t + Σ_i deposit_i − decay`.
///
/// This is the direct analogue of [`StigmergicField`](crate::topology::StigmergicField)
/// but framed at the niche-construction level rather than as an
/// alternative interaction topology — the two are co-extensive in the
/// limit. The math doc uses both framings interchangeably; we keep both
/// traits so that users can pick the framing that fits their concrete
/// substrate without forcing a vocabulary.
pub trait NicheConstruction {
    type Space: MeasurableSpace;
    type Field;
    type Deposit;

    /// Per-agent deposit derived from the agent's state.
    fn deposit(&self, x: &Point<Self::Space>) -> Self::Deposit;

    /// Accumulate one agent's deposit into the shared field.
    fn accumulate(&self, field: &mut Self::Field, deposit: Self::Deposit);

    /// Global decay step applied between rounds.
    fn decay(&self, field: &mut Self::Field);
}
