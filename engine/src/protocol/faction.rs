use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A terraforming AI faction. Purely visual/flavor at MVP — see RFC-001 §2
/// and RFC-002 §5.1's `FactionModifiers` extension seam (task 1.3) for how
/// mechanical asymmetry is deferred without blocking this enum's shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Faction {
    SiliconSwarm,
    SporeColony,
    CryoArchitects,
    MagmaForge,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_serde() {
        for faction in [
            Faction::SiliconSwarm,
            Faction::SporeColony,
            Faction::CryoArchitects,
            Faction::MagmaForge,
        ] {
            let json = serde_json::to_string(&faction).unwrap();
            let decoded: Faction = serde_json::from_str(&json).unwrap();
            assert_eq!(faction, decoded);
        }
    }
}
