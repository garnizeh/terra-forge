# Terra-Forge

A deterministic, web-native multiplayer strategy game — area-control in the vein of *Risk*/*War*, wrapped in a sci-fi "terraforming AI factions" theme. Built as a learning/development sandbox emphasizing architectural elegance.

**Status: design phase closed, MVP implementation starting.** No source code exists yet — see [Project status](#project-status) below.

## What makes it different

- **No RNG in combat.** Outcomes are resolved by a deterministic attrition matrix (attacker size, defender size, territory defense bonus → guaranteed result). Any "random"-looking behavior (map generation, spawn points) goes through a single seeded PRNG shared by server and clients.
- **The Re-compile Delay.** Capturing a territory doesn't grant immediate control — it enters a `Compiling` state that produces nothing, can't attack, and has zero defense, until the owner either waits it out or commits units to accelerate it.
- **Isomorphic core.** The game rules are written once in Rust and compiled both to a native binary (server) and to WebAssembly (browser client) from the same source, so the client can validate and predict moves locally while the server stays the single source of truth.

## Documentation

The design lives entirely in `docs/` and is the authoritative source for anything not yet reflected in code:

| Document | Covers |
|---|---|
| [docs/idea/01_first_idea.md](docs/idea/01_first_idea.md) | Original concept — historical/superseded, kept for provenance |
| [docs/rfc/rfc-0001](docs/rfc/rfc-0001_terra-forge_product_design_specification.md) | Product spec: user journeys, domain model, non-functional requirements |
| [docs/rfc/rfc-0002](docs/rfc/rfc-0002_terra-forge_high_level_architecture.md) | Technical architecture: technology choices, protocols, data/cache layer |
| [docs/rfc/rfc-0003](docs/rfc/rfc-0003_terra-forge_mvp_definition.md) | **MVP scope** — what ships first and what's explicitly deferred |
| [CLAUDE.md](CLAUDE.md) | Working summary of the above, kept current as implementation progresses |

## Project status

RFC-001 through RFC-003 are complete and their open questions resolved. Implementation follows a phased build order (full detail in [CLAUDE.md](CLAUDE.md)):

0. ~~Design documentation~~ — done
1. **Headless Core Engine** (pure Rust game logic, unit-tested, no visuals) — current phase
2. CLI/local prototype to playtest the rules
3. Multiplayer backbone (backend server, WebSockets, event-sourced sync)
4. Full frontend rendering (Wasm in-browser) — **MVP ships here**
5. Platform expansion (accounts, persistence, matchmaking, spectator mode)
6. AI factions/bots

There is nothing to build, run, or test yet.

## License

[MIT](LICENSE)
