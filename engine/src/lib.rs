//! Terra-Forge Core Engine.
//!
//! Pure game-logic library, compiled from this same source to both a native
//! `rlib` (linked into the Platform Server) and `wasm32-unknown-unknown`
//! (loaded by the frontend via `wasm-pack`).
//!
//! Constraints, enforced by review and dependency-tree inspection rather
//! than convention alone:
//! - Zero I/O: no `std::net`, no filesystem access beyond pure computation.
//! - Zero DB, zero UI/network dependencies.
//! - Zero wall-clock time: no `std::time::Instant` / `SystemTime`. Turn
//!   deadlines are a Platform Server concern — the Engine only knows
//!   phases, never deadlines.

pub mod faction_modifiers;
pub mod match_state;
pub mod protocol;
