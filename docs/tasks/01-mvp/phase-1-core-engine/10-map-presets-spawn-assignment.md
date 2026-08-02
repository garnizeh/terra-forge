# 1.10 — PRNG-seeded map presets & spawn assignment

## Goal

Author 2–3 hand-authored map presets (RFC-003 §9) stored as ordinary `Map`/`Territory`/`Continent` instances (task 1.2's types), and implement seeded spawn-point assignment so map generation is deterministic and drop-in-replaceable by a future procedural generator without reworking how the rest of the Engine consumes a map.

## Context

- RFC-003 §9: "a small, fixed set of hand-authored presets (e.g. 2-3 layouts sized for 2-4 players)... the presets must be stored as ordinary serialized `Map`/`Territory`/`Continent` instances (the Engine's normal in-memory representation), so a future PRNG-seeded generator producing the same structure is a drop-in addition, not a rework of how the Engine consumes a map. Spawn-point assignment within a preset still goes through the seeded PRNG per RFC-001 §8's determinism NFR regardless of how the layout itself was produced."
- RFC-001 §8: PRNG Determinism NFR — "Any non-deterministic-seeming behavior (map generation, spawn point assignment) must be driven by a single PRNG seed set once by the server per `MatchInstance`... so that 'random' sequences are bit-identical across server and clients."

## Acceptance criteria

- [ ] 2–3 hand-authored map presets exist as static/constructible `Map` instances (with their `Continent`s and `Territory`s, including `adjacent_territory_ids` graphs and `Continent.control_bonus` values). **Every supported player count (2–4 players) must have at least one preset** where all starting territories are distinct (non-overlapping) and reachable from each player's spawn.
- [ ] Preset layouts are drafted and presented for review before being treated as final — this is original content the RFCs deliberately leave to implementation rather than a value to invent silently and ship unreviewed (the normal `start-task` plan-review step covers this).
- [ ] The Engine receives an immutable `seed` parameter (a u64 or similar) per `MatchInstance` (provided by Phase 3) that drives spawn-point assignment deterministically: the same seed + same preset always produces the same player-to-territory spawn mapping.
- [ ] A single shared, seeded PRNG (initialized with the match's seed) drives spawn-point assignment — which player starts in which territory(ies) — for a given preset, deterministically: identical seed + identical preset = identical assignment every time.
- [ ] The PRNG implementation is chosen specifically for cross-platform determinism: it must produce bit-identical output sequences on native and `wasm32-unknown-unknown` builds from the same seed — an explicit, non-platform-dependent algorithm, not anything relying on OS randomness or float operations that could differ by target.
- [ ] Territory adjacency graphs are validated as symmetric (if A is adjacent to B, B is adjacent to A) and fully connected (no territory isolated from the rest of the map) for every preset — an unvalidated broken map would silently make task 1.7's `Fortify` contiguity requirement unreachable for affected territories.
- [ ] Tests: for each supported player count, verify that spawn assignments are deterministic (same seed → same assignment); use at least a documented set of test seeds (e.g. seeds 0, 1, 42) to verify multiple distinct assignments exist per preset; adjacency symmetry and connectivity hold for every preset (this doubles as part of task 1.11's property-based coverage for map topology validation).

## Open questions

None blocking implementation directly, but flagged: the presets' actual territory count, layout, and continent groupings are genuinely new creative content this task produces, not something derived from the RFCs — treat the acceptance criterion above about presenting drafts for review as load-bearing, not a formality.

## Out of scope

A procedural/algorithmic map generator — explicitly deferred per RFC-003 §9; this task only guarantees the *data shape* is generator-compatible later. Map-selection UI (Phase 4). Lobby-time preset choice (Phase 3).

## Depends on

1.2.

## Status

Not started
**PR:** (none yet)
