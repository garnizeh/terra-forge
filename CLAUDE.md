# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project status

**Design phase closing; MVP implementation starting.** The initial documentation set (RFC-001 product spec, RFC-002 architecture, RFC-003 MVP scope) is complete and the open questions in it have been resolved — the planning work is done, and the next work in this repository is **writing the MVP**, beginning at Phase 1 of the build order below (the headless Core Engine).

Practically, that means:
- **The design questions are settled.** Don't reopen or re-derive the toolchain, architecture, or scope decisions — they're recorded in the RFCs. If something genuinely needs to change, amend the relevant RFC rather than diverging from it silently in code.
- **Nothing has been scaffolded yet.** No source, build system, or tests exist; there is nothing to build, lint, or run. The toolchain *choices* are settled (see "Technology decisions"), but no workspace, package layout, or dependency version is committed — don't assume any exists until you've checked, and don't invent commands that aren't there.
- **Scope discipline is the active constraint.** RFC-003 is the operative document for what to build now; see "MVP scope" below.

The design documents live in:
- [docs/idea/01_first_idea.md](docs/idea/01_first_idea.md) — original concept doc. **Historical/superseded**, kept for provenance; where it disagrees with the RFCs, the RFCs win. Read it for the *why*, not the *what*.
- [docs/rfc/rfc-0001_terra-forge_product_design_specification.md](docs/rfc/rfc-0001_terra-forge_product_design_specification.md) — product spec: 17 user journeys, full domain model, NFRs. Describes the **complete long-term product**, not what ships first.
- [docs/rfc/rfc-0002_terra-forge_high_level_architecture.md](docs/rfc/rfc-0002_terra-forge_high_level_architecture.md) — technical architecture: technology per component, protocols, data/cache layer, deployment. Also **long-term**. Read before assuming any toolchain, framework, or infra — it records decisions already made so they don't get re-derived or re-asked.
- [docs/rfc/rfc-0003_terra-forge_mvp_definition.md](docs/rfc/rfc-0003_terra-forge_mvp_definition.md) — **MVP scope: read this first before writing any code.** RFC-001/002 describe everything Terra-Forge is meant to become; RFC-003 draws the line for what actually ships first and marks every use case, entity, NFR, and technology as MVP or deferred, with reasons.
- [docs/process/dev-lifecycle.md](docs/process/dev-lifecycle.md) — how work is planned and shipped (task docs, branches/PRs, ADRs, testing, the never-invent rule). See "Development workflow" below for the summary.

## What Terra-Forge is

A deterministic, web-native multiplayer strategy game (area-control, in the vein of *Risk*/*War*) with a sci-fi "terraforming AI factions" theme, built as a learning/development sandbox emphasizing architectural elegance.

Distinctive mechanics to keep in mind when implementing game logic:
- **No RNG in combat.** Combat is resolved via a deterministic attrition matrix (attacker size, defender size, territory defense bonus → guaranteed outcome).
- **Turn structure** has three phases per player: Compile (draft units) → Execute (attack) → Optimize (single fortify move between contiguous owned territories).
- **The "Re-compile Delay":** capturing a territory doesn't grant immediate control — it enters a `Compiling` state during which it generates no units, doesn't count toward its `Continent` bonus, can't be an attack source, and confers zero defense. It resolves either by **waiting** (completes free at the start of the owner's next Compile phase) or by **committing units** from that turn's freshly-generated pool during a Compile phase (`AccelerateCompile`), which consumes them rather than depositing them.
- **Units are the only currency.** There is deliberately no second abstract "resource" type — the idea doc's older wording about "expending extra resources" is superseded by RFC-001 UC3.
- Any shuffling/randomness that does occur (map gen, spawn points) must go through a shared, seeded PRNG so server and clients stay in sync.

## Target architecture (isomorphic core)

Three strictly separated modules — **do not blur these boundaries** when writing code:

1. **Core Engine (shared, "the Brain")** — pure game logic in Rust: domain entities (`Map`, `Territory`, `Player`, `Unit`), rule validation, deterministic state transitions. **Zero I/O, zero DB, zero UI/network dependencies, and no wall-clock time** (turn *deadlines* are a server concern; the Engine only knows phases). Compiles to a native `rlib` (linked into the server) and to `wasm32-unknown-unknown` from the *same source* — this dual-target compilation is the whole point, so engine code must stay platform-agnostic.
2. **Platform Server (backend)** — owns sessions, lobby lifecycle, WebSocket/HTTP I/O, turn timers, and is the sole state authority. Feeds validated player intents into the native Engine, persists resulting events, broadcasts them.
3. **Frontend Application (client)** — "dumb" visual client. Uses the Wasm-compiled Engine locally to validate/predict moves instantly and render optimistic feedback, then reconciles with server-authoritative events. Handles input translation, Canvas2D rendering, and out-of-game UI.

**State sync model:** Event Sourcing, not full-state broadcast. The server emits an immutable, ordered `EventLog`; since server and every client run the identical compiled Engine, replaying the same event sequence produces bit-identical state everywhere. The same ledger backs reconnection, crash recovery, and (post-MVP) replay/dispute review.

**Frontend state rule:** game state is **not** duplicated into a JS store — the Wasm Engine instance *is* the state, read through a subscription hook. Zustand holds UI-only state (modals, form drafts, connection banners). Mirroring Engine state into JS recreates exactly the two-sources-of-truth bug class determinism is meant to prevent. The canvas renderer runs its own `requestAnimationFrame` loop outside React's render cycle.

## Technology decisions (RFC-002 §3 — already settled, don't re-derive)

Rust + Axum on Tokio (server links the Engine as an in-process crate — no FFI, no serialization boundary) · PostgreSQL via `sqlx` (compile-time-checked SQL, no ORM) · ULID primary keys stored as native `UUID` · `wasm-pack`/`wasm-bindgen` for the Wasm target · Vite + React + TypeScript SPA (no SSR) · Canvas2D custom renderer (not WebGL — `isPointInPath` gives exact hit-testing, but pre-filter with a broad-phase spatial index) · Zustand · React Router · `ts-rs` to generate TS types from Rust (one canonical schema) · JSON over WSS for now · `tracing` for structured logs · Docker + Compose.

Deferred infra (post-MVP): Redis, multi-instance topology, Kubernetes, S3 cold storage, Prometheus/Grafana.

## MVP scope (RFC-003 — the operative constraint for current work)

**MVP = roadmap Phases 1–4.** The bar: two people play a complete match against each other in a browser, over the network, and server + client Engine states stay bit-identical.

**In scope:** private link-shared lobbies (2–4 human players), the full three-phase turn loop with the Re-compile Delay and the deterministic attrition matrix, reconnection mid-match, turn-timer auto-skip with forfeit escalation, concession, win/loss resolution.

**Explicitly out of scope — don't build these yet:** accounts/OTP/OAuth login, `PlayerProfile`/MMR, matchmaking discovery or public lobbies, spectator mode, in-match chat, tournaments, leaderboards, moderation/reports, replay viewer UI, AI bots, faction mechanical asymmetry, Redis, multi-instance scaling, Kubernetes.

**MVP identity is ephemeral, not accounts:** a display name plus a server-issued session token scoped to a single `match_id`, with a 4-hour sliding TTL refreshed on each accepted action. No `User` row, no cross-match identity. Starting a new match issues a new token.

**MVP maps:** 2–3 hand-authored presets, stored as ordinary serialized `Map`/`Territory`/`Continent` instances so a PRNG-seeded generator drops in later without reworking how maps are consumed. Spawn-point assignment still goes through the seeded PRNG.

## Domain model reference

**Game-state entities (ship in full at MVP)** — owned and mutated exclusively by the Engine, never written directly by client or API code outside the turn-resolution path:
- `MatchInstance` — holds the PRNG `seed`, `current_turn`, `current_phase`, `status`; 1:many `EventLog`, 1:1 `Map`. Its `state_blob` is a periodic snapshot cache, **never** the source of truth.
- `EventLog` — the append-only ledger (`sequence_number`, `event_type`: `PlayerAction` | `SystemEvent`, `action_payload`). This *is* the source of truth; state is always reconstructable by replaying it from sequence 0.
- `Map` → 1:many `Continent` → 1:many `Territory` (`owner_id`, `faction`, `unit_count`, `status`: `Active` | `Compiling`, `adjacent_territory_ids`).
- `GameAction` — `Deploy` | `Attack` | `Fortify` | `Concede` | `AccelerateCompile`.
- `Faction` — enum (`SiliconSwarm`, `SporeColony`, `CryoArchitects`, `MagmaForge`), purely visual at MVP. Rule functions should still take a `FactionModifiers` lookup parameter (shipped as a uniform no-op table) so asymmetry later is a data change, not a signature change.

**Platform entities:** MVP has only a trimmed `Lobby` (host session, map preset, optional turn timer, status). `User`, `PlayerProfile`, `MatchHistory`, `Report`, `Tournament`, and `ChatMessage` are defined in RFC-001 §7 but **have no MVP tables** — don't create them until the use case that needs them ships.

**Authorization has two tiers** — don't collapse them into one check:
- **Turn-gated** (`Deploy`, `Attack`, `Fortify`, `AccelerateCompile`): only from the player whose turn it is, only in a phase that permits the action.
- **Participant-gated** (`Concede`, and later chat): from any active participant regardless of turn order — a player must be able to surrender while waiting — but never from a spectator, eliminated player, or non-participant.

Both are re-validated against authoritative server state on **every** message; channel admission never authorizes an action.

## Build order

Later phases assume earlier ones are solid — don't jump ahead (e.g. don't wire up networking before the headless engine has test coverage for combat and state transitions):

0. ~~Design documentation (RFC-001, RFC-002, RFC-003)~~ — **complete**
1. Headless Core Engine (pure logic, unit-tested, no visuals) — **← current phase**
2. CLI/local text prototype to playtest rules and the Re-compile Delay
3. Multiplayer backbone (server, WebSockets, two synchronized headless clients via Event Sourcing)
4. Full frontend rendering (Wasm in-browser, visual map/animations)
   — **← MVP ships here (RFC-003)**
5. Platform expansion (accounts, DB persistence for platform entities, matchmaking, spectator mode)
6. AI factions/bots (headless engine enables fast simulation for MCTS/minimax-style bots)

## Testing expectations

- **Engine:** `cargo test` for the combat matrix and phase transitions; `proptest` specifically for the attrition matrix and map generation (where edge cases hide); `wasm-pack test --headless` to catch non-portable code (e.g. a stray `std::time::Instant`) before it reaches the browser.
- **Server:** `cargo test` for service logic; integration tests via `testcontainers-rs` against a real ephemeral Postgres — not a mocked DB, given how state-sensitive this system is.
- **Frontend:** Vitest + React Testing Library; Playwright for the React↔Wasm↔Canvas seam that unit tests can't cover.
- **Cross-cutting:** the determinism check — server and client Engine states compared byte-for-byte (or by deterministic hash) after a full match. This is the test that validates the entire architectural bet.

## Development workflow

Full detail in [docs/process/dev-lifecycle.md](docs/process/dev-lifecycle.md). Summary:

1. **Plan first.** Work is grouped by milestone (`docs/tasks/<NN>-<slug>/`, e.g. the current `01-mvp` covering build-order Phases 1–4) and, inside that, by build-order phase. Each milestone has a high-level `overview.md`; each task within a phase gets its own file. Each task maps to exactly **one pull request**, with explicit, testable acceptance criteria. Use the `plan-tasks` skill.
2. **Execute one task at a time.** Branch → present the implementation plan for review and wait for approval → develop with tests as you go → check acceptance criteria → **ask before opening the PR**, even once criteria are met. Use the `start-task` skill.
3. **Architecture changes get an ADR.** Any new or revised architectural decision is recorded in `docs/adr/` (Nygard format: Status/Context/Decision/Consequences) before or alongside the implementation that depends on it; amend the relevant RFC too if the decision changes what it states. Use the `adr` skill.
4. **Test everything that can be tested** — see "Testing expectations" above; a task's acceptance criteria should make the required coverage explicit.
5. **Never invent.** Nothing gets built, named, or architected beyond what an RFC, an approved task doc, or an approved ADR already covers. Hit a gap or ambiguity → stop, present the options with tradeoffs, and wait for a decision.
6. **Check library currency via Context7.** Before introducing any new dependency (Cargo crate, npm package, GitHub Action, CLI tool) — or writing code against an existing one's API — use the Context7 MCP tool to confirm current versions and docs instead of relying on training data, which goes stale. This project has already hit real drift this way (e.g. the wasm-pack installer's upstream GitHub org moved in 2025).
