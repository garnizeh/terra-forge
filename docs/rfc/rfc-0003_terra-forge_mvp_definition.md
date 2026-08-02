# RFC-003: Terra-Forge MVP Definition

* **Author:** Principal Architecture & Product Team
* **Status:** Draft / Proposed
* **Subject:** Scoping the Minimum Viable Product — which subset of RFC-001's product surface and RFC-002's architecture ships first, and why
* **Companion documents:**
  * [RFC-001](rfc-0001_terra-forge_product_design_specification.md) defines the complete long-term product (17 use cases, full domain model, full NFR set) and explicitly defers MVP scoping to "a separate planning pass" (§1). This RFC is that pass.
  * [RFC-002](rfc-0002_terra-forge_high_level_architecture.md) defines the complete long-term architecture and likewise defers MVP scoping (§1). This RFC selects the subset of its Technology Decision Log that MVP actually needs standing up.
  * [CLAUDE.md](../../CLAUDE.md) — the six-phase roadmap this RFC is grounded in.

---

## 1. Purpose & Scope

RFC-001 and RFC-002 describe *everything Terra-Forge is meant to become*. Neither commits to what ships first. This RFC does that: it draws one line through both documents, marking every use case, entity, NFR, and technology choice as either **MVP** or **deferred**, with a stated reason for each deferral.

This is a **scoping** document, not a new design. It introduces no product ideas or technical choices that aren't already present in RFC-001/RFC-002 — it only selects and, where a full RFC-001 use case is too large to ship whole, trims it down to a smaller version that preserves the use case's core intent. Any trim is called out explicitly against the section it trims.

**What "MVP" means here:** the smallest slice of the product that (a) lets two people play a complete match of Terra-Forge against each other, start to finish, in a browser, over the network, and (b) proves the architecture's central bet — that a Rust Engine compiled to both native and Wasm keeps server and client bit-identical — under a real (if small) multiplayer session. Anything not load-bearing for that is deferred.

---

## 2. Guiding Principle: Roadmap Phases 1–4, Not Phase 5

CLAUDE.md's roadmap already draws a natural seam:

> Phase 4: Full frontend rendering (Wasm in-browser, visual map/animations)
> Phase 5: Platform expansion (accounts, DB persistence, matchmaking, spectator mode)

Phase 4 is the first point at which the roadmap describes something a user could open in a browser and play. Phase 5 is explicitly named "expansion" — it's additive to an already-playable game, not a precondition for one. That ordering is the load-bearing argument for this RFC's scope line: **MVP = Phases 1–4, plus only the thinnest possible identity/lobby substrate needed to let a match exist at all** (a match needs *some* notion of "which player is which," even without accounts).

This has one significant, deliberate consequence worth stating up front: **MVP has no persistent accounts.** RFC-001 Use Case 1 (email OTP / OAuth login, `PlayerProfile`, MMR) is a Phase 5 feature by the roadmap's own ordering, and this RFC keeps it there. MVP identity is a throwaway per-session display name — see §6.

A second consequence: because MVP explicitly excludes matchmaking (Phase 5) and never runs more than one match concurrently in its target deployment, most of RFC-002 §8–9 (Redis pub/sub fan-out, multi-instance match-ownership registry, Kubernetes) has no MVP audience yet. See §7.

---

## 3. Success Criteria

MVP is done when all of the following hold simultaneously:

1. Two players, each in their own browser, on two different machines, can create a private lobby, join it, and play a full match to a win condition (dominance, elimination, or concession) with no developer intervention.
2. Every combat resolution and phase transition the two clients render is bit-identical to what the server computed — verifiable by comparing client and server Engine state at match end.
3. A player can refresh their browser mid-match and rejoin without the match stalling or desyncing.
4. The same `engine` crate, unmodified, is the thing running natively on the server and compiled to Wasm in the browser (RFC-002 §2 principle #1 and #4 — no parallel TypeScript reimplementation of any rule).
5. The Re-compile Delay, the deterministic attrition matrix, and all three turn phases (Compile/Execute/Optimize) are playable exactly as specified in RFC-001 Use Case 3 — this is the mechanic the entire product concept rests on, so it ships whole, not trimmed.

---

## 4. In-Scope Use Cases (trimmed from RFC-001 §5)

| RFC-001 UC | MVP treatment |
|---|---|
| UC2 — Matchmaking & Lobby Configuration | **Trimmed.** Host creates a private lobby (map preset, player count, turn timer on/off) and shares an invite link (folds in UC13's link-sharing, minus username search invites). No `Public` lobby, no matchmaking discovery/queue — those are Phase 5. |
| UC3 — Executing a Turn Loop | **In full.** Compile / Execute / Optimize, the deterministic attrition matrix, and the Re-compile Delay (including the accelerate-with-units path) ship exactly as specified — see §3 point 5. |
| UC7 — Reconnecting to an In-Progress Match | **In full.** Cheap given Event Sourcing is already required for UC3's client/server sync, and a genuinely playable multiplayer game must survive a tab refresh. |
| UC8 — Turn Timer Expiration & AFK Handling | **Trimmed.** Auto-skip on timeout ships (needed so one AFK player can't freeze the match indefinitely); the *optional* per-lobby timer is part of UC2's config. Repeated-timeout-escalates-to-forfeit ships too, since it reuses the same Engine transition as Concession (UC9) — no separate mechanism to build. |
| UC9 — Conceding a Match | **In full.** Cheap (one `GameAction` variant) and necessary — without it, a losing player's only way out of a match is to force an opponent to grind out full elimination. |
| UC14 — Lobby Host Migration & Participant Removal | **Trimmed.** Host can remove a not-yet-started participant. Host-migration-on-disconnect (keeping a pre-start lobby alive if the host drops) is deferred — for MVP's private, host-shares-a-link flow, the fallback is simply that the lobby dies and the host re-creates it. |

**Section 6's win/loss conditions (Victory, Elimination, Concession, Forfeit, Match Closure) ship in full** — they're small, and every exit path above depends on them.

---

## 5. Deferred to Post-MVP

| RFC-001 UC / RFC-002 section | Deferred to | Why it can wait |
|---|---|---|
| UC1 — Account Setup (email OTP/OAuth, `PlayerProfile`, MMR) | Phase 5 | Roadmap explicitly places accounts in Phase 5, after full frontend rendering (§2). Ephemeral per-session identity (§6) satisfies everything MVP actually needs from "who is this player." |
| UC4 — Spectating a Live Match | Phase 5 | Roadmap names spectator mode as a Phase 5 item explicitly. No spectator socket, no read-only channel, no crowd of viewers to serve. |
| UC5 — Practicing Against AI Bots | Phase 6 | CLAUDE.md dedicates a whole phase to bots specifically because they need a stable, tested headless Engine first — building bot search against a still-changing rule set would mean re-tuning it repeatedly for no benefit. |
| UC6 — Tournament Play & Leaderboards | Phase 5 | Depends on `PlayerProfile`/MMR (UC1), which is itself deferred. |
| UC10 — Post-Match Summary & Rating Update | Trimmed, rest deferred | MVP shows final territory count and the win/loss/concession reason (data the match already has); the MMR delta half of this use case is inapplicable with no accounts. |
| UC11 — Match Replay & Dispute Review | Phase 5 | The `EventLog` that makes this possible is already an MVP artifact (needed for UC7 reconnection), so nothing is lost by deferring — a replay *viewer* is pure additive UI on data MVP is already producing. |
| UC12 — Reporting Player Conduct | Phase 5 | No moderation team, no `Report` table, no accounts to sanction yet. |
| UC13 — Inviting Friends to a Lobby | Folded into UC2 (link only) | Username search requires accounts (UC1). |
| UC15 — Browsing Player Profiles & Match History | Phase 5 | Depends entirely on `PlayerProfile`/`MatchHistory` (UC1). |
| UC16 — Tournament Check-In & No-Show Handling | Phase 5 | Depends on Tournament (UC6). |
| UC17 — In-Match Chat | Phase 5 | Genuinely useful, but additive to a working match loop, not required for one — and RFC-001 §5 itself flags the harder spectator-chat variant as its own design pass; even participant-only chat can wait. |
| Faction mechanical asymmetry (RFC-001 §2 open question) | Unresolved, no phase yet | Already explicitly unresolved in RFC-001; MVP factions stay purely visual, per that RFC's own default. |
| RFC-002 §8 — Redis (pub/sub fan-out, session/matchmaking state) | Phase 5 | A direct consequence of §7 below: MVP runs one server instance, so there is no cross-instance fan-out problem yet to solve, and no matchmaking queue to store. |
| RFC-002 §9 — Horizontal scaling / match-ownership registry | Phase 5+ | Same reason — meaningless with exactly one server instance. |
| RFC-002 §10 — Kubernetes | Phase 5+ | RFC-002 itself already scopes K8s as "target, not required early." Docker Compose covers MVP. |
| RFC-002 §7 — Cold storage archival to S3/R2/MinIO | Phase 5+ | Archival exists to keep Postgres sized for match volume that doesn't exist at MVP scale. `event_log` just stays in Postgres. |
| RFC-001 §8 — Moderation Access Control NFR | Phase 5 | No `Report` table, nothing to gate. |

---

## 6. MVP Identity: Ephemeral Session, Not Accounts

Because UC1 is deferred (§5) but a match still needs to know who's who, MVP substitutes a minimal stand-in:

* On creating or joining a lobby, a player supplies a **display name only** — no email, no password, no OAuth, no OTP.
* The server issues a session token scoped to a single `match_id`, with a **4-hour sliding TTL** — refreshed on every accepted action (including reconnection heartbeats), not fixed from issuance. This covers untimed matches (turn timer is optional per §9) running long without the token expiring mid-game, while still expiring a genuinely abandoned session. Starting a new match always issues a new token — a token never spans two matches, so there's no reuse path to reason about. It is not persisted beyond the match's lifetime and grants no identity outside it — closing the tab and rejoining with the same display name is a *new* identity as far as the system is concerned (this is acceptable for MVP; it becomes a real account exactly when UC1 ships).
* This is enough to populate every place RFC-001's domain model needs an "actor": `Territory.owner_id`, `GameAction.actor_id`, lobby participant lists. It is deliberately **not** RFC-001's `User` entity — no `PlayerProfile`, no cross-match history, no row that outlives the match.
* Consequence for RFC-002's auth stack (§3, §6): MVP needs no JWT access/refresh pair, no `jsonwebtoken`, no OTP email flow, no `otp_code` table, no transactional email provider. The lobby/match session token is a much smaller mechanism — an opaque, server-generated, single-purpose credential valid only for that match's WS connection.

---

## 7. MVP Technical Scope (subset of RFC-002 §3's Technology Decision Log)

| Component | MVP scope | Deferred |
|---|---|---|
| Core Engine | Full RFC-002 §5.1 as written — native + Wasm dual target, `cargo test` + `proptest` + `wasm-pack test --headless` | Faction-modifier table stays the no-op default (§5's asymmetry deferral) |
| Backend | Rust + Axum + Tokio, **single instance** — no match-ownership registry, no cross-instance relay | Multi-instance topology (RFC-002 §9), reverse-proxy match-aware routing beyond a plain LB |
| Database | PostgreSQL via `sqlx`, holding only `lobby` (trimmed), `match_instance`, `event_log`; ULID PKs as specified | `user`, `player_profile`, `match_history`, `report`, `tournament`, `chat_message`, `otp_code` tables — none exist until the use cases that need them ship |
| Cache / fan-out | **None.** No Redis. | Redis (pub/sub, session store, rate limiting) — reintroduced with Phase 5 multi-instance + matchmaking |
| Frontend | Vite + React + TypeScript, Canvas2D board renderer, Zustand for UI-only state, `wasm-pack build --target web`, `WsClient` with reconnect/resume | — (frontend stack is already minimal in RFC-002; nothing to trim) |
| Type sharing | `ts-rs` for `GameAction` and event payloads | — |
| Wire format | JSON over WSS | `bincode`/`postcard` migration (already an open question in RFC-002 §13, unaffected by MVP) |
| Auth | Ephemeral match-session token (§6) | JWT, OTP email, OAuth (RFC-002 §6) |
| Containerization | Docker + Docker Compose (server, Postgres) | Kubernetes, managed cloud services |
| CI/CD | `engine-ci`, `server-ci`, `frontend-ci` as specified in RFC-002 §10 | Release workflow can stay minimal — no versioned public releases yet |
| Observability | `tracing` structured logs | Prometheus/Grafana dashboards — worth deferring until there's traffic to observe; add early if debugging needs it |
| Cold storage | None | S3-compatible archival (RFC-002 §7) |

---

## 8. MVP Domain Model (subset of RFC-001 §7)

**Game-State Entities — ship in full**, unchanged from RFC-001 §7: `MatchInstance`, `EventLog`, `Map`, `Continent`, `Territory`, `GameAction`, `Faction`. This is the Engine's own model; trimming it would mean trimming the game, which contradicts §3's success criteria.

**Platform Entities — heavily trimmed:**

* **Lobby** — kept, but narrowed: `id`, `host_session_id`, `map_config` (one of a small fixed set of presets — see §9), `turn_timer_seconds` (nullable), `status`. Drops `is_private`/`invite_code` distinction (MVP lobbies are always link-only), drops `chat_mode`/`spectator_chat_visibility` (no chat, no spectators yet).
* **No `User`, `PlayerProfile`, `MatchHistory`, `Report`, `Tournament`, `ChatMessage`.** These simply don't exist as tables until the use case that needs them ships (§5). A lobby's "participants" are the ephemeral session identities from §6, held in server memory and referenced by `session_id` inside `Lobby`/`MatchInstance` — not a persisted relation to any `User` row.

---

## 9. MVP Match Configuration

To keep the Engine's first playtest surface small without touching the rules themselves:

* **Player count:** 2–4, fixed at lobby creation. No AI bots to fill empty seats (UC5 deferred) — an MVP lobby simply can't start until every seat is filled by a human.
* **Map:** a small, fixed set of **hand-authored** presets (e.g. 2–3 layouts sized for 2–4 players) for the first ship, not the fully open `size_config` parameterization RFC-001 implies long-term. Hand-authored is chosen over generator-first purely for shipping speed — but the presets must be stored as ordinary serialized `Map`/`Territory`/`Continent` instances (the Engine's normal in-memory representation), so a future PRNG-seeded generator producing the same structure is a drop-in addition, not a rework of how the Engine consumes a map. Spawn-point assignment within a preset still goes through the seeded PRNG per RFC-001 §8's determinism NFR regardless of how the layout itself was produced.
* **Turn timer:** optional, per-lobby, matching UC8's trimmed scope (§4).

---

## 10. MVP Non-Functional Requirements (subset of RFC-001 §8)

| NFR | MVP status |
|---|---|
| Determinism | **In full** — this is §3's core success criterion, non-negotiable at any scope. |
| Client-Side Prediction & Rollback | **In full** — required for UC3 to feel correct; it's an Engine/frontend integration concern, not a Phase-5 feature. |
| PRNG Determinism | **In full.** |
| Resilience & Recovery / reconnection | **In full**, scoped to single-instance recovery (process restart replays from `EventLog`) rather than RFC-002 §9's instance-failover handoff, which requires the multi-instance registry this RFC defers. |
| Security (turn-gated + participant-gated action authorization) | **In full** — server-side re-validation on every action, unchanged from RFC-001 §8, just checked against the §6 session token instead of a JWT claim. |
| Timeout Determinism | **In full**, per §4's trimmed UC8. |
| Scalability ("hundreds of concurrent matches") | **Deferred.** MVP's single-instance deployment is explicitly not sized for this — see §7. Revisit when Phase 5 multi-instance work begins. |
| Spectator Isolation | **N/A at MVP** — no spectator channel exists yet to isolate (UC4 deferred). Re-enters scope the moment UC4 ships. |
| Chat/Game-State Isolation | **N/A at MVP** — no chat exists yet (UC17 deferred). |
| Moderation Access Control | **N/A at MVP** — no `Report` table (UC12 deferred). |

---

## 11. Definition of Done

MVP ships when every item below is checked, mirroring §3's success criteria in testable form:

- [ ] `engine` crate: combat matrix, phase transitions, Re-compile Delay, map generation — full `cargo test` + `proptest` coverage, compiles to both a native `rlib` and `wasm32-unknown-unknown`.
- [ ] `wasm-pack test --headless` passes (no accidentally non-portable code).
- [ ] Backend: lobby create/join via invite link, WS participant channel, `GameAction` validation and authorization (turn-gated and participant-gated per §10), `EventLog` persisted to Postgres, reconnection via sequence-number replay.
- [ ] Frontend: lobby creation/join UI, Canvas2D board rendering reading live from the Wasm Engine instance, phase-appropriate action UI for Compile/Execute/Optimize, win/loss/concession end screen.
- [ ] End-to-end: two real browsers, two machines (or two isolated browser profiles), full match played to completion, including at least one deliberate mid-match reconnect.
- [ ] Determinism check: server and both clients' final Engine states compared byte-for-byte (or via a deterministic hash) at match end — this is the one test that validates the entire architectural bet, not just a feature.
- [ ] `docker-compose up` brings up server + Postgres locally with no manual steps beyond `.env` configuration.

---

## 12. Scoping Decisions Made During Review

These were raised as open questions in an earlier draft and have since been settled:

* **Map presets — hand-authored or generator-first?** **Decided: hand-authored first** (§9), 2–3 presets, but stored in the Engine's normal `Map`/`Territory`/`Continent` representation so a PRNG-seeded generator can be added later without changing how the rest of the system consumes a map.
* **Ephemeral session token lifetime?** **Decided: 4-hour sliding TTL**, refreshed on every accepted action, scoped to a single `match_id` (§6). A new match always issues a new token.
* **Is single-instance deployment sufficient for MVP's actual first users?** **Decided: yes.** MVP targets few concurrent matches (likely one at a time initially); this RFC's "hundreds of concurrent matches" NFR (RFC-002 §3) is explicitly Phase-5-scale, not Phase-4. Revisit §7's single-instance simplification only if real usage exceeds this — solving for scale that doesn't exist yet is exactly the premature work CLAUDE.md's phased ordering is meant to avoid.
