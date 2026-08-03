# 1.2 — Domain entities & protocol types

## Goal

Model [RFC-001 §7](../../../rfc/rfc-0001_terra-forge_product_design_specification.md#7-domain-model--entity-relationships)'s Game-State Entities in Rust — `Map`, `Continent`, `Territory`, `GameAction`, `Faction` — as the shared vocabulary every later Phase 1 task operates on, plus the `protocol` sub-module [RFC-002 §5.1](../../../rfc/rfc-0002_terra-forge_high_level_architecture.md#51-core-engine-module-a) recommends for the `serde`+`ts-rs`-derived wire types that will eventually cross the Engine boundary (Phase 3/4).

## Context

- RFC-001 §7 gives the full attribute list for every Game-State Entity — this task implements them attribute-for-attribute, not a reinterpretation.
- RFC-003 §8: "Game-State Entities — ship in full, unchanged from RFC-001 §7... trimming it would mean trimming the game." No MVP simplification applies here.
- RFC-003 §6: MVP identity is an ephemeral, per-match session — there is no `User` entity. Anywhere RFC-001 says `owner_id`/`actor_id` references `User`, this task must instead use an opaque player identifier with no account semantics.
- RFC-002 §5.1: "a `protocol` sub-module (or sibling crate) holding the `serde`+`ts-rs`-derived types that cross the Engine boundary... separately from the pure rule-evaluation code."

## Acceptance criteria

- [x] `Territory { id, continent_id, owner_id: Option<PlayerId>, faction: Option<Faction>, unit_count: u32, status: TerritoryStatus, adjacent_territory_ids: Vec<TerritoryId> }` matches RFC-001 §7 attribute-for-attribute.
- [x] `Continent { id, map_id, name, control_bonus: u32 }` matches RFC-001 §7.
- [x] `Map { id, match_id, size_config }` — `size_config` is a forward dependency: this task defines the field but leaves its concrete type/shape pending. Task 1.10 (hand-authored presets) must approve the actual type before task 1.2 is done; alternatively, use a generic or enum variant large enough to accommodate every preset that task 1.10 will author, leaving the type fully specified in this task rather than deferred.
- [x] `GameAction { action_type: ActionType, actor_id: PlayerId, source_territory_id: TerritoryId, target_territory_id: Option<TerritoryId>, unit_count: u32 }`, with `ActionType = Deploy | Attack | Fortify | Concede | AccelerateCompile`, per RFC-001 §7.
- [x] `Faction` enum: `SiliconSwarm | SporeColony | CryoArchitects | MagmaForge`, per RFC-001 §2.
- [x] `TerritoryStatus` enum: `Active | Compiling`.
- [x] `PlayerId` is an opaque newtype — not RFC-001's `User` entity, no account/profile semantics — consistent with RFC-003 §6's ephemeral MVP identity model. The Engine must not assume a `User` row exists anywhere.
- [x] A `protocol` sub-module (`engine::protocol`, or a sibling crate if a concrete reason emerges during implementation to split it out — the sub-module form is the simpler default per RFC-002's own phrasing) holds every type above with `#[derive(Serialize, Deserialize)]` and `#[derive(TS)]`, kept separate from modules containing rule-evaluation logic.
- [x] Every ID type (`TerritoryId`, `ContinentId`, `MapId`, `PlayerId`) is a distinct newtype, not a bare `String`/`u64` — prevents accidentally passing, say, a `ContinentId` where a `TerritoryId` is expected. (A fifth newtype, `MatchId`, was also added for `Map.match_id` — see Notes.)
- [x] Unit tests cover `serde` (de)serialization round-trips for every protocol type.

## Notes (decisions made during implementation)

- **ID representation:** ID newtypes wrap a minimal in-house `Ulid` value type (`engine::protocol::Ulid`) rather than the `ulid` crate. The `ulid` crate unconditionally depends on `web-time` (a wall-clock polyfill) on `wasm32-unknown-unknown` regardless of feature flags, which would violate the Engine's zero-wall-clock-time constraint via its dependency tree even though no generation function is ever called. The in-house type only parses/formats/compares the canonical 26-character Crockford Base32 form — the Engine never generates IDs, so no RNG or clock is needed.
- **`Map.match_id`:** RFC-001 requires this field but the task's original ID-type list only named four newtypes and `MatchInstance` is out of scope here. Resolved by adding a fifth opaque newtype, `MatchId`, without modeling `MatchInstance` itself.
- **`MapSizeConfig`:** implemented as an empty placeholder struct, doc-commented as reserved for a post-MVP procedural generator; task 1.10's hand-authored presets don't consume it.

## Out of scope

`MatchInstance` and `EventLog` as full Game-State Entities per RFC-001 §7 — these belong to RFC-003 §8's scope but are Phase 3/platform-server concerns, not Engine state. The Engine needs only the slice of in-memory `MatchInstance` state (current turn/phase, player list, eliminated status) that pure phase-transition functions require; task 1.4 owns defining that minimal slice. Persisted MatchInstance fields like `lobby_id`, `turn_deadline`, `state_blob`, and the entire `EventLog` entity belong to Phase 3. Faction modifier data (task 1.3). Combat resolution logic (task 1.5).

## Depends on

1.1.

## Status

Not started
**PR:** (none yet)
