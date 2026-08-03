# 1.5 — Deterministic combat attrition matrix

## Goal

Implement `resolve_combat`, arguably the single most load-bearing function in the whole product — [RFC-003 §3](../../../rfc/rfc-0003_terra-forge_mvp_definition.md#3-success-criteria) point 5 ships it "exactly as specified... this is the mechanic the entire product concept rests on, so it ships whole, not trimmed." Attacker force size, defender force size, and the defending territory's defense bonus combine to a single guaranteed outcome, with zero randomness.

## Context

- RFC-001 UC3: "Combat resolution is fully deterministic — no dice or random rolls. The server's engine computes the result from a fixed attrition matrix: attacking force size, defending force size, and the defending territory's inherent defense bonus combine to yield a single guaranteed outcome. Given the same inputs, the server and every connected client's local engine always compute the identical result."
- CLAUDE.md: "No RNG in combat. Combat is resolved via a deterministic attrition matrix (attacker size, defender size, territory defense bonus → guaranteed outcome)."
- RFC-002 §5.1: combat resolution takes modifiers as a parameter; the resolver must be able to look up each combatant's faction-specific modifiers, so it needs both the attacker and defender's Faction identities, not just unit counts.
- RFC-001 §8 / RFC-003 §10: Determinism NFR — "in full... non-negotiable at any scope."

## Acceptance criteria

- [x] `resolve_combat(attacker_units: u32, attacker_faction: Faction, defender_units: u32, defender_faction: Faction, defense_bonus: u32, modifiers: &FactionModifiers) -> CombatOutcome` is a pure function: identical inputs always produce identical outputs — no `HashMap` iteration order, no floating point, nothing that could differ between a native and a wasm build. The `defense_bonus: u32` input range is `0..=u32::MAX`; the function must not panic or produce undefined behavior on any valid u32 value.
- [x] `CombatOutcome` captures at minimum: attacker units remaining, defender units remaining, and whether the territory changed hands (feeds task 1.8's Compiling-status trigger).
- [x] The attrition formula itself is implemented only once the open question below is resolved — this task cannot be meaningfully completed before that.
- [x] The resolver actually looks up `modifiers[attacker_faction]` and `modifiers[defender_faction]` to select each combatant's faction-specific modifiers (task 1.3's seam), even though the uniform table currently makes both lookups identical — proves the seam reaches this function and is threaded correctly for asymmetry to drop in later as a data change.
- [x] Unit tests cover named boundary cases: equal forces, attacker vastly outnumbered, defender vastly outnumbered, the minimum legal attack size (interacts with task 1.7's minimum-garrison open question), a full territory capture (defender reduced to 0), and a **Pyrrhic capture** (`attacker_units == defender_units + defense_bonus`, e.g. A=6/D=3/B=3 — territory captured with `attacker_remaining == 0`; see ADR-0001's Consequences).
- [x] Baseline determinism is asserted here too (same inputs, called twice in the same test run, produce identical outputs) — the broader property-based sweep is task 1.11's job.

## Open questions

- **The attrition matrix formula — resolved: sequential-exchange closed form, see [ADR-0001](../../../adr/0001-deterministic-combat-attrition-formula.md).** Every RFC names "the deterministic attrition matrix" and its inputs/determinism requirement, but none gave the actual mapping from (attacker size, defender size, defense bonus) to outcome. Three candidates were considered — (a) closed-form proportional, (b) threshold/ratio rule, (c) sequential-exchange closed form (Risk-style unit-for-unit trade, computed as a formula) — and (c) was chosen; ADR-0001 has the full formula and rationale.

- **The defending territory's "inherent defense bonus" data source — resolved: flat constant, see [ADR-0001](../../../adr/0001-deterministic-combat-attrition-formula.md).** RFC-001 §7's `Territory` attributes include no defense-bonus field. Decided against adding one (which would have retroactively changed task 1.2's merged `Territory` shape and task 1.10's map presets) in favor of a single constant applied uniformly to every defending territory, consistent with MVP factions being purely visual.

- **Pyrrhic capture (`attacker_units == defender_units + defense_bonus` exactly) is allowed, not special-cased.** The formula lets a successful capture consume the attacker's entire force (`attacker_remaining == 0`), leaving the newly `Compiling` territory with `unit_count == 0`. `resolve_combat` stays a pure function of its inputs and does not prevent this. Whether the game's *rules* ever let a player reach this exact input combination — e.g. via a minimum-garrison/minimum-attack-size floor — is deliberately left to task 1.7's minimum-garrison open question ([07-gameaction-validation-legality.md](07-gameaction-validation-legality.md)), not decided here. See ADR-0001's Consequences for the full discussion.

## Out of scope

Invoking this function from an `Attack` `GameAction` (task 1.7 owns the validation + call site). Any UI/animation concerns.

## Depends on

1.2, 1.3.

## Status

In review
**PR:** [#6](https://github.com/garnizeh/terra-forge/pull/6)
