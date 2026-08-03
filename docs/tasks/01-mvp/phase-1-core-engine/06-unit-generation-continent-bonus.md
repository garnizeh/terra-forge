# 1.6 — Unit generation & Continent control bonus

## Goal

Implement Compile-phase unit generation — driven by territories controlled plus a bonus for any fully-controlled `Continent` — with `Compiling` territories excluded from both counts, per RFC-001 UC3 step 1.

## Context

- RFC-001 UC3: "Player reviews newly generated autonomous units and deploys them... Generation is driven by the number of territories controlled plus a bonus for any fully-controlled `Continent`... territories still in the 'Compiling' status do not contribute."
- RFC-001 §7: `Continent.control_bonus` "applies to a player's Compile phase only when every child `Territory` shares that player's `owner_id` and none are `Compiling`."

## Acceptance criteria

- [ ] A pure function computes a player's unit generation for a given Compile phase from: the count of their `Active` (non-`Compiling`) territories, plus `control_bonus` for every `Continent` where 100% of child territories are owned by that player **and** none of them are `Compiling`.
- [ ] `Compiling` territories are excluded from both the base territory count and from continent-full-control checks — a continent with even one `Compiling` territory (even if player-owned) withholds its bonus entirely.
- [ ] The base generation-per-territory-count function is implemented only once the open question below is resolved.
- [ ] A per-Compile-phase unit pool is maintained internally, initialized each Compile phase with the freshly-generated amount; both `Deploy` (task 1.7) and `AccelerateCompile` (task 1.8) consume from this same pool. The pool resets when entering a new Compile phase and is exhausted (cannot go negative) — consumption attempts that exceed the remaining balance are rejected.
- [ ] Generated units are returned as a value the caller can spend (via `Deploy`, task 1.7's validation, or `AccelerateCompile`, task 1.8) — this task does not itself mutate `Territory.unit_count`, and does not perform consumption; consumption is validation+atomic deduction, owned by the validation task (1.7).
- [ ] Unit tests: a player controlling zero territories (should not panic, even if this state is expected to be transient — a player at zero territories is presumably already eliminated per task 1.9); a player controlling one full continent; a player controlling a continent minus one `Compiling` territory (bonus correctly withheld); a player controlling multiple continents (bonuses correctly summed).

## Open questions

- **The base generation-per-territory-count formula is unspecified.** RFC-001 says generation is "driven by the number of territories controlled" without giving the actual function. Other area-control games commonly use something like a floor-division with a minimum (e.g. `max(3, territories / 3)`), but nothing in this project's RFC corpus commits to a specific formula or constant. This needs a decision before the function can be fully implemented.

## Out of scope

`AccelerateCompile`'s consumption of these generated units (task 1.8, which depends on this task's output). The `Deploy` action that actually spends generated units on a territory (task 1.7).

## Depends on

1.2, 1.4.

## Status

Not started
**PR:** (none yet)
