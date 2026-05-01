# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository shape

Cargo workspace (`resolver = "3"`) with two members:

- **`sim-core/`** — the substrate-agnostic emergent-systems scaffold. All real code lives here. See `sim-core/CLAUDE.md` for the crate-level spec, architecture, and conventions; do not duplicate that content in this file.
- **`sim-main/`** — currently a Hello-World binary. A driver/entry-point lives here once the scaffold has something to drive.

## Authoritative spec lives outside the source tree

`docs/notes/emergent_systems_mathematical_treatment.md` is the source of truth that `sim-core` is a one-to-one Rust translation of, and `docs/notes/lliterature_review.md` (note the typo) is the prior-art synthesis behind it. Both are **gitignored** (`.gitignore` excludes `docs/notes` and `docs/drafts`), so they exist only on the local filesystem — treat them as load-bearing references rather than expecting them in remote clones. When code cites a section like "§5.2.1", it points into the math doc.

## Workspace-level commands

```
cargo build                           # build all members
cargo test                            # run all tests across the workspace
cargo build -p sim-core               # build a single crate
cargo test  -p sim-core               # test a single crate
cargo test  -p sim-core step_composes # run a single test by name
cargo doc --workspace --open          # render rustdoc for everything
cargo clippy --workspace --all-targets
cargo fmt --all
```

For deeper sim-core development, work from `sim-core/` and follow `sim-core/CLAUDE.md`.
