# 1.4 — Turn phase state machine

## Goal

Implement the Compile → Execute → Optimize phase progression at the match level, plus turn advancement across players, with **no wall-clock time anywhere** — the Engine only ever knows "what phase is it," never "how long until the deadline," per CLAUDE.md and [RFC-002 §5.1](../../../rfc/rfc-0002_terra-forge_high_level_architecture.md#51-core-engine-module-a).

## Context

- CLAUDE.md: "Turn structure has three phases per player: Compile (draft units) → Execute (attack) → Optimize (single fortify move)."
- RFC-001 UC3 journey (Compile/Execute/Optimize steps 1-3).
- RFC-002 §5.1: "Explicitly not this crate's job: ... wall-clock time (turn timers are a Platform Server concern — the Engine only knows phases, never deadlines)." And on UC8 specifically: "the Platform Server decides *when* a deadline has passed, but the resulting skip/forfeit is applied through the same Engine transition functions as any other action — the Engine never learns that a clock was involved." This task must expose a phase-skip transition callable with no deadline/timer concept baked in, for Phase 3 to call later.

## Acceptance criteria

- [x] An internal Engine match-state representation tracks: `current_turn`, `current_phase` (`Compile | Execute | Optimize`), ordered list of active players, per-player **eliminated/released status** (a player is ineligible for actions once released per task 1.9), and per-Optimize-phase **Fortify-used flag** (whether a Fortify has already been submitted in this Optimize phase, since RFC-001 UC3 limits to "single fortify move" — this flag resets entering each new Optimize phase).
- [x] A pure transition function advances `Compile → Execute → Optimize → (next player's) Compile`, incrementing `current_turn` on wraparound to the first player; the Fortify-used flag resets when entering a new Optimize phase; no parameter or code path reads any clock.
- [x] Turn order is round-robin over the match's player list — **the exact ordering rule is an open question, see below.**
- [x] A phase-skip transition exists for each phase (`skip_compile` / `skip_execute` / `skip_optimize`, or one function parameterized by phase) that advances state exactly as if the active player took no action in that phase — callable directly by whatever later decides a timeout occurred (Phase 3), with the function itself taking no deadline/time input.
- [x] "Who is the active player" and "is the active player released/eliminated" are exposed as queryable state, for task 1.7's validation to consume.
- [x] Unit tests: normal progression through all three phases for one player; wraparound from the last player's Optimize back to the first player's Compile with `current_turn` incrementing; the skip-phase path for each of the three phases; Fortify-used flag set/reset across phase boundaries.

## Open questions

- **Turn order convention — resolved: lobby-join order.** RFC-001/002/003 establish that turns are per-player and sequential (Compile→Execute→Optimize, then the next player), but none specify *which* order the players go in. Two candidates were considered: (a) lobby-join order — simplest, matches how most area-control board games seat players, no extra dependency; (b) an order derived from `MatchInstance.seed`. Decided on (a): `MatchState::new` takes the player list in lobby-join order and round-robins over it as-is — no PRNG dependency for this piece.

## Notes (decisions made during implementation)

- **Module placement:** lives at `engine::match_state` (`MatchState`, `Phase`), not under `engine::protocol` — it's internal Engine state, not a `serde`/`ts-rs` wire type crossing the Engine boundary at MVP (that's Phase 3's `state_blob`/persistence concern per task 1.2's Out of scope note).
- **Released-status setter deliberately omitted.** This task's `MatchState` stores per-player `released` status and exposes `is_active_player_released`/`is_player_released` as read queries, but no method sets it to `true` — flipping that flag is task 1.9's `release_to_neutral` transition. Since 1.9 already depends on 1.4 and extends the same module, it adds the setter when it implements the release transition, rather than this task inventing that transition ahead of time.
- **`skip_compile`/`skip_execute`/`skip_optimize`** are thin wrappers around the same `advance()` transition (each `debug_assert_eq!`s the expected current phase first) — this task doesn't track "did the active player act" for Compile/Execute (only Optimize's Fortify-used flag is tracked), so a skip and a normal advance are mechanically identical at this layer; the distinct names exist purely so a future timeout handler (Phase 3) can name the phase it's skipping.

## Out of scope

Turn *timers*/deadlines (Phase 3, Platform Server). What triggers a skip to actually be called (Phase 3's timeout detection). Win/loss-driven early match termination (task 1.9 owns this, though it calls into this task's state to stop advancing a closed match).

## Depends on

1.2.

## Status

In review
**PR:** [#5](https://github.com/garnizeh/terra-forge/pull/5)
