# 1.3 — Faction modifiers seam

## Goal

Thread a `FactionModifiers` lookup parameter through the rule functions that need it (combat resolution, task 1.5, chiefly), per [RFC-001 §2](../../../rfc/rfc-0001_terra-forge_product_design_specification.md#2-factions--biomes)'s open question and [RFC-002 §5.1](../../../rfc/rfc-0002_terra-forge_high_level_architecture.md#51-core-engine-module-a)'s explicit instruction — shipped now as a uniform no-op table so every faction resolves identically, keeping "turn on asymmetry" a future data change rather than a signature change threaded through every call site later.

## Context

- RFC-001 §2: "the Core Engine's rule-evaluation functions should be designed from the start to accept a per-faction modifier lookup (defaulting to a uniform, no-op table) rather than hardcoding faction-agnostic math."
- RFC-002 §5.1: `fn resolve_combat(attacker, defender, modifiers: &FactionModifiers) -> Outcome`. "Ship it now as a uniform, no-op table — the point isn't to design the modifiers today, it's to make 'faction X gets a defense bonus in biome Y' a data change to that table later instead of a signature change threaded through every call site."
- CLAUDE.md: "Rule functions should still take a `FactionModifiers` lookup parameter (shipped as a uniform no-op table) so asymmetry later is a data change, not a signature change."
- Faction mechanical asymmetry itself remains an explicitly unresolved RFC-001 §2 open question with no assigned phase — this task builds the seam, not the mechanic.

## Acceptance criteria

- [x] `FactionModifiers` type defined as a lookup keyed by `Faction` (e.g. a `HashMap<Faction, ModifierSet>` or an array indexed by enum discriminant), with fields limited to exactly what task 1.5's combat resolution needs to read — nothing speculative beyond that.
- [x] A `FactionModifiers::uniform()` (or `Default`) constructor produces a table where every `Faction` maps to an identical, neutral `ModifierSet` — the only table Phase 1 ships.
- [~] Every rule function whose output could plausibly depend on faction (combat resolution at minimum) takes `&FactionModifiers` as an explicit parameter instead of hardcoding faction-agnostic math inline. **Not yet applicable:** no rule-evaluation function exists in the crate at this point in the build order (tasks 1.4–1.9 are all "Not started"); combat resolution itself is task 1.5, which depends back on this task's seam. See Notes.
- [~] A unit test asserts combat outcomes are bit-identical regardless of which `Faction` is passed in, when using the uniform table — proving the no-op table is genuinely a no-op rather than accidentally asymmetric. **Substituted for now:** `resolve_combat` doesn't exist yet, so the test instead asserts the uniform table itself returns an identical `ModifierSet` for every `Faction` — the same property, checked at the seam rather than through a combat call that can't exist yet. See Notes.

## Notes (scope decision made during implementation)

CLAUDE.md's build order is explicit that later phases assume earlier ones are solid — task 1.5 (combat resolution) has its own unresolved open questions (the attrition formula itself, and where a territory's defense bonus comes from) that this task must not preempt. So this task ships only what task 1.2's `Faction` enum makes possible today:

- `ModifierSet` is an empty struct (`engine::faction_modifiers::ModifierSet`) — no rule function reads a field from it yet, so no field is speculatively added.
- `FactionModifiers` is a lookup from every `Faction` to a `ModifierSet`, with a `uniform()` constructor (and matching `Default` impl) producing identical entries for all four factions.
- The bit-identical-outcome test is expressed as "every faction's looked-up `ModifierSet` is `==` the others" rather than a `resolve_combat` call, since that function doesn't exist yet.

AC #3 and #4 above are marked `[~]` (not fully applicable yet, not skipped) rather than `[x]`: task 1.5 is expected to close them for real by actually threading `&FactionModifiers` through `resolve_combat` and testing combat outcomes directly, per its own acceptance criteria which already reference "task 1.3's seam."

## Out of scope

Any actual per-faction modifier values. Any asymmetric-mechanic design — RFC-001 §2 explicitly defers this with no phase assigned; this task must not preempt that unresolved decision.

## Depends on

1.2 (needs the `Faction` enum).

## Status

Merged
**PR:** [#4](https://github.com/garnizeh/terra-forge/pull/4)
