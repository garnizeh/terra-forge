# ADR-0001: Deterministic combat attrition formula and defense-bonus source

## Status

Accepted

## Context

RFC-001 UC3 and RFC-002 §5.1 both name "the deterministic attrition matrix" as the mechanic combat resolution rests on, and CLAUDE.md lists "no RNG in combat" first among the game's distinctive mechanics — but none of the RFCs specify the actual mapping from (attacker size, defender size, defense bonus) to outcome, nor where the defending territory's "inherent defense bonus" comes from. [Task 1.5](../tasks/01-mvp/phase-1-core-engine/05-combat-attrition-matrix.md) flags both as open questions blocking implementation of `resolve_combat`, and calls out that the formula choice "likely warrants its own ADR."

Three candidate formula shapes were considered:

- **(a) Closed-form proportional** — both sides always take losses proportional to the opposing effective strength; capture only happens when the defender's real units hit zero, independent of any ratio requirement.
- **(b) Threshold/ratio rule** — the attacker needs an N:1 ratio over (defender units + defense bonus) to capture outright; below that ratio nothing changes hands. Introduces a new, RFC-unspecified tunable (the ratio N).
- **(c) Sequential-exchange closed form** — a Risk-style unit-for-unit trade computed directly as an integer formula (no dice-by-dice resolution). The defense bonus acts as extra defender strength consumed before real defender units start taking losses.

For the defense bonus's data source, two options were considered:
- A flat constant applied to every defending territory.
- A new `defense_bonus` field on `Territory`, which would retroactively change task 1.2's already-merged struct and task 1.10's map presets, for a terrain-variation mechanic no RFC currently describes.

## Decision

We will implement `resolve_combat` using **(c), the sequential-exchange closed form**, with the defense bonus applied as a **flat constant** (not a per-`Territory` field).

Formula, given `attacker_units: u32`, `defender_units: u32`, `defense_bonus: u32`:

```
defender_effective = defender_units.saturating_add(defense_bonus)
exchanged          = min(attacker_units, defender_effective)

attacker_remaining          = attacker_units - exchanged            // exchanged <= attacker_units, no underflow
defender_effective_remaining = defender_effective - exchanged        // exchanged <= defender_effective, no underflow

if defender_effective_remaining == 0 {
    // captured: attacker_units >= defender_units + defense_bonus
    defender_remaining = 0
    territory_captured = true
} else {
    // held
    defender_remaining = defender_effective_remaining.saturating_sub(defense_bonus)
    territory_captured = false
}
```

All arithmetic uses `saturating_add`/`saturating_sub` and `min`, so no branch can panic or overflow for any `u32` input, including `defense_bonus` at `u32::MAX`.

The defense bonus itself is a single named constant (e.g. `DEFAULT_DEFENSE_BONUS: u32`) applied uniformly to every defending territory, consistent with MVP factions being purely visual and terrain not existing as a mechanic yet. Its exact magnitude is a tuning value, not part of this architectural decision, and can change later without a new ADR as long as it stays a flat constant.

`resolve_combat` still takes `attacker_faction`, `defender_faction`, and `&FactionModifiers` per task 1.3's seam, and looks up both sides' `ModifierSet`s — but since Phase 1 ships only the uniform no-op table, this has no effect on the formula above yet. That plumbing is not part of this decision; it is already settled by task 1.3.

Options (a) and (b) were rejected: (a) has no RFC-grounded rule for the loss-proportion split independent of the capture condition, which would mean inventing math beyond what was actually decided; (b) introduces an unspecified ratio constant with no precedent in the RFCs. (c) was chosen because it has no free-floating tunable beyond the (separately-decided) flat defense bonus, its safety against overflow/underflow is trivial to verify, and it is the most direct reading of "attrition matrix" given the game's stated *Risk* lineage (CLAUDE.md: "in the vein of *Risk*").

## Consequences

- The formula is easy to reason about and trivially safe across the full `u32` range — no float, no `HashMap` iteration, satisfies the determinism and no-panic requirements in task 1.5's acceptance criteria directly.
- **Accepted quirk:** when `defender_units <= attacker_units < defender_units + defense_bonus`, the exchange can drive `defender_remaining` to `0` while `territory_captured` stays `false` — the defending garrison is wiped out but the position holds because the (abstract) defense bonus wasn't fully overcome. This is a legitimate consequence of modeling the bonus as extra effective strength rather than real units, not a bug. It means a territory can exist with `unit_count == 0` and `status == Active`, undefeated. Later tasks (1.7 validation, 1.9 win/loss) should be aware zero-unit-but-uncaptured is a reachable state, not treat `unit_count == 0` as synonymous with "no longer owned by this player."
- **Accepted quirk — "Pyrrhic capture":** when `attacker_units == defender_units + defense_bonus` exactly, the exchange consumes the entire attacking force in the same step that wins the territory: `attacker_remaining == 0` and `territory_captured == true`. The conquered territory enters `Compiling` with `unit_count == 0`. This is not special-cased away — `resolve_combat` stays a pure function of its inputs, so this outcome falls straight out of the formula rather than needing extra logic. Whether the game's *rules* ever let a player reach this exact input combination (e.g. via a minimum-garrison/minimum-attack-size constraint) is deliberately left to task 1.7's minimum-garrison open question, not decided here — see the cross-reference in both task docs.
- Choosing a flat constant means defense bonus cannot vary by terrain/territory at MVP; if a future phase wants terrain-derived defense, that requires revisiting this ADR (a new ADR superseding this one) and reopening `Territory`'s shape — deferred deliberately, not foreclosed.
- The specific defense-bonus magnitude is free to retune via ordinary code review without touching this ADR, since only "flat constant vs. per-territory field" is the architectural commitment here.
