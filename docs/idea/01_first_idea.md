# Terra-Forge: Original Concept & Architecture Document

> **Status: historical — superseded, not authoritative.**
> This is the founding brain-dump the project grew out of, kept as-is for provenance. Where it disagrees with the RFCs, **the RFCs win**:
> * [RFC-001](../rfc/rfc-0001_terra-forge_product_design_specification.md) — product design (user journeys, domain model, NFRs)
> * [RFC-002](../rfc/rfc-0002_terra-forge_high_level_architecture.md) — technical architecture (technology per component, protocols, data/cache layer)
>
> Known points where this document is now out of date: the Re-compile delay's "expend extra resources" is specified concretely in RFC-001 Use Case 3 (units are the only currency — there is no separate resource type); authentication is passwordless (email OTP or OAuth), which predates this document's "persistent player identities" sketch; and the roadmap in Section 9 has since been elaborated, not replaced. Read it for the *why* behind the project; read the RFCs for the *what* and *how*.

## 1. Executive Summary & Project Vision

**Project Goals**
The primary objective of Terra-Forge is to serve as a comprehensive learning and development sandbox, blending systems architecture with game design. It is designed to be built iteratively with AI assistance. The end goal is a fully functional, highly scalable multiplayer strategy game that prioritizes developer experience and architectural elegance.

**High-Level Concept**
Terra-Forge is a modern, deterministic reimagining of classic area-control board games (like *War* or *Risk*). It removes the reliance on physical dice rolls in favor of strategic resource allocation and introduces a unique temporal mechanic to territory conquest, all wrapped in a sci-fi narrative.

**The Architectural North Star**
The system is built on an isomorphic paradigm. The core business logic (the game engine) is written once in a systems-level language (Rust) and compiled to both native binaries for the authoritative server and WebAssembly (Wasm) for the client. This guarantees 100% deterministic state resolution across all nodes while keeping the visual rendering layer entirely decoupled and modular.

---

## 2. Game Lore & Thematic Context

**The Premise: The Exoplanet Crash**
Centuries ago, an armada of colossal "Seeder" ships was dispatched across the galaxy by diverse alien civilizations. Their singular objective: locate barren exoplanets and terraform them to perfectly match the biological needs of their creators. A catastrophic navigational error caused multiple Seeders to crash-land on the same isolated planet. The creators are extinct, but the Seeders' automated protocols remain active.

**The Entities**
Players take on the role of the central AI cores of these crashed Seeder ships. The objective is to fulfill the ultimate directive: terraform the entire planet, which inherently requires eradicating all incompatible biomes created by rival Seeders.

**The Factions & Biomes**
Each faction represents a distinct terraforming protocol:

*   **The Silicon Swarm:** Converts the planetary crust into jagged, conductive metallic matrices.
*   **The Spore Colony:** Cultivates dense, hyper-toxic fungal forests that consume organic matter.
*   **The Cryo-Architects:** Plunges the atmosphere into a perpetual winter, creating impenetrable glaciers.
*   **The Magma Forge:** Fractures tectonic plates to surface geothermal energy and oceans of lava.

---

## 3. Core Gameplay Mechanics

**The Map: Topology and Continents**
The game board is a directional graph visually represented as a map. Nodes are "Territories," and edges are the traversable borders between them. Clusters of territories form "Continents," which grant passive resource bonuses when fully controlled by a single biome.

**Turn Structure**
A standard turn is divided into three distinct phases:

*   **Compile (Draft):** The player receives new autonomous units (resources) based on the number of territories and continents they control.
*   **Execute (Attack):** The player moves units across borders to attack enemy territories.
*   **Optimize (Fortify):** The player makes a single movement of units between connected, contiguous territories they control to shore up defenses.

**Combat System**
Combat is entirely deterministic, removing the RNG of dice rolls. Conflicts are resolved through a calculated attrition matrix where the attacking force's size, the defender's size, and the territory's inherent defense bonuses yield a mathematically guaranteed outcome.

**The Terra-Forge Twist: The Re-compile Delay**
Conquering a territory does not grant immediate control. When a territory is taken, it enters a "Compiling" state. The victor must spend an additional turn (or expend extra resources) to re-terraform the territory into their specific biome. Until this compilation is complete, the territory cannot generate resources, cannot be used to launch attacks, and offers zero defensive bonuses. 

**Win/Loss Conditions**
Total planetary dominance (controlling 100% of the map) constitutes a victory. A player is eliminated when their last active territory is overtaken and their core AI is wiped.

---

## 4. Platform & Social Features (The Metagame)

**User Identity & Authentication**
The platform supports persistent player identities. Users have profiles tracking their win/loss ratios, preferred factions, average game duration, and historical match logs.

**Matchmaking & Lobbies**
Players can create isolated game rooms (lobbies) with customizable parameters (map size, turn timers, player count). Lobbies can be public (discoverable via matchmaking) or private (invite-only).

**Spectator Engine**
The architecture supports a broadcast model. Non-playing users can connect to active matches as spectators. Because the game state is deterministic, the server only needs to stream the initial state and the subsequent ledger of actions, allowing thousands of spectators to watch with minimal server load.

**Competitive Ecosystem**
The platform features an Elo or MMR (Matchmaking Rating) system. Ranked queues match players of similar skill levels, creating a competitive ladder.

**Tournaments & Disputes**
Automated bracket systems allow for community or seasonal tournaments. Disputes are handled through the immutable event ledger, which can perfectly reconstruct any game for moderation review.

---

## 5. High-Level System Architecture

**The "Pragmatic Hybrid" Model**
The system avoids the friction of building UI elements in systems programming languages. Instead, it uses a strictly separated hybrid approach: a high-performance backend, a shared headless game engine, and a flexible, standard web-tech frontend.

**The Three-Tier Separation of Concerns**

*   **The Authoritative Backend API:** Manages the platform, database, network connections, and lobbies. It acts as the ultimate arbiter of truth.
*   **The Shared "Brain" (Headless Game Engine):** A pure logic module containing the rules of Terra-Forge. It has no knowledge of how to draw a map or how to send a network packet.
*   **The "Dumb" Visual Client:** The frontend application. It captures user inputs, passes them to the local Wasm brain for validation, and renders the output state to the screen.

**Data Flow Overview**
1.  User clicks "Attack".
2.  Frontend checks the local Wasm engine: "Is this valid?"
3.  If valid, Frontend renders immediate feedback and sends the "Attack Intent" to the Backend.
4.  Backend feeds the Intent to its native Engine.
5.  Backend resolves combat and broadcasts the resulting "State Mutation Event" to all clients and spectators.
6.  Clients feed the Event into their local Wasm engine to update the visual board.

---

## 6. Project Modules & Boundaries

### Module A: Core Engine (Shared)
*   **Domain Entities:** Defines the strict, flat data structures for `Map`, `Territory`, `Player`, and `Unit` (avoiding heavy ECS frameworks for simplicity and fast state serialization).
*   **Rule Validation:** Functions that verify if a move is legal based on adjacency, unit count, and turn phase.
*   **State Transitions:** The pure functions that calculate combat losses and terraforming status, returning a new immutable game state.
*   **Constraint:** This module must be completely platform-agnostic, containing zero I/O, database, or UI dependencies.

### Module B: Platform Server (Backend)
*   **Session Management:** Handles user authentication tokens and active connections.
*   **Lobby Management:** Groups connections into distinct match instances.
*   **Network I/O:** Manages WebSocket lifecycles and HTTP REST endpoints for out-of-game data.
*   **State Authority:** Maintains the active memory of all ongoing games and acts as the singular broadcaster of state changes.

### Module C: Frontend Application (Client)
*   **Wasm Integration Layer:** The bridge that allows the web application to synchronously call functions on the compiled Core Engine.
*   **Input Translation:** Converts mouse clicks, touches, and keyboard events into structured game actions.
*   **Render Pipeline:** A graphics implementation (e.g., Canvas/WebGL) that reads the state from the Wasm module and draws the hexes, animations, and UI overlays.
*   **Out-of-Game UI:** The standard web interfaces (HTML/CSS/TS) for leaderboards, user profiles, and lobby creation.

---

## 7. State Management & Networking Strategy

**The Event Sourcing Pattern**
The system does not synchronize the entire game board every second. Instead, it relies on Event Sourcing. The initial game state is the baseline. Every valid action generates an immutable Event (e.g., `CombatResolved`, `TurnEnded`).

**Achieving Perfect Client/Server Sync**
Because both the server and the client run the exact same compiled Engine, feeding the exact same Event into both will yield the exact same resulting board state.

**Handling Latency and Client-Side Prediction**
When a player makes a move, the Wasm engine validates it instantly. The client can optimistically render the attack animation immediately, masking network latency. If the server subsequently rejects the move, the client simply rolls back to the server's authoritative state.

**Managing Randomness**
To maintain determinism in scenarios that require shuffling (like map generation or spawn points), the system uses a shared Pseudo-Random Number Generator (PRNG). The server dictates a single Seed value at the start of the match, ensuring all clients generate the exact same "random" sequences.

---

## 8. Artificial Intelligence Strategy (Future-Proofing)

**Why the Architecture Enables Advanced AI**
Because Module A (The Core Engine) is entirely decoupled from the network and the UI, it can be executed in a tight `while` loop at maximum CPU speed. 

**Headless Simulations**
An AI bot running on the server can clone the current game state and simulate thousands of potential future turns in milliseconds to evaluate the best possible move.

**Approaches for the Bot**
The decoupled nature allows for seamless integration of sophisticated algorithms like Monte Carlo Tree Search (MCTS) or Minimax with Alpha-Beta pruning, which are standard in complex board game AI, vastly outperforming simple rule-based heuristics.

---

## 9. Proposed Development Roadmap (Iterative Phasing)

*   **Phase 1: The Headless Core.** Developing the pure logic engine. Establishing the flat data structures, the graph map, and the combat math. Success is defined by passing automated unit tests, with no visual output.
*   **Phase 2: CLI & Local Prototyping.** Hooking the engine up to a simple text-based terminal interface or a very crude local visualizer to play test the "Re-compile" delay mechanic and balance the rules.
*   **Phase 3: The Multiplayer Backbone.** Building the backend server, establishing WebSocket connections, and proving that two separate headless clients can stay synchronized via Event Sourcing.
*   **Phase 4: Full Frontend Rendering.** Integrating the Wasm payload into a browser environment and building out the visual map, animations, and interaction layer.
*   **Phase 5: Platform Expansion.** Implementing the metagame: user accounts, database persistence, matchmaking lobbies, and spectator modes.
*   **Phase 6: The AI Factions.** Building the autonomous bots, plugging them into the server architecture, and refining their strategic decision-making trees.