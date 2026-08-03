# 1.7 — GameAction validation & legality

## Goal

Implement adjacency and phase-legality validation for every `GameAction` variant (`Deploy | Attack | Fortify | Concede | AccelerateCompile`) as pure Engine-level functions — the rules a move must satisfy to be legal at all, independent of *who* is allowed to submit it (session/identity authorization is a Phase 3 server concern layered on top of this).

## Context

- RFC-002 §5.1: Engine responsibilities include "adjacency and legality validation for every `GameAction` variant."
- RFC-001 §7: `GameAction` "Validated by the Engine against `MatchInstance.current_phase` and territory adjacency before being accepted; `Concede`... is valid in any phase, from any active participant, exactly once per match. `AccelerateCompile`... is valid only during its actor's Compile phase, targets a `Compiling` territory they own, and consumes `unit_count` units from that turn's newly-generated pool rather than depositing them."
- CLAUDE.md's two authorization tiers (turn-gated vs. participant-gated): this task implements the *legality* half of that split. The *identity* half (is this really that player's session) is explicitly Phase 3's job — the server re-validates every message; channel admission never authorizes an action on its own.

## Acceptance criteria

- [ ] **`Deploy`**: legal only during the actor's own Compile phase; target territory must be owned by the actor and `Active` (a `Compiling` territory cannot be a deploy target — it generates and holds nothing until conversion completes); `unit_count` must not exceed that Compile phase's generated pool (task 1.6).
- [ ] **`Attack`**: legal only during the actor's own Execute phase; `source_territory_id` must be owned by the actor and `Active` (RFC-001 UC3: a `Compiling` territory "cannot be used as a source for an attack"); `target_territory_id` must appear in the source's `adjacent_territory_ids` and must not be owned by the actor; `unit_count` must not exceed the source's `unit_count` minus any minimum-garrison requirement (see open question).
- [ ] **`Fortify`**: legal only during the actor's own Optimize phase; source and target both owned by the actor and both `Active`; connected via a path of the actor's own contiguous `Active` territories (RFC-001 UC3: "connected, contiguous territories they control" — not merely direct adjacency); at most one `Fortify` per Optimize phase.
- [ ] **`Concede`**: legal in any phase, from any active participant, usable exactly once per match per actor (an already-conceded or already-eliminated actor cannot submit it again).
- [ ] **`AccelerateCompile`**: legal only during the actor's own Compile phase; `target_territory_id` must be `Compiling` and owned by the actor; `unit_count` must equal a fixed cost (defined below, not in task 1.8) and is drawn from that Compile phase's freshly-generated pool (task 1.6), never from any territory's existing `unit_count`.
- [ ] Every variant's validation returns a specific, distinguishable rejection reason on failure (wrong phase, not adjacent, not owned, insufficient units, target not `Compiling`, etc.) rather than a bare boolean — Phase 3's server will want to relay *why* an action was rejected, and Phase 4's client will want to pre-validate with the same specificity for optimistic UI.
- [ ] Unit tests cover the legal case and every named illegal case, per variant, listed above.

## Open questions

- **`AccelerateCompile` fixed unit cost.** RFC-001 UC3 states the owner "commits a fixed number... of that turn's freshly-generated units" without naming the number. This task uses that cost to validate the `unit_count` parameter, so the value must be defined (as an open question here or in an approved ADR) before/during this task's implementation, not later in task 1.8 (to avoid a dependency cycle). See task 1.8's own open question for the same issue flagged in the Re-compile Delay context.
- **Minimum garrison / unit_count floor.** Neither RFC-001 nor RFC-002 states whether an `Attack` or `Fortify` may move *all* units out of a source territory (leaving `unit_count = 0`), or whether at least one unit must remain behind. This project's own design docs don't state a Risk-style "always leave one behind" rule, so it isn't safe to assume one silently — needs a decision before this task's `Attack`/`Fortify` unit_count bound can be finalized. This decision also determines whether a **"Pyrrhic capture"** (`attacker_units` exactly equal to `defender_units + defense_bonus`, leaving `attacker_remaining == 0` on a successful capture — see [task 1.5](05-combat-attrition-matrix.md#open-questions) and [ADR-0001](../../../adr/0001-deterministic-combat-attrition-formula.md)) is ever reachable through legal play, or only ever produced by non-minimal attacks that happen to land exactly on the boundary.

## Out of scope

Session/actor-identity authorization — confirming the `actor_id` genuinely belongs to the connected player is Phase 3's job, layered on top of this task's pure legality checks. Actually mutating state as a result of a valid action (owned by whichever transition function applies: 1.5 for `Attack` resolution, 1.6/1.8 for generation/acceleration, 1.9 for `Concede`).

## Depends on

1.2, 1.4.

## Status

Not started
**PR:** (none yet)
