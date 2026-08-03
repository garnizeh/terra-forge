# 1.8 — Compiling status & Re-compile Delay resolution

## Goal

Implement the Re-compile Delay itself — CLAUDE.md's signature mechanic — covering both resolution paths: waiting it out for free at the next Compile phase, and `AccelerateCompile` consuming freshly-generated units to finish it immediately.

## Context

- CLAUDE.md: "The Re-compile Delay: capturing a territory doesn't grant immediate control — it enters a `Compiling` state during which it generates no units, doesn't count toward its `Continent` bonus, can't be an attack source, and confers zero defense. It resolves either by waiting (completes free at the start of the owner's next Compile phase) or by committing units from that turn's freshly-generated pool during a Compile phase (`AccelerateCompile`), which consumes them rather than depositing them."
- RFC-001 UC3 step 4 gives the full journey text for this mechanic, including "Units are the only currency in this design — there is no separate abstract 'resource.' Committed units are consumed by the conversion (they do not join the territory's `unit_count`), so acceleration is a genuine trade-off against board presence, not a free action."
- RFC-003 §3 point 5: ships "exactly as specified... not trimmed," alongside the attrition matrix, as the mechanic the whole product concept rests on.

## Acceptance criteria

- [x] On territory capture (an `Attack` outcome from task 1.5 that reduces the defender to 0 and transfers ownership), the territory's `status` transitions to `Compiling` and its `faction` does **not** yet update to the new owner's — RFC-001 UC3: faction updates "once conversion completes."
- [x] While `Compiling`, a territory: generates no units in its owner's Compile phase (already enforced by task 1.6 excluding it from the count), does not count toward its `Continent`'s control bonus (already enforced by task 1.6), cannot be used as an `Attack` source (already enforced by task 1.7), and confers zero defense bonus if attacked — a `Compiling` territory's effective defense bonus passed into task 1.5's `resolve_combat` is always 0, regardless of whatever the base defense-bonus source turns out to be.
- [x] **Wait path**: a pure transition, invoked at the start of the owning player's next Compile phase (called from task 1.4's phase-advance logic), completes the conversion automatically at no unit cost — `status` becomes `Active`, `faction` updates to the owner's.
- [x] **Accelerate path**: `AccelerateCompile` (pre-validated as legal by task 1.7) completes the conversion immediately within the same Compile phase it's submitted, consuming the fixed unit cost (see open question) from that Compile phase's freshly-generated pool (task 1.6) — those units are consumed, **not** added to the territory's `unit_count`.
- [x] Unit tests: capture → `Compiling` → automatic wait-path completion at the next Compile phase; capture → `Compiling` → `AccelerateCompile` same-turn completion; an integration-style test exercising a `Compiling` territory's exclusion from generation, continent-bonus, attack-source eligibility, and defense bonus all together, to catch a partial implementation of any one of the four.

## Open questions

- **`AccelerateCompile`'s fixed unit cost — resolved: 3 units, already decided in task 1.7 ([`legality::ACCELERATE_COMPILE_COST`](../../../../engine/src/legality.rs)) and reused here rather than redefined.** RFC-001 UC3 states the owner "commits a fixed number of that turn's freshly-generated units" without naming the number; task 1.7 needed the same value to validate `AccelerateCompile`'s `unit_count`, so it was settled there first.

## Out of scope

The `Attack` resolution that triggers a capture (task 1.5). Unit-generation math (task 1.6). `AccelerateCompile`'s legality checks beyond what task 1.7 already covers — this task assumes the action arrives pre-validated.

## Depends on

1.2, 1.6, 1.7.

## Status

Not started
**PR:** (none yet)
