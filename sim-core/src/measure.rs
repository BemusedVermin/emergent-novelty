//! Measure-theoretic foundations.
//!
//! Mirrors §1 of the math doc. Every other module typifies its objects in
//! terms of the four traits defined here:
//!
//! - [`MeasurableSpace`] — a set X with an implicit σ-algebra.
//! - [`ProbabilityMeasure`] — μ ∈ P(Y).
//! - [`Map`] — a deterministic measurable map f : X → Y.
//! - [`Kernel`] — a Markov kernel K : X → P(Y).
//!
//! Every `Map` canonically lifts to a `Kernel` via [`Dirac`].

use rand_core::Rng;

/// A measurable space (X, ℱ).
///
/// In the math doc this is denoted 𝒳 (single-entity state space) or 𝒴
/// (codomain of a map). Here we represent it as a marker trait whose
/// associated `Point` type inhabits the underlying set X. The σ-algebra ℱ
/// is implicit — Borel for the standard cases — since Rust types do not
/// carry runtime σ-algebras.
///
/// All measure-theoretic operators ([`ProbabilityMeasure`], [`Kernel`],
/// [`Map`]) are typed in terms of `MeasurableSpace`s rather than raw
/// `Point` types so that the math signatures map across one-to-one.
pub trait MeasurableSpace {
    /// An inhabitant of X.
    type Point: Clone;
}

/// Convenience alias for the `Point` type of a measurable space.
pub type Point<S> = <S as MeasurableSpace>::Point;

/// A probability measure μ ∈ P(Y) on a measurable space Y.
///
/// Concrete measures expose at minimum [`sample`](Self::sample) — draw
/// y ~ μ. Some can additionally expose a density μ(dy)/dλ(y) against a
/// reference measure λ (Lebesgue, counting, …); [`density`](Self::density)
/// is optional and defaults to `None`, since many useful measures (implicit
/// pushforwards through neural nets, stigmergic field reads, …) sample but
/// have no closed-form density.
pub trait ProbabilityMeasure {
    type Space: MeasurableSpace;

    /// Draw y ~ μ.
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Point<Self::Space>;

    /// Density at y against a reference measure, if expressible.
    fn density(&self, _y: &Point<Self::Space>) -> Option<f64> {
        None
    }
}

/// A measurable map f : X → Y between measurable spaces. Deterministic.
///
/// For a stochastic transition use [`Kernel`]. Every `Map` lifts canonically
/// to a `Kernel` via [`Dirac`] — the standard categorical embedding of the
/// deterministic case into the stochastic.
pub trait Map {
    type Source: MeasurableSpace;
    type Target: MeasurableSpace;

    fn apply(&self, x: &Point<Self::Source>) -> Point<Self::Target>;
}

/// A Markov kernel K : X → P(Y).
///
/// For each x ∈ X, K(·|x) is a probability measure on Y. We collapse the
/// (build measure at x, then sample) pair into a single `sample(x, rng)`
/// call: materialising a fresh `ProbabilityMeasure` at every x is wasteful
/// for the common case where we only ever sample. A kernel-as-measure-
/// builder factoring (`fn at(&self, x) -> Self::Measure`) can be layered on
/// later if explicit pushforwards become useful.
///
/// `density(x, y)` reports K(dy|x)/dλ(y) where defined; defaults to `None`
/// for the same reason as on [`ProbabilityMeasure`].
///
/// **Reachability (§4.3 of the math doc).** A kernel induces a Markov chain
/// on its source space. The reachability proposition there asks that the
/// chain be irreducible — equivalently, that the kernel's support cover a
/// neighbourhood basis. We document this as an expectation; it is not
/// enforced at the type level. A property-test helper can be added later.
pub trait Kernel {
    type Source: MeasurableSpace;
    type Target: MeasurableSpace;

    /// Sample y ~ K(·|x).
    fn sample<R: Rng + ?Sized>(&self, x: &Point<Self::Source>, rng: &mut R) -> Point<Self::Target>;

    /// Density K(y|x) if expressible. Default: `None`.
    fn density(&self, _x: &Point<Self::Source>, _y: &Point<Self::Target>) -> Option<f64> {
        None
    }
}

/// The Dirac lift δ_f of a deterministic map f : X → Y, exhibiting f as a
/// degenerate Markov kernel that places all mass on f(x).
///
/// Categorically, this is the embedding of the Kleisli category of the
/// identity monad into the Kleisli category of the Giry monad: every
/// deterministic map is a kernel that ignores its randomness source.
pub struct Dirac<M: Map>(pub M);

impl<M: Map> Kernel for Dirac<M> {
    type Source = M::Source;
    type Target = M::Target;

    fn sample<R: Rng + ?Sized>(
        &self,
        x: &Point<Self::Source>,
        _rng: &mut R,
    ) -> Point<Self::Target> {
        self.0.apply(x)
    }

    // density: a Dirac measure has no L¹ density against Lebesgue (it is a
    // singular distribution). We leave `density` at the trait default
    // `None` rather than fabricate a delta-function representation.
}
