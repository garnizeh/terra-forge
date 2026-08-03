//! Wire-format types that cross the Engine boundary: `serde`+`ts-rs`-derived
//! structs and enums, kept separate from rule-evaluation logic (RFC-002
//! §5.1) so the wire-format types have one clear home.

mod action;
mod faction;
mod ids;
mod map;
mod territory;
mod ulid;

pub use action::{ActionType, GameAction};
pub use faction::Faction;
pub use ids::{ContinentId, MapId, MatchId, PlayerId, TerritoryId};
pub use map::{Continent, Map, MapSizeConfig};
pub use territory::{Territory, TerritoryStatus};
pub use ulid::{ParseUlidError, Ulid};
