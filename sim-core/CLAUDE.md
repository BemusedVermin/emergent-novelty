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

## Conventions

- **Edition 2024.** Single external dep: `rand_core = "0.10"`. Resist adding deps; the scaffold's value is in being substrate-agnostic and unopinionated.
- **`rand_core::Rng` everywhere** as `&mut R: Rng + ?Sized`, matching the math doc's stochastic-kernel signatures. The smoke test shows the `TryRng → Rng` blanket-impl path for tests.
- **Doc comments cite math-doc sections.** Maintain that convention when extending; it is how a future reader navigates from code back to the formalism.
- **Documented expectations, not type-level enforcement.** The math doc imposes laws (closure idempotence, variation reachability, viability non-prescriptivity, etc.) that Rust's type system cannot express. Document them in the rustdoc; if you need machine-checked versions, add property tests rather than fighting the type system.
- **No global fitness.** Selection enters only through relational `Viability` or external `Observer` scoring. A scalar fitness slot would silently break the design — flag it rather than adding one.
