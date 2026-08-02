# RFC-001: Terra-Forge Product Design Specification

* **Author:** Principal Architecture & Product Team
* **Status:** Draft / Proposed
* **Subject:** End-to-End Product Design, User Journeys, Features, and Entity Relationships for Terra-Forge

---

## 1. Executive Summary & Product Vision

**Terra-Forge** is a web-native, competitive strategy gaming platform. It blends the classic tactical tension of area-control board games (such as *War* or *Risk*) with a deep sci-fi narrative centered on autonomous AI faction colonization.

The product aims to deliver a frictionless, highly responsive browser-based experience supported by a rigorous, deterministic ruleset. Beyond just playing individual matches, Terra-Forge is designed as a complete competitive ecosystem featuring user profiles, matchmaking lobbies, live spectator streaming, dynamic leaderboards, and automated tournaments.

**Scope of this document:** This RFC captures Terra-Forge's complete, long-term product vision — every user journey the platform is ultimately meant to support, including social, moderation, and competitive-integrity flows that a first playable version would not need. It intentionally does not scope a minimum viable product; an MVP subset will be selected from this design in a separate planning pass, consistent with the phased build order in the idea doc's roadmap (headless engine first, metagame features last).

**The Technical Enabler:** These experience goals are made possible by an isomorphic architecture — the core game engine is written once, in a systems language, and compiled both to a native binary (for the authoritative server) and to WebAssembly (for the browser client). Running the identical logic on both ends is what allows local move validation, instant feedback, and deterministic state resolution without sacrificing the "zero-install, browser-only" promise.

---

## 2. Factions & Biomes

Every player embodies the AI core of a crashed Seeder ship, each executing a distinct terraforming protocol. Faction choice is captured at onboarding (Use Case 1) as `PlayerProfile.preferred_faction`, and it is the primary form of player expression on the board — a player's territories visually convert to their protocol's biome as they are conquered and recompiled (see the "Compiling" status in Use Case 3).

| Faction | Terraforming Protocol |
|---|---|
| **Silicon Swarm** | Converts the planetary crust into jagged, conductive metallic matrices |
| **Spore Colony** | Cultivates dense, hyper-toxic fungal forests that consume organic matter |
| **Cryo-Architects** | Plunges the atmosphere into perpetual winter, creating impenetrable glaciers |
| **Magma Forge** | Fractures tectonic plates to surface geothermal energy and lava |

**Open question for a future revision:** whether factions carry mechanical asymmetry (unique units or abilities) or remain a purely thematic/visual identity layered on a shared ruleset. This RFC assumes the latter — no asymmetric mechanics are specified anywhere in the current design corpus, and the domain model in Section 7 does not include per-faction stat overrides. Purely visual-identity factions risk making the game read as a reskinned *Risk* once the novelty of the Re-compile delay wears off, which is a real long-term depth concern — but resolving it (what asymmetry, how it's balanced) is game-design work this RFC isn't positioned to do today. To keep the door open without committing to specifics, the Core Engine's rule-evaluation functions should be designed from the start to accept a per-faction modifier lookup (defaulting to a uniform, no-op table) rather than hardcoding faction-agnostic math — see RFC-002 §5.1. That keeps "turn on asymmetry" a data change later, not an Engine rewrite, without speculatively designing the modifiers themselves now.

---

## 3. Product Characteristics & User Experience (UX) Goals

*   **Zero-Install Accessibility:** Running entirely via WebAssembly (Wasm) in modern web browsers, requiring no client downloads or heavy game-engine runtimes.
*   **Instant Feedback & Predictability:** Leveraging local Wasm execution for instant move validation and client-side prediction, ensuring the UI feels snappy even under network latency.
*   **Tactical Depth over RNG:** Eliminating dice-based randomness in favor of deterministic combat resolution and the strategic depth of the "Re-compile" delay mechanic.
*   **Social & Spectator First:** Built-in capabilities for non-playing users to stream matches in real-time, and for participants to communicate via configurable in-match chat, fostering community engagement and competitive streaming.
*   **Clean, Minimalist Aesthetics:** Styled around a futuristic, data-dense cyberpunk or industrial sci-fi interface, prioritizing tactical clarity over visual clutter.

---

## 4. Target User Personas

1.  **The Strategic Tactician (The Player):** Enjoys turn-based strategy, resource management, and outsmarting opponents through positioning rather than twitch reflexes. Wants fair, balanced matches and clear visibility of game state.
2.  **The Competitive Climber (The Ranked Enthusiast):** Driven by MMR progression, leaderboards, and seasonal tournaments. Seeks competitive integrity and verifiable match histories.
3.  **The Spectator / Community Viewer:** Enjoys watching high-level matches, studying strategies on the leaderboard, and participating in seasonal tournament brackets.
4.  **The Moderator (Platform Staff):** Not a player persona — a non-competing actor responsible for competitive integrity. Reviews conduct reports and disputed matches via replay, and issues rulings without ever mutating the immutable game record itself.

---

## 5. Core Use Cases & User Journeys

### Use Case 1: Account Setup & Profile Management
*   **Actor:** New or Returning User
*   **Journey:**
    1. User lands on the web platform and authenticates via a one-time passcode emailed to their address, or via OAuth — Terra-Forge never collects or stores a password.
    2. The system provisions a unique player profile containing stats (win rate, preferred faction, global MMR).
    3. User sets up a custom display name and avatar representing their AI Seeder core.

### Use Case 2: Matchmaking & Lobby Configuration
*   **Actor:** Host / Match Participants
*   **Journey:**
    1. A player clicks "Create Lobby," choosing map size, turn timers, and whether the game is Public or Private (invite-only).
    2. The host can add AI Bots to empty slots to fill a training match.
    3. Once all slots are filled or ready, the host initiates the match, transitioning all connected clients from the lobby view to the game board.

### Use Case 3: Executing a Turn Loop
*   **Actor:** Active Player
*   **Journey:**
    1. **Compile Phase:** Player reviews newly generated autonomous units and deploys them across territories they control. Generation is driven by the number of territories controlled plus a bonus for any fully-controlled `Continent` (see Section 7); territories still in the "Compiling" status do not contribute. This is also the only phase in which a player may spend units to accelerate a Compiling territory's conversion (step 4 below).
    2. **Execute Phase:** Player selects a source territory, targets an adjacent enemy territory, and initiates an attack. The local Wasm engine validates the move instantly (adjacency, unit count, turn phase) and renders the outcome optimistically; the "Attack Intent" is sent to the server as the authoritative action.
        *   **Combat resolution is fully deterministic** — no dice or random rolls. The server's engine computes the result from a fixed attrition matrix: attacking force size, defending force size, and the defending territory's inherent defense bonus combine to yield a single guaranteed outcome. Given the same inputs, the server and every connected client's local engine always compute the identical result.
    3. **Optimize Phase:** Player makes a single movement of units between connected, contiguous territories they control to reinforce chokepoints before ending their turn.
    4. **The Terra-Forge Twist:** A territory captured during the Execute phase does not immediately change hands in full — it enters a **"Compiling"** status. While Compiling:
        *   It generates no units during its owner's Compile phase, and does not count toward its `Continent`'s control bonus.
        *   It cannot be used as a source for an attack during Execute.
        *   It confers no defense bonus and can be attacked as though undefended.

        Compiling resolves one of two ways: (a) the owner simply waits — it completes automatically at the start of their next Compile phase, at no cost; or (b) during a subsequent Compile phase, the owner commits a fixed number of that turn's freshly-generated units directly to the Compiling territory instead of deploying them elsewhere, completing the conversion immediately. **Units are the only currency in this design — there is no separate abstract "resource."** Committed units are consumed by the conversion (they do not join the territory's `unit_count`), so acceleration is a genuine trade-off against board presence, not a free action. This deliberately avoids introducing a second economy layer alongside unit count; revisit only if a distinct resource type earns its place through actual gameplay needs.

        Once conversion completes, the territory's `faction` updates to the owner's and it participates normally in all three phases.

### Use Case 4: Spectating a Live Match
*   **Actor:** Spectator
*   **Journey:**
    1. User browses the "Live Matches" directory from the main dashboard.
    2. Selects an ongoing high-tier match and connects via a low-bandwidth spectator socket stream.
    3. Receives the initial game state snapshot and streams live event updates, watching the board update in real-time without interfering with player turns.

### Use Case 5: Practicing Against AI Bots
*   **Actor:** Solo Player / Lobby Host
*   **Journey:**
    1. While configuring a lobby (Use Case 2), the host fills any unclaimed seats with AI Bots instead of waiting for human players.
    2. On a bot's turn, the server clones the current authoritative game state and runs a headless simulation — evaluating candidate moves via search strategies such as Monte Carlo Tree Search or Minimax with alpha-beta pruning — entirely within the same native Engine used for human move validation, so bot actions are subject to identical rules and produce identical `EventLog` entries.
    3. The chosen bot action is emitted as a normal state-mutation event, indistinguishable on the wire from a human player's action, and all clients (including spectators) update identically.
    4. Because bot decision-making runs server-side against the authoritative Engine, bot difficulty can scale by adjusting simulation depth/time budget without any client-side changes.

### Use Case 6: Tournament Play & Leaderboards
*   **Actor:** Competitive Participant
*   **Journey:**
    1. User registers for an upcoming weekend tournament bracket.
    2. The system automatically seeds the bracket based on current MMR ratings.
    3. As rounds progress, winners advance automatically, and match results are permanently recorded in the immutable event ledger for dispute resolution and leaderboard updates.

### Use Case 7: Reconnecting to an In-Progress Match
*   **Actor:** Active Player (previously disconnected)
*   **Journey:**
    1. The player's connection drops mid-match (network loss, closed tab, browser crash). The server marks their session `Disconnected` but the match continues — their territories remain intact, and any running turn timer keeps advancing.
    2. The player returns, re-authenticates, and the client detects an in-progress `MatchInstance` tied to their `User`, offering to rejoin.
    3. The server streams the current state snapshot plus any `EventLog` entries since the client's last known sequence number; the local Wasm engine replays them to deterministically rebuild an identical board state.
    4. The player resumes control from wherever the match currently stands — mid-Compile, mid-Execute, or mid-Optimize — with no special "catch-up" UI beyond the normal turn indicator.

### Use Case 8: Turn Timer Expiration & AFK Handling
*   **Actor:** Active Player (unresponsive), other Match Participants
*   **Journey:**
    1. `Lobby.map_config` sets a per-turn time limit; the server tracks a deadline against `MatchInstance.current_turn`/`current_phase`.
    2. If the active player takes no action before the deadline, the server auto-resolves the current phase on their behalf (e.g., skip Compile deployment, skip Execute with no attacks, skip Optimize) and emits a `TurnTimedOut` ledger entry so all clients — including spectators — see an explicit, auditable reason for the skip rather than silence.
    3. Repeated timeouts (a threshold configured at lobby creation) escalate to an automatic **forfeit**, so a permanently disconnected player doesn't stall the match indefinitely for everyone else. A forfeit resolves the board identically to a voluntary concession (Use Case 9 — territories go neutral, player becomes a read-only spectator), but is recorded distinctly: it is emitted as a `SystemEvent` rather than a player-submitted `GameAction`, and `MatchHistory.end_reason` records `TimedOut` rather than `Conceded`. The distinction matters because abandoning a match and choosing to surrender warrant different competitive-integrity treatment.

### Use Case 9: Conceding a Match
*   **Actor:** Active Player
*   **Journey:**
    1. A player who still controls territory but judges the match unwinnable opens the in-game menu and confirms "Concede."
    2. The client sends a `Concede` action; the server validates it originates from an active participant and emits it as a normal `GameAction`/`EventLog` entry — no special-casing outside the Event Sourcing model.
    3. The conceding player's territories become **neutral**, retaining their existing unit garrisons as an unowned defending force — they are *not* redistributed to remaining players. Rationale: handing a conceder's board position to a rival would let concession be weaponized to swing a match (and, in a tournament context, to collude), whereas leaving neutral garrisons means the surrendered ground still has to be taken by force. The conceding player's own status transitions the same way as Elimination (Section 6): read-only spectator for the remainder of the match.
    4. `PlayerProfile` stats and `MatchHistory` record the concession distinctly from a combat elimination (see `MatchHistory.end_reason` in Section 7), since competitive-integrity tooling (Use Case 11) may treat a pattern of early concessions as a signal.

### Use Case 10: Post-Match Summary & Rating Update
*   **Actor:** Any Match Participant
*   **Journey:**
    1. On match closure (Section 6), the server computes MMR deltas for every participant from the final standings and writes one `MatchHistory` record per player (result, end reason, faction played, duration, MMR before/after).
    2. Each client receives a summary screen: final territory count, key moments (optionally extracted from the `EventLog`), and the MMR change.
    3. The player can navigate directly from the summary into Use Case 11 (full replay) or back to matchmaking (Use Case 2) for a rematch or a new lobby.

### Use Case 11: Match Replay & Dispute Review
*   **Actor:** Any User (reviewing their own past match) or a Moderator (reviewing any match, typically prompted by a Use Case 12 report)
*   **Journey:**
    1. A user selects a completed match from their `MatchHistory`, or a moderator opens a match referenced by a report.
    2. The server serves the match's full `EventLog` sequence; the client (or a moderation tool built on the same Wasm Engine) replays it deterministically from turn one, reproducing the exact board state at any point in the match — possible precisely because of the determinism guarantee in Section 8.
    3. The viewer can scrub turn-by-turn. A moderator specifically can flag a sequence range and attach a ruling (e.g., confirm fair play, annul the result) that updates the match's record without ever mutating the immutable `EventLog` itself.

### Use Case 12: Reporting Player Conduct
*   **Actor:** Any Match Participant or Spectator
*   **Journey:**
    1. During or after a match, a user flags another participant (abusive display name, suspected scripting/automation abuse, griefing) and submits a reason tied to the `MatchInstance`.
    2. The system creates a `Report` referencing the reporter, the reported `User`, and the match, and queues it for moderator review.
    3. A moderator investigates via Use Case 11's replay tooling and resolves the report (dismissed, warning issued, `PlayerProfile` sanctioned). The resolution itself is logged for accountability, independent of the game's own `EventLog`.

### Use Case 13: Inviting Friends to a Lobby
*   **Actor:** Lobby Host
*   **Journey:**
    1. While configuring a private lobby (Use Case 2), the host generates a shareable invite link or searches for a specific username to invite directly.
    2. Invited users receive a notification (in-app and/or via the link) and can join the lobby's waiting room directly, bypassing public matchmaking discovery.
    3. The host can revoke a pending invite, or the link itself, any time before the match starts.

### Use Case 14: Lobby Host Migration & Participant Removal
*   **Actor:** Lobby Host, Lobby Participants
*   **Journey:**
    1. Before a match begins, the host can remove a participant from the lobby, freeing their slot for another player or an AI Bot (Use Case 5).
    2. If the host disconnects while the lobby is still `Waiting`, host privileges transfer automatically to the next-longest-connected participant, so the lobby doesn't become unstartable.
    3. If the host disconnects after the match is `In-Progress`, no migration is needed — the host holds no special in-match authority beyond any other player, and the match continues under Use Case 7's reconnection handling.

### Use Case 15: Browsing Player Profiles & Match History
*   **Actor:** Any User
*   **Journey:**
    1. From a leaderboard entry, a lobby roster, or a spectated match, a user opens another player's public profile.
    2. The profile displays win/loss record, MMR, preferred faction, and a paginated list of recent `MatchHistory` entries, each linking into Use Case 11's replay.
    3. **Privacy note:** only aggregate stats and completed-match history are public; in-progress match state is never exposed through this journey — spectating a live match remains Use Case 4's dedicated flow.

### Use Case 16: Tournament Check-In & No-Show Handling
*   **Actor:** Tournament Registrant, Tournament System
*   **Journey:**
    1. Ahead of each round's start time, registered players receive a check-in window (e.g., 15 minutes) and must confirm presence.
    2. A player who fails to check in before the deadline is marked `no-show`; their bracket opponent receives an automatic walkover win without a `MatchInstance` ever being created.
    3. The advancing player's `MatchHistory` record carries `end_reason: Walkover` with a null `match_id` (no match was ever played). Rating calculations must treat `Walkover` differently from a competitive win — a bracket advancement earned by an opponent's absence is not evidence of skill, and inflating MMR from it would corrupt the seeding the bracket itself depends on.

### Use Case 17: In-Match Chat
*   **Actor:** Match Participants; Spectators (only if the host allows it)
*   **Journey:**
    1. At lobby creation (Use Case 2), the host configures a chat mode — `Disabled`, `Quick Pings` (a fixed set of preset callouts, e.g. "Attack incoming," "Need reinforcements" — no free text), or `Free Text` — and, independently, whether spectators can view the participant channel (`Hidden` or `Read-Only`).
    2. **All in-match chat is authored by participants only.** Spectators are, at most, a read-only audience for it; they are never granted a write path, which keeps the Spectator Isolation guarantee (Section 8) structural rather than a rule the server has to remember to enforce. During the match, participants send messages through a chat panel and the server broadcasts each to the configured audience over a channel kept entirely separate from the game-state event stream.
    3. **Architectural boundary:** chat messages are never fed into the Core Engine and never appear in the `EventLog` — they carry no game-state meaning, so admitting them into the deterministic ledger would violate the Engine's zero-I/O constraint for no gameplay benefit. They persist instead as `ChatMessage` records, timestamped against the match's `sequence_number` at time of send so a moderator reviewing a report (Use Case 12) can line up chat and game replay side by side.
    4. Any participant can locally mute another participant's messages at any time (client-side, no server round-trip needed), and can flag a specific message when filing a Use Case 12 report.

**Open questions for a future revision:**
*   **Spectator-to-spectator chat** (a "crowd" channel alongside the participant one) is deliberately excluded above. It is genuinely desirable for the "Social & Spectator First" pillar, but it would require giving the spectator socket an inbound message path — directly weakening the Spectator Isolation NFR's current strength, which is that no such path exists at all. Adding it means designing that channel so it cannot become a vector for `GameAction` injection, plus its own moderation surface (a match with thousands of viewers is a very different moderation problem than one with six players). That is its own design pass, not a footnote to this one.
*   **Voice chat** is likewise out of scope — it implies real-time media infrastructure (SFU/relay, codecs) orthogonal to this platform's WebSocket/event-sourcing model.

---

## 6. Match Resolution & Win/Loss Conditions

*   **Victory:** A player achieves total planetary dominance by controlling 100% of the map's territories (fully compiled, not merely captured). Note that neutral territories left behind by a concession or forfeit still have to be conquered — surrendering does not hand anyone a shortcut to this condition.
*   **Player exit:** A player leaves an in-progress match one of three ways — **Elimination** (their last active territory is overtaken and their core AI destroyed), **Concession** (voluntary surrender, Use Case 9), or **Forfeit** (repeated turn timeouts, Use Case 8). All three release the player's territories to neutral and leave them connected in a read-only spectator capacity for the remainder of the match; they differ only in what `MatchHistory.end_reason` records.
*   **Match closure:** Upon victory, or once only one active player remains by any combination of the exits above, `MatchInstance.status` transitions to `Closed`, the final `EventLog` sequence is sealed (immutable) with a `MatchClosed` entry, and `MatchHistory`, `PlayerProfile` stats (wins/losses, MMR) and `Lobby.status` update accordingly.

---

## 7. Domain Model & Entity Relationships

The platform's data architecture relies on cleanly decoupled entities managing identity, competitive state, and game progression. Platform entities (identity, matchmaking, competitive record) are distinct from game-state entities (map, territories, factions) — the latter are owned and mutated exclusively by the authoritative Engine inside a `MatchInstance`, never written to directly by client or API code outside the turn-resolution path (see the Security NFR in Section 8).

### Platform Entities

*   **User**
    *   *Attributes:* `id`, `username`, `email`, `created_at`, `status`
    *   *Relationships:* 1-to-1 with **PlayerProfile**, 1-to-many with **MatchHistory**, 1-to-many with **Report** (as reporter or reported party).

*   **PlayerProfile**
    *   *Attributes:* `user_id`, `mmr`, `games_played`, `wins`, `losses`, `preferred_faction`
    *   *Relationships:* Belongs to **User**. `preferred_faction` references **Faction** (Section 2).

*   **MatchHistory**
    *   *Attributes:* `id`, `user_id`, `match_id` (nullable — a tournament walkover per Use Case 16 produces a record with no match behind it), `result` (`Win` | `Loss`), `end_reason` (`Dominance` | `LastStanding` | `Eliminated` | `Conceded` | `TimedOut` | `Walkover`), `faction_played` (nullable, for the same walkover case), `mmr_before`, `mmr_after`, `duration` (nullable), `ended_at`
    *   *Relationships:* Belongs to **User**, optionally references **MatchInstance**. Outcome and cause are deliberately two fields rather than one conflated enum: *did this player win* and *how did it end* are independent questions, and every consumer needs a different pair of them — leaderboards read `result` alone, while competitive-integrity tooling (Use Cases 11–12) cares specifically about `end_reason` patterns like repeated `Conceded` or `TimedOut`. Written once by the authoritative server at match closure (Section 6, Use Case 10) and never mutated afterward — it is a summary snapshot, not a source of truth; the `EventLog` remains authoritative for replay (Use Case 11).

*   **Report**
    *   *Attributes:* `id`, `reporter_id`, `reported_user_id`, `match_id`, `chat_message_id` (nullable, set when the report targets a specific chat message rather than general match conduct), `reason`, `status` (`Pending` | `Reviewed` | `Actioned` | `Dismissed`), `moderator_notes`, `created_at`
    *   *Relationships:* References the reporting **User**, the reported **User**, the **MatchInstance** under dispute, and optionally a **ChatMessage** (Use Case 12). A moderation record, independent of `EventLog` — never part of the game-state audit trail, and visible only to platform staff (see the Moderation Access Control NFR in Section 8).

*   **Lobby**
    *   *Attributes:* `id`, `host_id`, `is_private`, `invite_code` (nullable, set when the host generates a shareable invite), `map_config`, `chat_mode` (`Disabled` | `QuickPings` | `FreeText`), `spectator_chat_visibility` (`Hidden` | `ReadOnly`), `status` (Waiting, In-Progress, Closed)
    *   *Relationships:* 1-to-many with **User** (Participants), 1-to-1 with **MatchInstance**. `chat_mode` and `spectator_chat_visibility` are set at lobby creation (Use Case 2) and carried onto the resulting `MatchInstance`'s chat channel (Use Case 17).

*   **Tournament**
    *   *Attributes:* `id`, `name`, `status`, `start_time`, `bracket_structure`
    *   *Relationships:* 1-to-many with **User** (Registrants), 1-to-many with **MatchInstance**.

### Game-State Entities

*   **MatchInstance**
    *   *Attributes:* `id`, `lobby_id`, `seed` (for PRNG), `current_turn`, `current_phase` (Compile / Execute / Optimize), `status` (In-Progress, Closed), `turn_deadline` (nullable, derived from `Lobby.map_config`'s timer setting), `state_blob`
    *   *Relationships:* Belongs to **Lobby**, 1-to-many with **EventLog**, 1-to-1 with **Map**. `state_blob` is a periodically-written snapshot of Engine state kept purely to speed up reconnection and crash recovery — it is a cache, never the source of truth, and can always be discarded and regenerated by replaying the `EventLog` from sequence 0.

*   **EventLog**
    *   *Attributes:* `id`, `match_id`, `sequence_number`, `event_type` (`PlayerAction` | `SystemEvent`), `action_payload`, `timestamp`
    *   *Relationships:* Belongs to **MatchInstance**. For `PlayerAction` entries, `action_payload` is a serialized **GameAction**; for `SystemEvent` entries it is a server-generated payload requiring no player input — `MatchStarted` (carrying the PRNG `seed`, always sequence number 0), `TurnTimedOut` (Use Case 8), `PlayerForfeited` (Use Case 8's escalation), `CompileCompleted`, and `MatchClosed`. Both kinds are equally part of the immutable, replayable Event Sourcing ledger (implements the audit trail); a replay consumes them identically.

*   **Map**
    *   *Attributes:* `id`, `match_id`, `size_config`
    *   *Relationships:* 1-to-many with **Territory**, 1-to-many with **Continent**. Generated once at match start, deterministically, from `MatchInstance.seed`.

*   **Continent**
    *   *Attributes:* `id`, `map_id`, `name`, `control_bonus`
    *   *Relationships:* 1-to-many with **Territory**. `control_bonus` applies to a player's Compile phase only when every child `Territory` shares that player's `owner_id` and none are `Compiling`.

*   **Territory**
    *   *Attributes:* `id`, `continent_id`, `owner_id` (nullable), `faction` (nullable until first capture), `unit_count`, `status` (`Active` | `Compiling`), `adjacent_territory_ids`
    *   *Relationships:* Belongs to **Continent**, belongs to **Map**. `faction` references **Faction** (Section 2); `owner_id` references **User**.

*   **GameAction**
    *   *Attributes:* `type` (`Deploy` | `Attack` | `Fortify` | `Concede` | `AccelerateCompile`), `actor_id`, `source_territory_id`, `target_territory_id` (nullable), `unit_count`
    *   *Relationships:* Serialized as `EventLog.action_payload` for `PlayerAction`-typed entries. Validated by the Engine against `MatchInstance.current_phase` and territory adjacency before being accepted; `Concede` (Use Case 9) is valid in any phase, from any active participant, exactly once per match. `AccelerateCompile` (Use Case 3) is valid only during its actor's Compile phase, targets a `Compiling` territory they own, and consumes `unit_count` units from that turn's newly-generated pool rather than depositing them.

*   **Faction** *(enum, not a persisted entity)*
    *   *Values:* `SiliconSwarm`, `SporeColony`, `CryoArchitects`, `MagmaForge` — see Section 2 for protocol descriptions.

*   **ChatMessage**
    *   *Attributes:* `id`, `match_id`, `sender_id`, `body`, `sequence_number_at_send` (the `EventLog.sequence_number` current at send time, for review-tool correlation only), `created_at`
    *   *Relationships:* Belongs to **MatchInstance**, references the sending **User** — always a match participant, never a spectator (Use Case 17). There is deliberately no per-message audience field: visibility is a property of the match, set once by `Lobby.spectator_chat_visibility`, so a single message can never be mis-scoped at write time. Deliberately **not** part of `EventLog` and never touches the Core Engine — a platform-layer record living entirely in Module B (Platform Server), never seen by Module A (Core Engine). See the Chat/Game-State Isolation NFR in Section 8.

---

## 8. Non-Functional Requirements (NFRs)

*   **Determinism:** Identical sequences of action payloads must yield bit-wise identical game states across the server and all client Wasm runtimes.
*   **Client-Side Prediction & Rollback:** The client's local Wasm Engine must validate and optimistically render a player's own action immediately on input, without waiting for a server round-trip. If the server subsequently rejects the action (e.g., stale state, invalid target), the client must roll back to the last server-authoritative state without requiring a manual refresh.
*   **PRNG Determinism:** Any non-deterministic-seeming behavior (map generation, spawn point assignment) must be driven by a single PRNG seed set once by the server per `MatchInstance` and distributed to all clients, so that "random" sequences are bit-identical across server and clients.
*   **Scalability:** The authoritative server backend must support hundreds of concurrent active matches and thousands of concurrent spectator connections with minimal latency.
*   **Resilience & Recovery:** Match states must be reconstructable at any moment using the immutable `EventLog` ledger in case of unexpected network drops or server restarts — this is also the mechanism backing player reconnection (Use Case 7), not just server-side recovery.
*   **Security (action authorization):** Every inbound action must be authorized server-side per action, not merely at connection time. Two distinct tiers exist and must not be collapsed into one check:
    *   **Turn-gated actions** (`Deploy`, `Attack`, `Fortify`, `AccelerateCompile`): accepted only from the player whose turn it currently is, and only in the phase that permits them.
    *   **Participant-gated actions** (`Concede`, chat messages): accepted from any active participant regardless of whose turn it is — a player must be able to surrender or speak while waiting — but never from a spectator, an eliminated player, or a non-participant.

    Both tiers are re-validated against authoritative server state on every message; channel admission alone never authorizes an action.
*   **Spectator Isolation:** Spectator connections are read-only by construction — the spectator channel must not accept `GameAction` submissions, and the server must reject any mutation attempt originating from a non-participant connection regardless of client-side enforcement. Chat visibility (Use Case 17) is a separate, independently configurable channel from game-event visibility — a spectator granted `ReadOnly` chat access still has zero write access to either channel.
*   **Chat/Game-State Isolation:** `ChatMessage` traffic must never be routed through the Core Engine or written to `EventLog`. It is handled entirely by the Platform Server (Module B) as ordinary application data — this preserves the Engine's zero-I/O, zero-UI/network constraint and keeps chat volume from affecting the size or replay cost of the deterministic ledger.
*   **Timeout Determinism:** Turn-timer expirations (Use Case 8) must be resolved authoritatively by the server and emitted as ordinary `SystemEvent` entries in the `EventLog`; client-side clocks are advisory display only and are never a source of truth for whether a deadline has passed.
*   **Moderation Access Control:** `Report` records and moderator rulings (Use Cases 11–12) must only be readable by platform-staff-authorized roles — never exposed through the public profile (Use Case 15) or match-history APIs, regardless of the requester's relationship to the match.
