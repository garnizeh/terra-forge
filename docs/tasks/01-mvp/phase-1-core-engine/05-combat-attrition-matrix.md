# 1.5 — Deterministic combat attrition matrix

## Goal

Implement `resolve_combat`, arguably the single most load-bearing function in the whole product — [RFC-003 §3](../../../rfc/rfc-0003_terra-forge_mvp_definition.md#3-success-criteria) point 5 ships it "exactly as specified... this is the mechanic the entire product concept rests on, so it ships whole, not trimmed." Attacker force size, defender force size, and the defending territory's defense bonus combine to a single guaranteed outcome, with zero randomness.

## Context

- RFC-001 UC3: "Combat resolution is fully deterministic — no dice or random rolls. The server's engine computes the result from a fixed attrition matrix: attacking force size, defending force size, and the defending territory's inherent defense bonus combine to yield a single guaranteed outcome. Given the same inputs, the server and every connected client's local engine always compute the identical result."
- CLAUDE.md: "No RNG in combat. Combat is resolved via a deterministic attrition matrix (attacker size, defender size, territory defense bonus → guaranteed outcome)."
- RFC-002 §5.1: combat resolution takes modifiers as a parameter; the resolver must be able to look up each combatant's faction-specific modifiers, so it needs both the attacker and defender's Faction identities, not just unit counts.
- RFC-001 §8 / RFC-003 §10: Determinism NFR — "in full... non-negotiable at any scope."

## Acceptance criteria

- [ ] `resolve_combat(attacker_units: u32, attacker_faction: Faction, defender_units: u32, defender_faction: Faction, defense_bonus: u32, modifiers: &FactionModifiers) -> CombatOutcome` is a pure function: identical inputs always produce identical outputs — no `HashMap` iteration order, no floating point, nothing that could differ between a native and a wasm build. The `defense_bonus: u32` input range is `0..=u32::MAX`; the function must not panic or produce undefined behavior on any valid u32 value.
- [ ] `CombatOutcome` captures at minimum: attacker units remaining, defender units remaining, and whether the territory changed hands (feeds task 1.8's Compiling-status trigger).
- [ ] The attrition formula itself is implemented only once the open question below is resolved — this task cannot be meaningfully completed before that.
- [ ] The resolver actually looks up `modifiers[attacker_faction]` and `modifiers[defender_faction]` to select each combatant's faction-specific modifiers (task 1.3's seam), even though the uniform table currently makes both lookups identical — proves the seam reaches this function and is threaded correctly for asymmetry to drop in later as a data change.
- [ ] Unit tests cover named boundary cases: equal forces, attacker vastly outnumbered, defender vastly outnumbered, the minimum legal attack size (interacts with task 1.7's minimum-garrison open question), and a full territory capture (defender reduced to 0).
- [ ] Baseline determinism is asserted here too (same inputs, called twice in the same test run, produce identical outputs) — the broader property-based sweep is task 1.11's job.

## Open questions

- **The attrition matrix formula itself is not specified anywhere in RFC-001/002/003.** Every document names it ("the deterministic attrition matrix") and states its inputs and the determinism requirement, but none give the actual mapping from (attacker size, defender size, defense bonus) to outcome. This is the single biggest open gap blocking Phase 1 and needs an explicit decision before this task can be implemented. Candidate approaches, not yet chosen between:
  - **(a) Closed-form proportional formula** — e.g. each side's losses are proportional to the opposing force's effective strength; attacker takes the territory if their remaining force is greater than zero after the exchange.
  - **(b) Threshold/ratio rule** — e.g. the attacker needs at least an N:1 ratio over (defender + defense bonus) to take the territory outright; otherwise both sides take losses proportional to the smaller force, and nothing changes hands.
  - **(c) Sequential-exchange-resolved-to-closed-form** — conceptually similar to classic *Risk* unit-for-unit exchange, but computed directly as a formula instead of resolved dice-roll-by-dice-roll, to stay a pure function.

  Given how central this mechanic is — RFC-003 explicitly ships it "whole, not trimmed" and CLAUDE.md lists it first among "Distinctive mechanics" — this decision likely warrants its own ADR once made, not just an inline implementation choice.

- **The defending territory's "inherent defense bonus" has no data source in the current domain model.** RFC-001 §7's `Territory` attributes (`id`, `continent_id`, `owner_id`, `faction`, `unit_count`, `status`, `adjacent_territory_ids` — see task 1.2) include no defense-bonus field, so as currently modeled it isn't per-territory stored data. Candidates: a flat constant applied uniformly to every defending territory (simplest, consistent with MVP factions being purely visual); or a new field added to `Territory`/map data in tasks 1.2/1.10 (e.g. terrain-derived, for later expansion). This needs a decision alongside the formula above, since choosing the second option changes task 1.2's `Territory` shape retroactively.

## Out of scope

Invoking this function from an `Attack` `GameAction` (task 1.7 owns the validation + call site). Any UI/animation concerns.

## Depends on

1.2, 1.3.

## Status

Not started
**PR:** (none yet)
