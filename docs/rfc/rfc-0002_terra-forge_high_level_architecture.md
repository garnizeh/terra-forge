# RFC-002: Terra-Forge High-Level Architecture

* **Author:** Principal Architecture & Product Team
* **Status:** Draft / Proposed
* **Subject:** Technical Architecture, Technology Selection, Component Responsibilities, and Communication/Data Design for Terra-Forge
* **Companion document:** [RFC-001](rfc-0001_terra-forge_product_design_specification.md) defines *what* the product does (use cases, domain model, NFRs). This RFC defines *how* it is built — technology choices, component boundaries, and the runtime topology that satisfies RFC-001's requirements.

---

## 1. Purpose & Scope

Like RFC-001, this document describes Terra-Forge's **complete long-term technical vision** — the architecture the platform is meant to grow into across all six roadmap phases (CLAUDE.md), not a minimum viable slice. MVP scoping is a separate, later exercise that will select a subset of what's described here.

Every technology choice below is a **recommendation with stated rationale**, not an irreversible decree — this is a Draft RFC specifically so choices can be challenged. Where a choice is genuinely contentious or low-confidence, it's called out explicitly as an open question (Section 13) rather than asserted as settled.

---

## 2. Architectural Principles (recap)

These carry over from CLAUDE.md and RFC-001 and constrain every decision in this document:

1. **Strict three-module separation** — Core Engine, Platform Server, Frontend Application. The Engine has zero I/O, zero DB, zero network dependencies, and compiles unmodified to both a native binary and WebAssembly.
2. **Server-authoritative, event-sourced state** — the server never broadcasts full state; it broadcasts an immutable, ordered `EventLog`. Both server and every client replay the identical compiled Engine over the identical events to reach bit-identical state.
3. **Determinism above all** — no source of nondeterminism (timing, floating point, hashmap iteration order, uninitialized memory) may leak into any Engine computation. This constrains technology choices inside the Engine crate more than anywhere else in the stack.
4. **Single source of truth for shared types** — wherever a data shape crosses the Engine boundary (client prediction, server authority, wire format), it should be defined once in Rust and mechanically propagated, not hand-duplicated in TypeScript.

---

## 3. Technology Decision Log

| Component | Technology | One-line rationale |
|---|---|---|
| Core Engine | Rust, `std` (no `no_std`) | Already mandated by RFC-001/CLAUDE.md for native+Wasm dual compilation. |
| Wasm build | `wasm-pack` + `wasm-bindgen` | De facto standard toolchain for Rust→Wasm with JS/TS bindings generation. |
| Backend language/framework | Rust + [Axum](https://github.com/tokio-rs/axum) on Tokio | Lets the server depend on the Engine as an in-process native crate — zero FFI, zero serialization boundary between "server" and "rules." Axum's `extract::ws` gives WebSocket support on the same stack as HTTP, no second framework needed. |
| Async runtime | Tokio | Axum's runtime; single async runtime for the whole backend avoids the complexity of mixing two. |
| Database | PostgreSQL | Relational domain model (Section 7 of RFC-001) with strong consistency needs (turn authorization, MMR updates) plus JSONB for flexible payloads (`action_payload`, `state_blob`). Mature, self-hostable, no vendor lock-in. |
| DB access layer | [`sqlx`](https://github.com/launchbadge/sqlx) (async, compile-time-checked queries) | No ORM "magic" — SQL stays SQL, queries are checked against the real schema at compile time. Fits the project's "architectural elegance, minimal abstraction" ethos better than Diesel (sync-first, macro-heavy) or SeaORM (full ORM layer). |
| ID generation | ULID (`ulid` crate), stored as native Postgres `UUID` columns | Every `id` in RFC-001's domain model is a ULID: 128-bit like UUIDv4 (same 16-byte storage, same B-tree index behavior), but lexicographically sortable by creation time and generatable independently on any server instance with no central coordination — both properties matter directly for this architecture (see Section 7). |
| Cache / real-time fan-out | Redis | Pub/Sub for cross-instance event broadcast, session/matchmaking-queue storage, rate limiting. Single well-understood piece of infra covering three needs instead of three separate systems. |
| Redis client | `redis-rs` + `deadpool-redis` | Standard async Redis client with connection pooling. |
| Frontend framework | Vite + React (SPA, no SSR) | User decision (see conversation). SSR's core value (crawlable/shareable content) doesn't apply to an authenticated, real-time game client; a plain SPA avoids all Wasm/SSR execution-boundary friction that a meta-framework like Next.js would introduce. |
| Frontend state management | [Zustand](https://github.com/pmndrs/zustand) | Minimal (~1kB), hook-based, no boilerplate/provider ceremony. Used only for **UI-layer** state — see Section 5.3 for why game state itself is *not* duplicated into a JS store. |
| Frontend routing | React Router | Standard SPA routing; no file-based router since there's no Next.js. |
| Game board rendering | Canvas2D (custom renderer, no library) | Turn-based territory board, not an action game — `CanvasRenderingContext2D.isPointInPath` gives free, exact hit-testing for territory polygons, which WebGL has no native equivalent for (color-picking or CPU-side point-in-polygon otherwise required). `isPointInPath` itself runs on the CPU, so hit-testing must still pre-filter candidates with a cheap broad-phase spatial index (a bounding-box or grid lookup) before the exact per-polygon check — calling it against every territory on every `mousemove` on a large map is the actual risk, not the technique itself. Revisit if that's insufficient, or if a future visual-effects requirement (Section 13) demands GPU shading. |
| Type sharing (Rust → TS) | [`ts-rs`](https://github.com/Aleph-Alpha/ts-rs) | Generates TypeScript interfaces directly from `#[derive(TS)]` Rust structs, for both Engine-boundary types and REST DTOs — one canonical schema, no hand-maintained duplicate types drifting from the source of truth. |
| Wire format (WS) | JSON now; `bincode`/`postcard` swappable later | JSON is debuggable and suffices at the NFR's stated scale (hundreds of matches). Because payload types are `serde`-derived Rust structs shared by both native server and Wasm client, switching serialization formats later is a one-line change, not a rewrite. |
| Auth | JWT (access + refresh) via `jsonwebtoken`; login via email OTP (`sha256` hash of a `getrandom`-sourced code, short TTL) or OAuth via `oauth2` crate — **no password storage anywhere in the system** | Stateless bearer tokens verify identically across any number of horizontally-scaled server instances without a shared session store. Matches RFC-001 Use Case 1's passwordless requirement; removes an entire class of credential-stuffing/password-breach risk by construction, not by policy. |
| Transactional email (OTP delivery) | Provider-agnostic (SMTP or a transactional email API — vendor left open, see Section 13) | Only channel through which a login code reaches the user; no other component needs it. |
| Containerization | Docker | Universal, cloud-agnostic packaging for Engine-linked server binary, Postgres, Redis, and the static frontend bundle. |
| Local orchestration | Docker Compose | Sufficient for single-node dev and the early roadmap phases (CLAUDE.md Phases 1–4). |
| Production orchestration | Kubernetes (target, not required early) | Only justified once the NFR's "hundreds of concurrent matches" scale is actually being approached — see Section 10. |
| CI/CD | GitHub Actions | Repository is already hosted on GitHub; no separate CI system to operate. |
| Observability | `tracing` (Rust structured logs) + Prometheus + Grafana | `tracing` is the de facto Rust standard and integrates with Tokio; Prometheus/Grafana are self-hostable, matching the no-vendor-lock stance. |
| Cold storage (closed-match event logs) | S3-compatible object storage (AWS S3, Cloudflare R2, or self-hosted MinIO — vendor left open) | Keeps Postgres sized for *active* data only; see Section 7 for the archival strategy and why it's not immediate-on-close. |

---

## 4. System Topology

```
                            ┌──────────────────────┐
                            │   Browser (Client)    │
                            │  Vite/React SPA        │
                            │  + Engine (Wasm)       │
                            └──────────┬────────────┘
                                       │  HTTPS (REST) + WSS (real-time)
                                       ▼
                            ┌──────────────────────┐
                            │  Reverse Proxy / LB   │   (TLS termination,
                            │  (e.g. nginx/Caddy)   │    match-aware routing —
                            └──────────┬────────────┘    see Section 9)
                                       │
                 ┌─────────────────────┼─────────────────────┐
                 ▼                     ▼                     ▼
        ┌──────────────┐     ┌──────────────┐       ┌──────────────┐
        │ Server        │     │ Server        │  ...  │ Server        │
        │ instance A    │     │ instance B    │       │ instance N    │
        │ (Axum+Engine) │     │ (Axum+Engine) │       │ (Axum+Engine) │
        └───────┬───────┘     └───────┬───────┘       └───────┬───────┘
                 │                     │                       │
                 └─────────────┬───────┴───────────┬───────────┘
                                ▼                   ▼
                       ┌───────────────┐   ┌───────────────┐
                       │  PostgreSQL    │   │     Redis      │
                       │ (durable state)│   │ (pub/sub, cache,│
                       │                │   │  matchmaking Q) │
                       └───────────────┘   └───────────────┘
```

The static frontend bundle (Vite build output) is served independently of the API — a CDN or static host, decoupled from the Axum server, which only ever serves REST/WS traffic. This keeps the "dumb visual client" boundary from CLAUDE.md honest at the deployment level too: the frontend has no server-side runtime of its own.

---

## 5. Component Deep-Dive

### 5.1 Core Engine (Module A)

* **Language:** Rust, compiled as a Cargo library crate (`engine`), targeting both a native `rlib` (linked into the server binary) and `wasm32-unknown-unknown` (via `wasm-pack`, consumed by the frontend).
* **Responsibilities:** domain entities (`Map`, `Territory`, `Player`, `Unit`), the combat attrition matrix, phase/turn-transition rules, adjacency and legality validation for every `GameAction` variant (including `AccelerateCompile`'s unit cost and `Concede`'s release of territories to neutral, per RFC-001 Use Cases 3 and 9), Compiling-status resolution, and PRNG-seeded map generation. Pure functions in, new immutable state out — see RFC-001 Section 7 for the entities this crate owns.
* **Explicitly not this crate's job:** anything involving `std::net`, `std::fs` beyond what's needed for pure computation, wall-clock time (turn timers are a Platform Server concern — the Engine only knows phases, never deadlines), or randomness sourced from anything but the injected PRNG seed. Note the division this implies for RFC-001 Use Case 8: the Platform Server decides *when* a deadline has passed, but the resulting skip/forfeit is applied through the same Engine transition functions as any other action — the Engine never learns that a clock was involved.
* **Internal structure recommendation:** a `protocol` sub-module (or sibling crate) holding the `serde`+`ts-rs`-derived types that cross the Engine boundary (`GameAction`, state-mutation event payloads) separately from the pure rule-evaluation code, so the wire-format types have one clear home.
* **Faction-asymmetry extension seam:** per RFC-001 §2's open question, combat and defense-bonus calculations should take a `FactionModifiers` lookup as an explicit parameter (`fn resolve_combat(attacker, defender, modifiers: &FactionModifiers) -> Outcome`) rather than computing purely from unit counts inline. Ship it now as a uniform, no-op table (every faction resolves identically) — the point isn't to design the modifiers today, it's to make "faction X gets a defense bonus in biome Y" a data change to that table later instead of a signature change threaded through every call site.
* **Testing:** `cargo test` for unit coverage of the combat matrix and phase transitions; `proptest` (property-based testing) specifically for the attrition matrix and map generation, since these are the two places where an edge case is most likely to hide (e.g., asymmetric unit counts, disconnected map graphs); `wasm-pack test --headless` to catch any accidental non-portable code (e.g., an accidentally-included `std::time::Instant` call) before it reaches the browser.

### 5.2 Platform Server (Module B)

* **Language/framework:** Rust, Axum on Tokio, depending on `engine` as a normal Cargo path dependency within the same workspace.
* **Responsibilities:** authentication and session issuance, lobby/matchmaking lifecycle, WebSocket connection management (player and spectator channels), turn-timer enforcement (Section 8's Timeout Determinism NFR from RFC-001), feeding validated `GameAction`s into the Engine, persisting the resulting events, and broadcasting them. Also owns everything **outside** the Engine's boundary that RFC-001 introduced: `ChatMessage` handling (never touches the Engine — see RFC-001's Chat/Game-State Isolation NFR), `Report` intake and moderator tooling, bot move computation (Use Case 5 — runs Engine simulations server-side via a background Tokio task pool, using `rayon` for CPU-parallel MCTS/minimax search so it doesn't block the async I/O runtime).
* **Match ownership model:** each in-progress `MatchInstance` has its authoritative in-memory state owned by exactly one server instance for the match's lifetime (see Section 9 for how that's assigned and routed to). Every accepted `GameAction` is applied to that in-memory state **and** durably appended to Postgres's `EventLog` before the client receives an acknowledgment — this ordering is what makes the Resilience & Recovery NFR (RFC-001 §8) actually hold under a mid-write crash.
* **Internal layering (recommended, not prescriptive):** `handlers` (Axum route/WS handlers, thin) → `services` (lobby, matchmaking, match-session, moderation — business logic) → `engine` calls + `repositories` (sqlx queries). Keeps the "is this a platform concern or a game-rule concern" question CLAUDE.md poses answerable by which layer code lives in.
* **Testing:** `cargo test` for service-layer unit tests; integration tests using `testcontainers-rs` to spin up ephemeral Postgres + Redis per test run, exercising real WS round-trips against an in-process Axum server (via `axum::Router` + a test client) rather than mocking the database — consistent with not trusting mocked-DB integration tests for a system this state-sensitive.

### 5.3 Frontend Application (Module C)

* **Framework/tooling:** Vite + React + TypeScript, built as a static SPA.
* **Responsibilities:** input translation, out-of-game UI (profile, lobby, leaderboard, tournament, chat panel, moderation-report form), and the render pipeline — all as described in CLAUDE.md's Module C.
* **State management split — the important architectural call here:**
    * **Game state** (territories, units, current phase — everything the Engine already tracks) is **not** duplicated into Zustand/Redux. The single Wasm `Engine` instance loaded into the page *is* the state; React components read from it through a thin subscription hook (e.g. `useEngineState(selector)`) that re-renders only on Engine-emitted change notifications. Mirroring Engine state into a second JS-side store would create exactly the two-sources-of-truth bug class RFC-001's determinism NFR is designed to prevent.
    * **UI-only state** (open modals, form drafts, connection-status banners, chat-input text) lives in Zustand, since it has no Engine equivalent and doesn't need replay/determinism guarantees.
* **Rendering:** a single component owns a `<canvas>` ref and runs its own `requestAnimationFrame` loop **outside** React's render cycle — it reads directly from the Engine each frame and paints imperatively. React itself never re-renders on a per-frame basis; it only mounts/unmounts the canvas and reacts to the same Engine-change notifications the rest of the UI uses. This is the one place where fighting a framework's declarative re-render cycle would cost real frame-time, so the code deliberately opts out of it.
* **Wasm integration:** `wasm-pack build --target web`, imported via a dynamic `import()` — Vite handles `.wasm` as a static asset natively for this target without needing a bundler plugin, keeping the build toolchain minimal.
* **Networking layer:** a `WsClient` wrapper handling connect/reconnect with backoff and resuming from the last known `EventLog` sequence number (implements RFC-001 Use Case 7 client-side); a thin `fetch`-based REST client for non-real-time calls (auth, profile, lobby CRUD, leaderboard, match history) — no HTTP client library, `fetch` is sufficient and keeps dependencies minimal.
* **Testing:** [Vitest](https://vitest.dev/) (native Vite pairing) + React Testing Library for component/unit tests; Playwright for end-to-end golden-path flows (log in → create lobby → play a turn → see board update) — the only reliable way to catch bugs in the React↔Wasm↔Canvas boundary, which unit tests alone won't cover. Note that passwordless login makes E2E setup non-trivial: the suite needs a way to obtain the OTP without a real inbox, so the server should expose a test-only hook (retrieving the current code for a whitelisted test address) that is compiled out of release builds via a Cargo feature flag — never a runtime config toggle, which would be one misconfiguration away from an authentication bypass in production.

---

## 6. Communication & Protocols

* **REST (HTTPS/JSON):** everything not latency-critical or per-turn — auth, profile CRUD, lobby creation/browsing, leaderboard queries, match history, tournament registration, report submission. Simple request/response; no reason to route this through WebSocket.
* **WebSocket (WSS):** two distinct channel types per `MatchInstance`:
    * **Participant channel** — bidirectional. Client sends `GameAction` intents and chat messages; server sends the resulting `EventLog` entries (both `PlayerAction` and `SystemEvent` per RFC-001's `EventLog.event_type` split) plus the chat stream.
    * **Spectator channel** — read-only by construction at the handler level (RFC-001's Spectator Isolation NFR): the server never wires a message-receive path for this channel type at all, so enforcement doesn't depend on the client behaving — or on a future maintainer remembering a rule. It carries the same event stream, plus the participant chat stream *only* when `Lobby.spectator_chat_visibility` is `ReadOnly`. This is precisely why RFC-001 defers spectator-authored chat to its own design pass: implementing it would mean giving this channel an inbound path, trading away a structural guarantee for a feature, and that trade deserves an explicit decision rather than being made implicitly.
* **Login flow (REST, precedes any WS activity):** the user submits their email; the server generates a short numeric/alphanumeric OTP, stores only its `sha256` hash with a short TTL (e.g. 10 minutes) in Postgres (or Redis, given the short lifetime — see Section 7), and emails it via the transactional email provider. The user submits the code back; the server compares hashes, and on match issues the JWT access/refresh pair. OAuth follows the provider's standard authorization-code flow instead, converging on the same JWT issuance step. Because the OTP is single-use and short-lived, it's hashed with plain `sha256` rather than a slow password-hashing KDF (`argon2`/`bcrypt`) — those exist to resist offline brute-forcing of long-lived secrets, which doesn't apply to a code that expires in minutes and is rate-limited (below).
* **OTP brute-force protection:** attempts to verify a code are rate-limited per email/IP via Redis counters (Section 8) — this is the mechanism actually protecting the login flow, since a short numeric code alone is guessable at high request rates without it.
* **Auth handshake (WS):** JWT access token passed as a WS subprotocol header (or short-lived query-param ticket exchanged right after HTTP upgrade, to avoid tokens in server logs) is validated before the connection is admitted to a match's broadcast group; every inbound `GameAction` is *additionally* re-checked against `MatchInstance.current_turn`'s actual active player server-side, never trusting channel admission alone (RFC-001's Security NFR).
* **Reconnection protocol (Use Case 7):** client sends its last known `sequence_number` on reconnect; server replies with either (a) the delta of `EventLog` entries since that point, or (b) if the gap is large, the latest periodic `state_blob` snapshot (Section 7) plus the smaller delta since that snapshot — avoiding a full from-turn-one replay for long matches.

---

## 7. Data Layer

PostgreSQL schema maps directly onto RFC-001 Section 7's entities, with these implementation notes:

* **Primary keys — ULID, everywhere:** every `id` column across every table (`user`, `lobby`, `match_instance`, `event_log`, `territory`, `match_history`, `report`, `chat_message`, etc.) is a ULID, generated application-side (via the `ulid` crate) rather than left to `gen_random_uuid()`/`SERIAL`. Stored as Postgres's native `UUID` type — same 16 bytes, same index performance as UUIDv4 — but two properties make it a better fit than either alternative for this system specifically:
    * **Sortable by creation time:** unlike UUIDv4 (fully random, causing B-tree index fragmentation and no useful ordering), a ULID's high bits encode a millisecond timestamp, so `ORDER BY id` and cursor-based pagination (leaderboard pages, match-history scrolling, `event_log` inspection) work directly off the primary key — no separate `created_at` index needed for the common case.
    * **Coordination-free across instances:** unlike an auto-increment integer (`SERIAL`), which requires a single sequence owner, any server instance in the horizontally-scaled fleet (Section 9) can mint IDs — including `event_log.id` under concurrent writes from different match-owning instances — without a round-trip to a central counter, and without leaking a guessable "how many rows exist" count the way sequential integers do.
    * On the wire and in the frontend, a ULID serializes as its 26-character Crockford Base32 string form (via `serde`, propagated to TypeScript as `string` through `ts-rs` — no special client-side ULID type needed, it's opaque to the frontend).
* **`event_log` table:** append-only, indexed on `(match_id, sequence_number)`; `action_payload` stored as `JSONB`. This is the highest-write-volume table in the system, and retaining every closed match's full log in Postgres indefinitely is not sustainable — but archiving is only safe once the *hot* window for a match has passed. Strategy: on `MatchInstance.status → Closed`, the row stays in Postgres through a retention window (a config value — see Section 13 — long enough to cover the flows that actually read a recently-closed match's log: the post-match summary (Use Case 10), casual replay (Use Case 11), and conduct reports (Use Case 12), which in practice arrive within days, not months, of a match ending). A background job then serializes the match's consolidated `event_log` (same `serde`-derived types as the live wire protocol — no third format to maintain) to the object store keyed by `match_id`, verifies the write, and only then deletes the Postgres rows. **Each archive object must embed a schema/format version header.** Archives are permanent while the wire format is explicitly expected to change (the JSON→`bincode` migration flagged in this same document), and an event log that can't be deserialized is a replay capability silently lost years later — the version tag is what lets a future reader pick the right decoder instead of guessing from the bytes. The Use Case 11 replay path checks Postgres first and transparently falls back to object storage on a miss — archival is invisible to the moderator/player triggering a replay. This table is also a candidate for partitioning by `match_id` range or time once volume within the retention window justifies it (Section 13).
* **`match_instance.state_blob`:** a `JSONB` snapshot of full Engine state, written periodically (e.g., every N turns or on phase boundaries, not every action) purely as a reconnection/recovery optimization — the `event_log` remains the actual source of truth; the snapshot is a cache of "replay events 0..K" that can always be regenerated by replaying from zero if it's ever lost or found inconsistent.
* **`otp_code` table (or Redis, see below):** `id`, `user_id` (nullable — a code can be requested before an account exists, for first-time signup), `email`, `code_hash`, `expires_at`, `consumed_at` (nullable), `attempt_count`. Given its lifetime is minutes, this arguably belongs in Redis (with native TTL expiry) rather than Postgres — listed here mainly to make explicit that **no `password_hash` column exists anywhere in the schema**; the `User` table has no password field at all, consistent with RFC-001's Use Case 1.
* **`chat_message` table:** intentionally a separate table with no foreign key relationship into `event_log` — only a soft correlation via `sequence_number_at_send`, per RFC-001's Chat/Game-State Isolation NFR.
* **Migrations:** `sqlx migrate`, plain versioned `.sql` files checked into the repo — no schema-as-code DSL, keeping the actual SQL inspectable.
* **Read scaling (future, not needed at launch):** leaderboard and match-history queries are read-heavy and tolerate slight staleness; a Postgres read replica is the natural first scaling lever if/when it's needed, before reaching for anything more exotic.

---

## 8. Caching & Real-Time Fan-Out (Redis)

Redis serves three distinct roles — worth keeping conceptually separate even though it's one piece of infrastructure:

1. **Cross-instance event fan-out (Pub/Sub):** when a server instance mutates a match it owns, it publishes the resulting event(s) to a Redis channel keyed by `match_id`. Every server instance holding a WebSocket connection interested in that match (a spectator connected to a *different* instance than the one owning the match) subscribes and relays. This is what lets spectator connections scale independently of which instance happens to own a given match — directly serving the "thousands of concurrent spectator connections" NFR.
2. **Session/matchmaking ephemeral state:** public-lobby matchmaking queue membership, rate-limit counters (chat messages, report submissions, login attempts) — short-lived data that doesn't belong in the durable Postgres store.
3. **Match-ownership registry:** see Section 9 — which server instance currently owns a given `MatchInstance`'s authoritative in-memory state.

Redis is not used as a queue for `GameAction` processing itself — actions are applied synchronously by the owning instance, so no consumer-group/ack complexity is needed there.

**Pub/Sub reliability — self-healing via sequence gap detection, not Redis Streams.** Plain Pub/Sub is fire-and-forget: a transient network hiccup between a relaying server instance and Redis silently drops whatever was published during the gap, and a spectator attached to that instance would desync with no signal that anything was missed. That risk is real and needs a mitigation — but the fix is to detect and self-heal, not to make the transport itself durable, because Postgres's `event_log` is *already* the durable, replayable ledger (Section 7); adding Redis Streams would stand up a second, parallel durability mechanism (with its own consumer-group offsets and trim/retention policy to manage) duplicating a problem that's already solved one layer down. Instead: every relayed event carries its `sequence_number`, and the relaying instance (or the client itself, symmetrically with Use Case 7's reconnection logic) checks for a gap against the last number it saw. On a detected gap, it issues the same delta-fetch-by-`sequence_number` request against Postgres that reconnection already uses — reusing existing machinery instead of introducing a new one. This keeps Pub/Sub simple and fast for the common case, while the rare miss is corrected within one round-trip rather than persisting as silent desync.

---

## 9. Horizontal Scaling & Match Affinity

Given the NFR's stated scale ("hundreds of concurrent matches," not millions), the simplest correct mechanism is preferred over a general leader-election system:

* When a `MatchInstance` starts, the initiating server instance claims ownership via a Redis key (`match:{id}:owner = {instance_id}`, set with a heartbeat-refreshed TTL).
* **Routing happens in the application layer, not the proxy.** An off-the-shelf reverse proxy (nginx/Caddy) cannot consult a Redis registry to make per-connection routing decisions without bolting on scripting (`ngx_http_lua`) or a custom module — so it doesn't try to. The load balancer distributes WS connections arbitrarily; whichever instance receives one looks up `match:{id}:owner` itself and then either **(a)** handles the connection directly if it is the owner, or **(b)** acts as a relay — forwarding the client's inbound actions to the owning instance and streaming that match's events back down from its Redis subscription (Section 8). Option (b) is exactly the path spectators already take, so this adds no mechanism that isn't needed anyway, and it keeps the proxy layer a dumb, replaceable component.
* Consistent hashing is deliberately not used: at "hundreds of matches" scale a registry lookup is cheap, and unlike a hash ring it doesn't reshuffle ownership of unrelated matches when an instance joins or leaves — which for stateful, long-lived match sessions is the property that actually matters.
* If an owning instance dies, its heartbeat lapses, the TTL expires, and — per the Resilience & Recovery NFR — a new instance can reclaim ownership and reconstruct the match's in-memory state purely from the latest `state_blob` snapshot + `EventLog` delta (the same mechanism Use Case 7 reconnection uses, just triggered by instance failure instead of client disconnect, which is a nice validation that the recovery path is exercised by more than one scenario).
* **Explicitly deferred:** true leader election (Raft/etcd-style) is unnecessary complexity at this scale and is called out as an open question (Section 13) to revisit only if match volume grows by an order of magnitude.

---

## 10. Deployment & Infrastructure

* **Containers:** one image for the Platform Server (Rust binary + Engine statically linked in), one build artifact for the static frontend bundle (no container needed if served from a CDN/object storage directly), standard upstream images for Postgres and Redis.
* **Local/early-phase environment:** Docker Compose bringing up server + Postgres + Redis, matching CLAUDE.md's Phase 1–4 scope (headless engine, CLI prototype, then multiplayer backbone) without requiring cluster infrastructure before it's earned.
* **Production target (Phase 5+):** Kubernetes, once there's an actual multi-instance deployment to orchestrate — a `Deployment` for the server (horizontally scaled per Section 9), `StatefulSet` or managed service for Postgres, managed or self-hosted Redis. Deliberately not committing to a specific cloud vendor here — that's an infra decision independent of this RFC's technology choices, and can be layered on top of the containerized artifacts without changing anything above this line.
* **CI/CD (GitHub Actions):** three independent workflows mirroring the module boundary — `engine-ci` (cargo test + clippy + fmt + `wasm-pack build` sanity check), `server-ci` (cargo test with `testcontainers`-backed integration tests), `frontend-ci` (vitest + eslint + `tsc --noEmit` + `vite build`). A release workflow builds and pushes the server's Docker image and the frontend's static bundle on tagged releases.
* **Wasm bundle size (Zero-Install Accessibility, RFC-001 §3):** the release build of the `engine` crate's Wasm target must not ship an unoptimized dev artifact. The release pipeline runs `wasm-pack build --release` (which already invokes `wasm-opt` from the Binaryen suite by default when available) with an explicit `wasm-opt -Oz` pass and `[profile.release] strip = true, lto = true, opt-level = "z"` set in the Engine crate's `Cargo.toml` — debug symbols and unused monomorphized code must not reach production. `frontend-ci` should assert a bundle-size budget (e.g., fail the build past a set `.wasm` size threshold) so a regression is caught in review, not discovered in production.
* **Observability:** `tracing` spans around request handling, Engine invocation, and DB queries, exported in a structured (JSON) format; Prometheus scraping Axum/Tokio metrics (request latency, active WS connections, match count); Grafana dashboards for the "hundreds of matches / thousands of spectators" NFR to actually be observable rather than assumed.

---

## 11. Testing Strategy Summary

| Layer | Tooling | What it catches |
|---|---|---|
| Engine | `cargo test`, `proptest`, `wasm-pack test` | Rule/combat-math correctness, Wasm-portability regressions |
| Server | `cargo test`, `testcontainers-rs` integration tests | Authorization gaps, event-sourcing/replay correctness, DB schema drift |
| Frontend | Vitest, React Testing Library, Playwright | Component correctness, and (via Playwright) the React↔Wasm↔Canvas integration seam that nothing else covers |
| Cross-cutting | Two headless server+client instances synchronized via Event Sourcing (CLAUDE.md Phase 3 milestone) | The actual determinism guarantee — server and client Engine producing bit-identical state from the same event stream |

---

## 12. Security Architecture (mapping RFC-001 NFRs to mechanisms)

| RFC-001 NFR | Concrete mechanism |
|---|---|
| Turn-gated action authorization (`Deploy`/`Attack`/`Fortify`/`AccelerateCompile`) | Server re-validates `actor_id ==` the `MatchInstance`'s current active player **and** that `current_phase` permits the action type, on every inbound message — independent of WS channel admission |
| Participant-gated action authorization (`Concede`, chat) | Validated against active-participant membership rather than turn order, since these are legitimate off-turn actions; still rejected for spectators, eliminated players, and non-participants |
| Timeout Determinism | Deadlines are tracked and fired server-side by the match-owning instance (Section 9); expiry emits a `TurnTimedOut` `SystemEvent` into the ledger. Client clocks drive countdown *display* only and are never consulted for enforcement |
| No password storage / no credential-breach exposure | Login is OTP (email) or OAuth only; there is no `password_hash` column in the schema and no password-based code path to secure in the first place |
| OTP guessing resistance | Redis-backed rate limiting on verification attempts per email/IP; codes are short-lived and single-use |
| Spectator Isolation | Spectator WS handler has no message-receive path wired at all — not a runtime check, a structural absence |
| PRNG Determinism | Seed generated once server-side per `MatchInstance`, embedded in the first `EventLog` entry, never re-rolled |
| Moderation Access Control | `report` table and moderator-only endpoints gated by a role claim in the JWT, checked at the handler layer before any query executes |
| Chat/Game-State Isolation | `ChatMessage` handling lives entirely in a Platform Server module that has no dependency on the `engine` crate — enforced at compile time by crate boundaries, not just convention |

---

## 13. Open Questions for Future Revision

* **Binary wire format migration point:** at what measured bandwidth/latency threshold does swapping JSON for `bincode`/`postcard` on the WS channel become worth the debuggability trade-off? Not answerable without production traffic data.
* **WebGL/PixiJS revisit:** two independent triggers, either sufficient on its own: (1) a future faction-visual-identity requirement (RFC-001 §2's open question on faction asymmetry) calling for shader-driven terraforming effects, or (2) hit-testing/redraw cost on very large maps exceeding what a spatial-index-assisted Canvas2D pass can hold at 60fps in profiling. If (2) is ever the trigger, the fix is specifically **color-picking**: render territories as flat, unique solid colors into an offscreen framebuffer and read back the pixel under the cursor — cheap, exact, GPU-side hit-testing that sidesteps CPU point-in-polygon entirely.
* **`event_log` partitioning strategy:** by `match_id` hash range, by time, or left unpartitioned until a specific query becomes slow — needs real volume data, not a guess made now.
* **Cold-storage retention window:** the exact duration a closed match's `event_log` stays "hot" in Postgres before archival to object storage (Section 7) is a product/support-process question (how long do disputes realistically take to surface?) as much as a technical one — pick a concrete number once Use Case 12's real-world report turnaround time is known, not before.
* **True leader election for match ownership:** deferred per Section 9; revisit if concurrent match count grows an order of magnitude beyond the current NFR.
* **Transactional email provider:** left unspecified deliberately (SMTP relay vs. an API-based service) — it's an interchangeable implementation detail behind a single "send OTP email" interface, not an architectural decision this RFC needs to pin down.
* **Cloud provider / managed-service choices** (managed Postgres vs. self-hosted, managed Kubernetes vs. self-hosted, CDN provider): explicitly out of this RFC's scope — it describes containerized, provider-agnostic artifacts so this choice can be made independently and later.
* **Bot compute isolation:** Use Case 5's server-side MCTS/minimax runs in the same process as request handling (via a bounded `rayon` pool) — revisit whether bot computation needs its own scaled-independently worker pool once bot difficulty/volume is known.
