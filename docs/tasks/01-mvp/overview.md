# MVP roadmap overview

High-level map of everything planned to reach MVP (CLAUDE.md build-order Phases 1–4, per [RFC-003 §2](../../rfc/rfc-0003_terra-forge_mvp_definition.md#2-guiding-principle-roadmap-phases-1-4-not-phase-5)). Phases 5–6 (accounts, matchmaking, spectator mode, AI bots) are explicitly post-MVP and out of scope — not listed here.

**What this document is:** a roadmap index — every anticipated task, one line each, so the shape of the remaining work is visible in one place.
**What it is not:** a task doc. No acceptance criteria, no dependency graph, no branch/PR live here — those get written per phase, in `phase-N-<slug>/` inside this same `01-mvp/` folder, close to when that phase's work actually starts (see [../README.md](../README.md)). Task lists below are a best-effort forecast grounded in the RFCs; expect them to be refined — split, merged, reordered — when a phase's detailed docs are actually planned via the `plan-tasks` skill. That refinement is normal, not scope creep, as long as it stays inside what the RFCs already describe.

Status legend: `[ ]` not started · `[~]` detailed task doc exists / in progress · `[x]` merged.

---

## Phase 1 — Headless Core Engine

**Goal:** the pure-Rust game engine (`engine` crate) — domain model, combat, turn/phase transitions, the Re-compile Delay, map/spawn generation — unit-tested, with zero I/O and zero wall-clock time, compiling unmodified to both a native `rlib` and `wasm32-unknown-unknown`. This is CLAUDE.md's **current phase**. Grounded in [RFC-002 §5.1](../../rfc/rfc-0002_terra-forge_high_level_architecture.md#51-core-engine-module-a) and [RFC-003 §7/§8](../../rfc/rfc-0003_terra-forge_mvp_definition.md#7-mvp-technical-scope-subset-of-rfc-002-3s-technology-decision-log).

Detailed task docs: [`phase-1-core-engine/`](phase-1-core-engine/). Several tasks carry an explicit **Open questions** section flagging design gaps the RFCs leave unspecified (the attrition matrix formula, turn order, unit-generation formula, `AccelerateCompile`'s unit cost, minimum garrison) — those need a decision before that specific task is executed; see each file.

| # | Task | Objective | Status |
|---|---|---|---|
| 1.1 | Rust workspace & `engine` crate scaffold | Cargo workspace; `engine` crate targeting native `rlib` + `wasm32-unknown-unknown`; `engine-ci` skeleton (`cargo test` + `clippy` + `fmt` + `wasm-pack build` sanity check). | [x] [doc](phase-1-core-engine/01-workspace-scaffold.md) |
| 1.2 | Domain entities & protocol types | `Map`, `Continent`, `Territory`, `GameAction`, `Faction` enum, per RFC-001 §7; a `protocol` submodule holding the `serde`+`ts-rs`-derived wire types separately from rule logic. | [x] [doc](phase-1-core-engine/02-domain-entities-protocol-types.md) |
| 1.3 | Faction modifiers seam | `FactionModifiers` lookup parameter threaded through rule functions, shipped as a uniform no-op table — the asymmetry extension point from RFC-001 §2 / RFC-002 §5.1, not asymmetry itself (still an open question, MVP stays visual-only). | [~] [doc](phase-1-core-engine/03-faction-modifiers-seam.md) |
| 1.4 | Turn phase state machine | Compile → Execute → Optimize transitions at the `MatchInstance` level; phases only, no deadlines (those are a Phase 3 server concern). | [~] [doc](phase-1-core-engine/04-turn-phase-state-machine.md) |
| 1.5 | Deterministic combat attrition matrix | `resolve_combat(attacker, defender, modifiers)` — attacker size, defender size, territory defense bonus → one guaranteed outcome, no RNG. | [~] [doc](phase-1-core-engine/05-combat-attrition-matrix.md) |
| 1.6 | Unit generation & Continent control bonus | Compile-phase unit generation from territory count plus full-`Continent` bonus; `Compiling` territories excluded from both. | [~] [doc](phase-1-core-engine/06-unit-generation-continent-bonus.md) |
| 1.7 | GameAction validation & legality | Adjacency and phase-legality checks for `Deploy`/`Attack`/`Fortify`/`Concede`/`AccelerateCompile`, as pure Engine-level functions (server-side re-authorization is a Phase 3 task). | [~] [doc](phase-1-core-engine/07-gameaction-validation-legality.md) |
| 1.8 | Compiling status & Re-compile Delay resolution | `Compiling` territory effects (no generation, no continent bonus, can't attack from, zero defense); wait-out-at-next-Compile path; `AccelerateCompile` consuming freshly-generated units rather than depositing them. | [~] [doc](phase-1-core-engine/08-compiling-status-recompile-delay.md) |
| 1.9 | Win/loss condition resolution | Victory (100% compiled control), Elimination, Concession, Forfeit-trigger — territories to neutral, garrisons retained, `MatchClosed` transition, per RFC-001 §6. | [~] [doc](phase-1-core-engine/09-win-loss-resolution.md) |
| 1.10 | PRNG-seeded map presets & spawn assignment | 2–3 hand-authored presets (RFC-003 §9) stored as ordinary `Map`/`Territory`/`Continent` instances; spawn-point assignment through the shared seeded PRNG. | [~] [doc](phase-1-core-engine/10-map-presets-spawn-assignment.md) |
| 1.11 | Property-based & cross-target test suite | `proptest` for the attrition matrix and map/spawn generation; `wasm-pack test --headless`; a native-vs-wasm determinism check on the same event sequence. | [~] [doc](phase-1-core-engine/11-property-based-cross-target-tests.md) |

---

## Phase 2 — CLI / local text prototype

**Goal:** a local, text-based harness on top of the `engine` crate to playtest the rules and the Re-compile Delay before any networking exists. CLAUDE.md names this phase in one line and doesn't detail it further (RFC-002/003 don't cover a CLI component at all) — so this task list is our own scaffolding proposal, more provisional than the other phases, kept intentionally thin.

| # | Task | Objective |
|---|---|---|
| 2.1 | CLI harness scaffold | Binary crate depending on `engine`; local hotseat loop for 2–4 players in one process. |
| 2.2 | Text rendering of match state | ASCII/text view of the map, territory ownership/unit counts, current turn/phase. |
| 2.3 | Turn command interface | Stdin commands mapped to `GameAction`s, phase-appropriate prompts (Compile/Execute/Optimize/AccelerateCompile/Concede). |
| 2.4 | Win/loss reporting | Loop exits and reports the outcome on Victory/Elimination/Concession, exercising Phase 1's resolution logic end to end. |
| 2.5 | Manual playtest pass | Play full matches by hand; file/fix any Engine rule bugs the CLI surfaces that Phase 1's automated tests didn't catch. |

---

## Phase 3 — Multiplayer backbone

**Goal:** the Platform Server — sessions, lobby lifecycle, WebSocket I/O, turn timers, feeding validated actions into the native `engine`, persisting and broadcasting events — plus two synchronized headless clients proving Event Sourcing keeps them bit-identical (CLAUDE.md's Phase 3 milestone). Grounded in [RFC-002 §5.2](../../rfc/rfc-0002_terra-forge_high_level_architecture.md#52-platform-server-module-b), §6, §7, §12, trimmed to [RFC-003 §7](../../rfc/rfc-0003_terra-forge_mvp_definition.md#7-mvp-technical-scope-subset-of-rfc-002-3s-technology-decision-log) (single instance, no Redis, ephemeral session identity, no spectator channel).

| # | Task | Objective |
|---|---|---|
| 3.1 | Backend workspace scaffold | Axum+Tokio server crate depending on `engine` in-process (no FFI); `handlers → services → repositories` layering; `server-ci` skeleton. |
| 3.2 | Postgres schema & migrations | `sqlx migrate` for `lobby` (trimmed), `match_instance`, `event_log` only; ULID primary keys as native `UUID`. |
| 3.3 | Ephemeral session identity | Display-name-only join; server-issued `match_id`-scoped session token, 4-hour sliding TTL refreshed on every accepted action; no JWT/OTP/OAuth. |
| 3.4 | Lobby lifecycle | Create private lobby (map preset, player count, optional turn timer), invite-link join, host removes a not-yet-started participant, start match. |
| 3.5 | WebSocket participant channel | `extract::ws`, session-token handshake, `GameAction` intents in / `EventLog` entries out, JSON wire format. |
| 3.6 | Action authorization & Engine integration | Server-side re-validation of turn-gated vs. participant-gated actions on every message; validated actions applied to in-memory state and persisted to `event_log` before ack. |
| 3.7 | Reconnection via sequence-number replay | Client sends last known `sequence_number`; server replies with the delta (or a `state_blob` snapshot + delta for large gaps); single-instance recovery scope only. |
| 3.8 | Turn timer & AFK/forfeit escalation | Server-tracked deadline per `current_turn`/phase; `TurnTimedOut` `SystemEvent`; repeated timeouts escalate to forfeit via the same Engine transition as Concede. |
| 3.9 | Concession handling | `Concede` `GameAction`, participant-gated authorization, neutral-territory release per Phase 1's resolution logic. |
| 3.10 | Win/loss resolution & match closure | `MatchInstance.status → Closed`, sealed final `EventLog` sequence, per RFC-001 §6. |
| 3.11 | Two-headless-client determinism harness | Two synchronized headless clients replaying the same `EventLog` via Event Sourcing; byte-for-byte (or hashed) state comparison — the cross-cutting bet this whole architecture rests on. |
| 3.12 | Server integration test suite | `testcontainers-rs` ephemeral Postgres; real WS round-trips against an in-process Axum router, not a mocked DB. |

---

## Phase 4 — Full frontend rendering *(MVP ships here)*

**Goal:** the "dumb" visual client — Vite/React SPA with the Wasm-compiled Engine as the actual game-state source, Canvas2D board rendering, and the UI needed to play a full match in a browser. Grounded in [RFC-002 §5.3](../../rfc/rfc-0002_terra-forge_high_level_architecture.md#53-frontend-application-module-c) and [RFC-003 §11](../../rfc/rfc-0003_terra-forge_mvp_definition.md#11-definition-of-done).

| # | Task | Objective |
|---|---|---|
| 4.1 | Frontend workspace scaffold | Vite + React + TypeScript SPA; `frontend-ci` skeleton (`vitest` + `eslint` + `tsc --noEmit` + `vite build`). |
| 4.2 | Wasm Engine integration | `wasm-pack build --target web`, dynamic `import()`; a `useEngineState(selector)` subscription hook — the Wasm instance *is* the game state, never mirrored into Zustand. |
| 4.3 | Generated type wiring | `ts-rs`-generated TS interfaces for `GameAction` and event payloads, consumed by the networking layer — one canonical schema. |
| 4.4 | `WsClient` networking layer | Connect/reconnect with backoff, resume from last known `sequence_number`; `fetch`-based REST client for lobby CRUD. |
| 4.5 | Lobby creation/join UI | Map preset selection, turn-timer toggle, invite-link display/join, display-name entry per the MVP identity model. |
| 4.6 | Canvas2D board renderer | Own `requestAnimationFrame` loop outside React's render cycle; territory polygons; `isPointInPath` hit-testing behind a broad-phase spatial-index pre-filter. |
| 4.7 | Phase-appropriate action UI | Compile (deploy + `AccelerateCompile`), Execute (attack), Optimize (fortify) controls; Concede control. |
| 4.8 | Win/loss/concession end screen | Final territory count and the win/loss/concession/forfeit reason, per RFC-003 §11. |
| 4.9 | Zustand UI-only state | Modals, form drafts, connection-status banners — explicitly not game state (see CLAUDE.md's frontend state rule). |
| 4.10 | Frontend test suite | Vitest + React Testing Library component tests; Playwright golden-path E2E (create lobby → join → play a turn → see board update), including one deliberate mid-match reconnect. |
| 4.11 | End-to-end MVP acceptance pass | Two real browsers/machines, full match to completion, `docker-compose up` bringing up server + Postgres with no manual steps beyond `.env` — RFC-003 §11's Definition of Done, checked item by item. |

---

## Out of scope (post-MVP, not planned here)

Per [RFC-003 §5](../../rfc/rfc-0003_terra-forge_mvp_definition.md#5-deferred-to-post-mvp): accounts/OTP/OAuth (`User`, `PlayerProfile`, MMR), matchmaking discovery/public lobbies, spectator mode, in-match chat, tournaments/leaderboards, replay viewer UI, moderation/reports, AI bots, faction mechanical asymmetry, Redis, multi-instance scaling, Kubernetes. These become CLAUDE.md Phase 5/6 objectives with their own numbered milestone folder (e.g. `docs/tasks/02-<slug>/`) and `overview.md`, created when that work actually starts — not designed here.
