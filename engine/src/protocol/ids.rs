//! Opaque, distinct ID newtypes for every entity in this module.
//!
//! Each wraps a [`Ulid`], but the types are intentionally *not*
//! interchangeable — passing a `ContinentId` where a `TerritoryId` is
//! expected is a compile error, not a runtime bug.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::ulid::Ulid;

macro_rules! id_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(pub Ulid);

        impl TS for $name {
            type WithoutGenerics = Self;
            type OptionInnerType = Self;

            fn name(_: &ts_rs::Config) -> String {
                String::from("string")
            }

            fn inline(cfg: &ts_rs::Config) -> String {
                <Self as TS>::name(cfg)
            }
        }
    };
}

id_newtype!(TerritoryId);
id_newtype!(ContinentId);
id_newtype!(MapId);

id_newtype!(
    /// References the match a `Map` belongs to. `MatchInstance` itself is a
    /// Phase 3 / Platform Server concern (RFC-001 §7) and is not modeled
    /// here — this newtype only lets `Map.match_id` exist as an opaque
    /// reference.
    MatchId
);

id_newtype!(
    /// An opaque, per-match player identifier.
    ///
    /// This is deliberately *not* RFC-001's `User` entity: per RFC-003 §6,
    /// MVP identity is an ephemeral session scoped to a single match, with
    /// no account, profile, or cross-match history. The Engine must not
    /// assume a `User` row exists anywhere.
    PlayerId
);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! round_trip_test {
        ($test_name:ident, $ty:ident) => {
            #[test]
            fn $test_name() {
                let id = $ty(Ulid::from_u128(123456789));
                let json = serde_json::to_string(&id).unwrap();
                let decoded: $ty = serde_json::from_str(&json).unwrap();
                assert_eq!(id, decoded);
            }
        };
    }

    round_trip_test!(territory_id_round_trips, TerritoryId);
    round_trip_test!(continent_id_round_trips, ContinentId);
    round_trip_test!(map_id_round_trips, MapId);
    round_trip_test!(match_id_round_trips, MatchId);
    round_trip_test!(player_id_round_trips, PlayerId);
}
