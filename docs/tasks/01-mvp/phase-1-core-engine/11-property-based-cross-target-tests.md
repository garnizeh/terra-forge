# 1.11 — Property-based & cross-target test suite

## Goal

Close out Phase 1 with the property-based and cross-compilation testing CLAUDE.md and RFC-002 §5.1 specifically call for — `proptest` where edge cases are likeliest to hide (the attrition matrix, map generation), and a `wasm-pack test --headless` pass to catch non-portable code before it ever reaches the browser in Phase 4.

## Context

- CLAUDE.md: "`cargo test` for the combat matrix and phase transitions; `proptest` specifically for the attrition matrix and map generation (where edge cases hide); `wasm-pack test --headless` to catch non-portable code... before it reaches the browser."
- RFC-002 §5.1: identical framing, naming "asymmetric unit counts, disconnected map graphs" as the kind of edge case `proptest` exists to catch.
- RFC-003 §11 (Definition of Done): "`engine` crate: combat matrix, phase transitions, Re-compile Delay, map generation — full `cargo test` + `proptest` coverage, compiles to both a native `rlib` and `wasm32-unknown-unknown`." "`wasm-pack test --headless` passes (no accidentally non-portable code)."

## Acceptance criteria

- [ ] `proptest` coverage for `resolve_combat` (task 1.5): generates attacker/defender unit counts over the range `[0, u32::MAX]` and defense-bonus values over `[0, u32::MAX]`, ensuring each generated triple is tested twice (determinism property: same inputs → same output both times). The property must hold for **all** generated values: 0, 1, and all values up to the maximum; no generated input ever panics or produces undefined behavior.
- [ ] `proptest` coverage for spawn determinism (task 1.10's seeded PRNG): generated seeds (for each supported player count) never produce non-deterministic spawn assignments (the determinism property: same seed + same preset → identical player-to-territory mapping, checked twice per generated seed). Hand-authored preset topology validation (connectivity, symmetric adjacency) is owned by task 1.10's unit tests, not property-based tests here — this task only verifies spawn determinism.
- [ ] `wasm-pack test --headless` runs the crate's test suite (or an appropriate wasm-target subset) in a headless browser environment and passes — surfacing any accidental non-portable code (e.g. a stray `std::time::Instant`) introduced anywhere in tasks 1.1–1.10.
- [ ] A native-vs-wasm determinism check: the same fixed sequence of `GameAction`s (and forced skip/timeout-equivalent transitions from task 1.4) replayed against both a native build and a wasm build must: (a) progress to a terminal match state (per task 1.9: Victory, or the last active player remains and the match closes), not halt or diverge mid-match, and (b) produce byte-identical (or identically-hashed) final Engine state at that terminal point. This is Phase 1's own instance of the cross-cutting determinism check RFC-002 §11 and RFC-003 §3 describe as validating "the entire architectural bet" — scoped here to native-vs-wasm; the fuller server-vs-client version is Phase 3's job (see `docs/tasks/01-mvp/overview.md`, task 3.11).
- [ ] `engine-ci` (from task 1.1) is updated to actually run all of the above, not just the sanity-check skeleton from that task.

## Open questions

None — this task consumes and hardens what tasks 1.1–1.10 already built rather than introducing new mechanics.

## Out of scope

Server-vs-client determinism (Phase 3's two-headless-client harness). Any test infrastructure for the CLI (Phase 2) or the server/frontend (Phase 3/4).

## Depends on

1.1, 1.5, 1.6, 1.7, 1.8, 1.9, 1.10 — effectively all prior Phase 1 tasks; this is the hardening/wrap-up task for the phase.

## Status

Not started
**PR:** (none yet)
