use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::faction::Faction;
use super::ids::{ContinentId, PlayerId, TerritoryId};

/// Whether a territory is fully under its owner's control or still subject
/// to the Re-compile Delay (task 1.8) after a recent capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum TerritoryStatus {
    Active,
    Compiling,
}

/// A single territory on the map. Per RFC-001 §7, attribute-for-attribute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Territory {
    pub id: TerritoryId,
    pub continent_id: ContinentId,
    pub owner_id: Option<PlayerId>,
    pub faction: Option<Faction>,
    pub unit_count: u32,
    pub status: TerritoryStatus,
    pub adjacent_territory_ids: Vec<TerritoryId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ulid::Ulid;

    #[test]
    fn territory_status_round_trips_through_serde() {
        for status in [TerritoryStatus::Active, TerritoryStatus::Compiling] {
            let json = serde_json::to_string(&status).unwrap();
            let decoded: TerritoryStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, decoded);
        }
    }

    #[test]
    fn territory_round_trips_through_serde() {
        let territory = Territory {
            id: TerritoryId(Ulid::from_u128(1)),
            continent_id: ContinentId(Ulid::from_u128(2)),
            owner_id: Some(PlayerId(Ulid::from_u128(3))),
            faction: Some(Faction::MagmaForge),
            unit_count: 5,
            status: TerritoryStatus::Active,
            adjacent_territory_ids: vec![TerritoryId(Ulid::from_u128(4))],
        };

        let json = serde_json::to_string(&territory).unwrap();
        let decoded: Territory = serde_json::from_str(&json).unwrap();
        assert_eq!(territory, decoded);
    }

    #[test]
    fn territory_round_trips_with_no_owner() {
        let territory = Territory {
            id: TerritoryId(Ulid::from_u128(10)),
            continent_id: ContinentId(Ulid::from_u128(20)),
            owner_id: None,
            faction: None,
            unit_count: 0,
            status: TerritoryStatus::Compiling,
            adjacent_territory_ids: vec![],
        };

        let json = serde_json::to_string(&territory).unwrap();
        let decoded: Territory = serde_json::from_str(&json).unwrap();
        assert_eq!(territory, decoded);
    }
}
