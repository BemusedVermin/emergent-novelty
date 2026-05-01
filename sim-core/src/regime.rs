//! Dynamical regime — Wolfram class and Langton λ as marker types.
//!
//! Mirrors §3.3 and §9 of the math doc.
//!
//! A substrate's regime is a coarse classification of long-run behaviour.
//! For 1D and 2D CAs, [`WolframClass`] partitions rule space; Langton's
//! [`LangtonLambda`] is a continuous proxy. Empirically, Class IV /
//! edge-of-chaos / λ ≈ λ_c is where evolvability and computational depth
//! co-occur — a substrate-design heuristic, not a property the simulator
//! can verify at runtime.
//!
//! These types are markers only. There is no estimator here; concrete
//! substrates can carry them as descriptive metadata.

/// Wolfram's four classes of cellular-automaton long-run behaviour.
///
/// Class I (quiescent), II (periodic), III (chaotic), IV (complex /
/// edge-of-chaos). The non-prescriptive design recommendation is that the
/// substrate's parameterisation should make Class IV reachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WolframClass {
    /// Class I — dynamics quench to a fixed point / quiescent state.
    Quiescent,
    /// Class II — dynamics settle into simple periodic structures.
    Periodic,
    /// Class III — dynamics produce aperiodic / chaotic patterns.
    Chaotic,
    /// Class IV — complex, "edge-of-chaos"; localised structures with
    /// long-range interactions. Empirically the regime in which
    /// evolvability and computational depth co-occur.
    Complex,
}

/// Langton's λ parameter — Pr_x[ Φ(x) ≠ q ], the probability that the
/// update produces something other than the quiescent state q.
///
/// A continuous proxy for [`WolframClass`]. For 2D outer-totalistic CAs,
/// the critical value is empirically λ_c ≈ 0.273; mutual information
/// between successive states peaks near λ_c.
///
/// Convention: 0.0 ≤ λ ≤ 1.0; not enforced by the type.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LangtonLambda(pub f64);
