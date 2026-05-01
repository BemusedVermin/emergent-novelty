//! The variation slot V : 𝒳 → P(𝒳).
//!
//! Mirrors §4 of the math doc.

use crate::measure::{MeasurableSpace, Point};
use rand_core::Rng;

/// A variation operator V : 𝒳 → P(𝒳). §4.1.
///
/// V is a Markov kernel from 𝒳 to itself; calling `perturb(x, archive, rng)`
/// samples y ~ V(·|x; A_t).
///
/// **Locality (§4.1).** A variation operator is *local* if its support
/// concentrates on a small metric neighbourhood of x; *global* otherwise.
/// Neither is enforced — this is documentation.
///
/// **Reachability (§4.3).** The composition V ∘ Φ should be irreducible
/// on 𝒳: for any open U ⊆ 𝒳 and any x_0, ∃T such that
/// Pr[x_T ∈ U | x_0] > 0. This is the formal floor of the
/// "adjacent-possible" argument; violating it (e.g., a frozen archive of
/// fixed mutations) implicitly bounds the system. Documented expectation,
/// not enforced.
///
/// **Archive parameter.** `Archive` carries any state the operator is
/// conditioned on:
///
/// - memoryless operators (Gaussian, bit-flip): set `type Archive = ();`
///   and ignore the parameter;
/// - IMGEP / curiosity-driven (Forestier–Oudeyer): the descriptor archive;
/// - FM-guided mutation (OMNI-EPIC, ASAL): the FM context;
/// - prompt mutation (Vallinder–Hughes): the prior generation's strategies.
///
/// Splitting the archive out as an explicit parameter, rather than mutable
/// state on `self`, keeps `Variation` pure with respect to `&self` and
/// matches the math notation V(x; A_t).
pub trait Variation {
    type Space: MeasurableSpace;

    /// State on which V is conditioned. Use `()` for memoryless operators.
    type Archive;

    fn perturb<R: Rng + ?Sized>(
        &self,
        x: &Point<Self::Space>,
        archive: &Self::Archive,
        rng: &mut R,
    ) -> Point<Self::Space>;
}
