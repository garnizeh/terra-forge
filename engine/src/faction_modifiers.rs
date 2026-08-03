//! The `FactionModifiers` lookup seam (RFC-001 §2, RFC-002 §5.1): rule
//! functions whose output could plausibly depend on faction take this as an
//! explicit parameter instead of hardcoding faction-agnostic math, so
//! turning on asymmetry later is a data change here, not a signature
//! change threaded through every call site.
//!
//! Phase 1 ships only the uniform, no-op table below — actual per-faction
//! values are an explicitly deferred RFC-001 §2 open question with no
//! assigned phase. No rule function in this crate reads a modifier value
//! yet (task 1.5's combat resolution will be the first); `ModifierSet` is
//! kept empty until a concrete consumer needs a field.

use crate::protocol::Faction;

/// Per-faction modifiers read by rule functions such as combat resolution.
/// Empty for now — see the module doc comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModifierSet {}

/// A complete lookup table from every `Faction` to its `ModifierSet`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FactionModifiers {
    silicon_swarm: ModifierSet,
    spore_colony: ModifierSet,
    cryo_architects: ModifierSet,
    magma_forge: ModifierSet,
}

impl FactionModifiers {
    /// The only table Phase 1 ships: every faction maps to an identical,
    /// neutral `ModifierSet`.
    pub fn uniform() -> Self {
        let modifiers = ModifierSet::default();
        Self {
            silicon_swarm: modifiers,
            spore_colony: modifiers,
            cryo_architects: modifiers,
            magma_forge: modifiers,
        }
    }

    /// Looks up the modifiers for a given faction.
    pub fn get(&self, faction: Faction) -> ModifierSet {
        match faction {
            Faction::SiliconSwarm => self.silicon_swarm,
            Faction::SporeColony => self.spore_colony,
            Faction::CryoArchitects => self.cryo_architects,
            Faction::MagmaForge => self.magma_forge,
        }
    }
}

impl Default for FactionModifiers {
    fn default() -> Self {
        Self::uniform()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_FACTIONS: [Faction; 4] = [
        Faction::SiliconSwarm,
        Faction::SporeColony,
        Faction::CryoArchitects,
        Faction::MagmaForge,
    ];

    #[test]
    fn uniform_table_is_genuinely_uniform() {
        let modifiers = FactionModifiers::uniform();
        let first = modifiers.get(ALL_FACTIONS[0]);
        for &faction in &ALL_FACTIONS[1..] {
            assert_eq!(
                modifiers.get(faction),
                first,
                "faction {faction:?} did not match the uniform baseline"
            );
        }
    }

    #[test]
    fn default_matches_uniform() {
        assert_eq!(FactionModifiers::default(), FactionModifiers::uniform());
    }
}
