# 1.1 — Rust workspace & `engine` crate scaffold

## Goal

Stand up the Cargo workspace and the `engine` crate itself — the empty vessel every other Phase 1 task fills in. It must build for both compilation targets from day one (native `rlib` for a future server to link, and `wasm32-unknown-unknown` via `wasm-pack`), per CLAUDE.md's "Target architecture" and [RFC-002 §5.1](../../../rfc/rfc-0002_terra-forge_high_level_architecture.md#51-core-engine-module-a), and carry a CI skeleton so every subsequent task is checked automatically.

## Context

- RFC-002 §3: Rust `std` (no `no_std`), `wasm-pack` + `wasm-bindgen` as the Wasm toolchain.
- RFC-002 §5.1: crate structure recommendation — a `protocol` sub-module or sibling crate for wire types (built in 1.2, not this task).
- RFC-002 §10: `engine-ci` = `cargo test` + `clippy` + `fmt` + `wasm-pack build` sanity check; release profile `strip = true, lto = true, opt-level = "z"` plus a `wasm-opt -Oz` pass.
- CLAUDE.md: the Engine "compiles to a native `rlib` (linked into the server) and to `wasm32-unknown-unknown` from the *same source*... engine code must stay platform-agnostic," with "zero I/O, zero DB, zero UI/network dependencies, and no wall-clock time."

## Acceptance criteria

- [ ] Cargo workspace created at the repo root with `engine` as a member crate.
- [ ] `engine` crate compiles natively (`cargo build`) with zero warnings.
- [ ] `engine` crate compiles to `wasm32-unknown-unknown` (via `wasm-pack build` or `cargo build --target wasm32-unknown-unknown`) with zero warnings.
- [ ] `engine`'s dependency tree contains nothing that pulls in `std::net`, filesystem I/O beyond pure computation needs, or wall-clock time (`std::time::Instant`/`SystemTime`) — checked by reviewing the actual dependency tree, not just by intent, since a transitive dependency can violate this silently.
- [ ] `Cargo.toml`'s release profile sets `strip = true`, `lto = true`, `opt-level = "z"` per RFC-002 §10 (the release *pipeline* that invokes `wasm-opt -Oz` is a later, deployment-adjacent concern — this task only owns the crate-level profile settings).
- [ ] `engine-ci` GitHub Actions workflow skeleton exists and passes on the near-empty crate: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`, and a `wasm-pack build` sanity check.
- [ ] A pinned Rust toolchain (`rust-toolchain.toml` or equivalent) ensures the wasm target is consistently available, not dependent on whatever's locally installed.
- [ ] A crate-level doc comment or README states the zero-I/O / zero-wall-clock-time constraint explicitly, so it's asserted up front rather than rediscovered after a violation.

## Out of scope

Any domain types (Territory, Map, GameAction, etc. — task 1.2). Any CI workflow beyond `engine-ci` (`server-ci`/`frontend-ci` have nothing to run against yet). Publishing or release automation.

## Depends on

None — first task in Phase 1.

## Status

In review
**PR:** [#2](https://github.com/garnizeh/terra-forge/pull/2)
