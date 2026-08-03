use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::ids::{ContinentId, MapId, MatchId};

/// A grouping of territories; `control_bonus` applies to a player's Compile
/// phase only when every child `Territory` is owned by them and `Active`
/// (RFC-001 §7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Continent {
    pub id: ContinentId,
    pub map_id: MapId,
    pub name: String,
    pub control_bonus: u32,
}

/// Placeholder for a future procedural map generator's sizing parameters.
///
/// RFC-003 §9's MVP maps are hand-authored presets (task 1.10), which
/// construct a `Map` directly and don't consume this field at all. Its
/// concrete shape is intentionally left unspecified until a PRNG-seeded
/// generator (post-MVP) needs it — see task 1.2's acceptance criteria.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MapSizeConfig {}

/// A match's map: one `Map` per `MatchInstance`, generated once at match
/// start (RFC-001 §7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Map {
    pub id: MapId,
    pub match_id: MatchId,
    pub size_config: MapSizeConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ulid::Ulid;

    #[test]
    fn continent_round_trips_through_serde() {
        let continent = Continent {
            id: ContinentId(Ulid::from_u128(1)),
            map_id: MapId(Ulid::from_u128(2)),
            name: "Sable Reaches".to_string(),
            control_bonus: 3,
        };

        let json = serde_json::to_string(&continent).unwrap();
        let decoded: Continent = serde_json::from_str(&json).unwrap();
        assert_eq!(continent, decoded);
    }

    #[test]
    fn map_size_config_round_trips_through_serde() {
        let config = MapSizeConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let decoded: MapSizeConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, decoded);
    }

    #[test]
    fn map_round_trips_through_serde() {
        let map = Map {
            id: MapId(Ulid::from_u128(1)),
            match_id: MatchId(Ulid::from_u128(2)),
            size_config: MapSizeConfig::default(),
        };

        let json = serde_json::to_string(&map).unwrap();
        let decoded: Map = serde_json::from_str(&json).unwrap();
        assert_eq!(map, decoded);
    }
}
