use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::ids::{PlayerId, TerritoryId};

/// Per RFC-001 §7: `Concede` is valid in any phase, from any active
/// participant; `AccelerateCompile` is valid only during its actor's
/// Compile phase. Legality itself is validated elsewhere (task 1.7) — this
/// enum only names the variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ActionType {
    Deploy,
    Attack,
    Fortify,
    Concede,
    AccelerateCompile,
}

/// A player-issued action, serialized as `EventLog.action_payload` for
/// `PlayerAction`-typed entries (RFC-001 §7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GameAction {
    pub action_type: ActionType,
    pub actor_id: PlayerId,
    pub source_territory_id: TerritoryId,
    pub target_territory_id: Option<TerritoryId>,
    pub unit_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ulid::Ulid;

    #[test]
    fn action_type_round_trips_through_serde() {
        for action_type in [
            ActionType::Deploy,
            ActionType::Attack,
            ActionType::Fortify,
            ActionType::Concede,
            ActionType::AccelerateCompile,
        ] {
            let json = serde_json::to_string(&action_type).unwrap();
            let decoded: ActionType = serde_json::from_str(&json).unwrap();
            assert_eq!(action_type, decoded);
        }
    }

    #[test]
    fn game_action_round_trips_through_serde() {
        let action = GameAction {
            action_type: ActionType::Attack,
            actor_id: PlayerId(Ulid::from_u128(1)),
            source_territory_id: TerritoryId(Ulid::from_u128(2)),
            target_territory_id: Some(TerritoryId(Ulid::from_u128(3))),
            unit_count: 7,
        };

        let json = serde_json::to_string(&action).unwrap();
        let decoded: GameAction = serde_json::from_str(&json).unwrap();
        assert_eq!(action, decoded);
    }

    #[test]
    fn game_action_round_trips_with_no_target() {
        let action = GameAction {
            action_type: ActionType::Concede,
            actor_id: PlayerId(Ulid::from_u128(1)),
            source_territory_id: TerritoryId(Ulid::from_u128(2)),
            target_territory_id: None,
            unit_count: 0,
        };

        let json = serde_json::to_string(&action).unwrap();
        let decoded: GameAction = serde_json::from_str(&json).unwrap();
        assert_eq!(action, decoded);
    }
}
