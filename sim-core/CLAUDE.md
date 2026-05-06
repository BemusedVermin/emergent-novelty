# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Authoritative spec

This crate is a **one-to-one Rust translation** of `../docs/notes/emergent_systems_mathematical_treatment.md`. That document is the source of truth — the trait signatures, doc comments, and section references in the code (e.g. "§2", "§5.2.1", "§7.2") all point back into it. When adding or modifying a slot, read the relevant section of the math doc first; deviating from it without updating the math doc is a smell.

## The six-tuple, in one paragraph

A simulator is `M = (𝒳, Φ, V, F, 𝒯, 𝒪)`: a measurable space `𝒳`, substrate dynamics `Φ`, variation `V`, viability `F`, topology `𝒯`, observer `𝒪`. Each lives in its own module (`substrate`, `variation`, `viability`, `topology`, `observer`) as a trait whose `Space: MeasurableSpace` associated type ties the slot to the substrate. `SimulatorInstance` in `lib.rs` glues all five slots together; its `where` bounds enforce that every slot's `Space` equals `S::Space`, so wiring a mismatched slot is a compile error.

`SimulatorInstance::step` is the population update `μ_{t+1} = 𝒮_F ∘ V ∘ Φ_𝒯 (μ_t)`, implemented as a literal three-stage pipeline (substrate-step → variation → cull). **The observer is reachable on `self` but is never invoked by `step`** — that omission is the formal expression of non-prescriptive design (§2 of the math doc). Do not introduce a feedback path from `Observer` into the dynamics; if you need to score during a run, score externally and let the user act on the score.

`𝒮_F` is intentionally **cull-only**: entities that fail viability are removed, but no replication happens here. The math doc's "possibly producing offspring" half is left to higher-level orchestration so the scaffold doesn't commit to a replication policy.

## Measure-theoretic foundations

`measure.rs` holds four traits everything else builds on:

- `MeasurableSpace` — a marker trait with an associated `Point: Clone`. The σ-algebra is implicit (Borel for standard cases).
- `ProbabilityMeasure` — `sample(rng) -> Point`, with optional `density`.
- `Map` — deterministic `apply(x) -> y`.
- `Kernel` — stochastic `sample(x, rng) -> y`. `Dirac<M: Map>` lifts any `Map` into a `Kernel`.

Every operator type signature is written in terms of `MeasurableSpace`s rather than raw `Point` types, so the math signatures and the Rust signatures map across one-to-one. Preserve that discipline when adding new traits.

## Stubs vs. live code

Currently live:
- `SimulatorInstance::step` and the five slot traits (callers can implement them).
- `Population`, `Trajectory`, `Dirac` — basic data types.

Currently stubbed (`unimplemented!`):
- All four `ClosureOperator` impls in `closure.rs` (`MarkovBlanketClosure`, `AutopoieticClosure`, `RafClosure`, `MinimalCriterionClosure`).
- All three named observers in `observer.rs` (`KnnNoveltyObserver`, `FmEmbeddingObserver`, `OmegaResidenceObserver`).

When implementing a stub, keep the math-doc citation in the doc comment (e.g. "Hordijk–Steel polynomial-time RAF detection") and remove the `unimplemented!` panic; the stub structure is deliberately minimal so the implementation can land without re-shaping the public API.

## Common commands

```
cargo build              # compile the crate
cargo test               # run unit + integration tests (tests/smoke.rs is the wiring smoke test)
cargo test step_composes # run a single test by name
cargo doc --open         # render the math-doc-linked rustdoc; the docs ARE the spec, not decoration
cargo clippy             # lint
cargo fmt                # format
```

The `tests/smoke.rs` integration test wires one trivial impl per slot into `SimulatorInstance::step` and asserts the trait bounds compose. Keep it green — it is the canary for any change that touches a slot trait signature.

## Public test harness for downstream impls

`sim_core::laws` exposes pure-functional checkers for the math doc's laws — closure idempotence (§5.2), the Dirac lift's collapse to its underlying map (§1), kernel-density non-negativity, one-step mass conservation (§3.1), and 𝒮_F's cull-only invariant (§2). Each helper takes a slot impl plus a sample input and returns `Result<(), LawViolation>`; downstream crates can drive them with proptest, quickcheck, or hand-rolled samples — the harness has no test-framework dep of its own. When you add a new slot trait or a new documented law, add a matching `laws::*` helper in the same change so the property is checkable from outside the crate.

The companion `sim_core::quantities` module exposes `NonNeg` and `Probability` — the two pointwise constraints that are cheap to check at construction. New scalar fields whose math-doc range is `ℝ_+` or `[0, 1]` should reach for these rather than reintroducing bare `f64`.

## Conventions

- **Edition 2024.** The runtime crate's only hard dep stays `rand_core = "0.10"` so the scaffold remains substrate-agnostic. Static-correctness tooling (`flux`, `creusot`, `typenum`) is **feature-gated** — see the Verification stack below — and never enters a default build.
- **`rand_core::Rng` everywhere** as `&mut R: Rng + ?Sized`, matching the math doc's stochastic-kernel signatures. The smoke test shows the `TryRng → Rng` blanket-impl path for tests.
- **Doc comments cite math-doc sections.** Maintain that convention when extending; it is how a future reader navigates from code back to the formalism.
- **Type-level enforcement first.** When the math doc imposes a constraint, reach for the strongest static check that fits *before* falling back to runtime helpers. The progression is:
  1. **Newtype wrappers** ([`NonNeg`], [`Probability`], `LangtonLambda`) for pointwise ranges Rust's base types can't express.
  2. **Refinement types via Flux** (gated behind the `flux` feature) for relational invariants between values — preconditions, postconditions, indexed types. See [Verification stack](#verification-stack).
  3. **Deductive proof via Creusot** (gated behind the `creusot` feature, Linux-only) for properties that need induction or quantifiers SMT can't discharge alone — closure idempotence, mass conservation, the Dirac collapse equation.
  4. **Runtime law helpers** in `sim_core::laws` *only* for what the layers above genuinely cannot reach (currently: ergodic claims like variation reachability, and meta-properties like viability non-prescriptivity).
- **No global fitness.** Selection enters only through relational `Viability` or external `Observer` scoring. A scalar fitness slot would silently break the design — flag it rather than adding one.

## Verification stack

| Layer | Feature flag | Toolchain | Platform | What it proves |
|---|---|---|---|---|
| Newtypes | always on | stable | all | Pointwise scalar constraints (NaN, range) |
| Const-generic shapes | always on | stable | all | Compile-time dimension correctness for fixed-dim spaces — see [`RealVector<const N>`](src/measure.rs). [`nalgebra`](https://docs.rs/nalgebra) is the upgrade path when matrix-shape arithmetic is needed at the substrate layer |
| Flux refinements | `flux` | Flux's pinned nightly (currently `nightly-2025-11-25`); see `.github/workflows/rust.yml` `flux-verify` job | Linux only in CI (Z3 + source build); the proc-macro shim is no-op on stable everywhere else | SMT-decidable relational invariants |
| Creusot proofs | `creusot` | Creusot v0.11.0's pinned nightly (currently `nightly-2026-02-27`); see `creusot-verify` CI job | Linux/WSL only (opam + Why3 + Creusot from source) | Inductive / quantifier-heavy laws via Why3 + SMT or Coq |
| `sim_core::laws` | always on | stable | all | Property-test fallback for laws above the static frontier |

A clean `cargo build` and `cargo test` use only the newtype layer; downstream users opt into the heavier layers per crate. CI runs all of them — feature-gated jobs run on the platforms each tool supports.

### Creusot and the LGPL isolation rule

`creusot-contracts` is **LGPL-2.1-or-later**. The crate's other deps are permissive (MIT / Apache-2.0 / BSD-2/3 / etc.). Mixing LGPL into a proprietary release binary triggers LGPL §6's relinking obligation — workable but not free. The scaffold avoids the question entirely by guaranteeing **the `creusot` feature is never enabled in any distributed artifact**:

1. The feature is `default = []` in `sim-core/Cargo.toml`.
2. The release CI job (`.github/workflows/rust.yml` → `release`) builds with default features only — `cargo build --release`. No `--features creusot`, no `--all-features`.
3. `deny.toml` carries narrow per-crate exceptions for `creusot-contracts` and `creusot-contracts-proc` so the LGPL allowance is impossible to broaden by accident — adding any other LGPL crate would still fail `cargo deny check`.
4. Every Creusot annotation lives behind `#[cfg_attr(feature = "creusot", creusot_contracts::macros::ensures(...))]` so the attribute is stripped (along with all references to `creusot_contracts`) on a default build.

If you ever need to make the scaffold available under a different license arrangement that does include Creusot at runtime, revisit this rule deliberately. Do **not** flip the `default` features just to make `cargo flux --features flux,creusot` more ergonomic — that would silently re-introduce the LGPL surface into release builds.
