//! Deterministic combat resolution (RFC-001 UC3, RFC-002 §5.1). The
//! attrition formula and the defense bonus's flat-constant data source are
//! decided in [ADR-0001](../../../docs/adr/0001-deterministic-combat-attrition-formula.md):
//! a sequential-exchange closed form, computed directly rather than
//! resolved dice-roll-by-dice-roll, to stay a pure function.
//!
//! `resolve_combat` is pure: identical inputs always produce identical
//! outputs, with no float, no `HashMap` iteration order, and no panic for
//! any `u32` input (including `defense_bonus` up to `u32::MAX`) — every
//! operation below is `saturating_add`/`saturating_sub`/`min`.
//!
//! `CombatOutcome` deliberately does not live under `engine::protocol`: it
//! never crosses the Engine boundary at MVP. Server and every client run
//! the identical compiled Engine and replay the same `Attack` `GameAction`
//! from the `EventLog`, so each side recomputes the same outcome locally
//! instead of receiving it over the wire (mirrors `match_state.rs`'s
//! placement rationale).

use crate::faction_modifiers::FactionModifiers;
use crate::protocol::Faction;

/// The flat defense bonus applied to every defending territory (ADR-0001).
/// A tuning value, not an architectural one — free to change without a new
/// ADR as long as it stays a single constant, not per-territory data.
pub const DEFAULT_DEFENSE_BONUS: u32 = 1;

/// The result of resolving one `Attack` (task 1.7 owns the call site).
/// ADR-0001 documents two "quirk" states this can produce: a held-but-
/// emptied defender (defense bonus alone kept the territory, even though
/// `defender_remaining == 0`), and a Pyrrhic capture (the attacker's force
/// is fully consumed in the same exchange that wins the territory).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatOutcome {
    pub attacker_remaining: u32,
    pub defender_remaining: u32,
    pub territory_captured: bool,
}

/// Resolves combat via ADR-0001's sequential-exchange closed form. Looks up
/// `modifiers[attacker_faction]`/`modifiers[defender_faction]` (task 1.3's
/// seam) so it reaches this function, even though Phase 1's uniform table
/// makes both lookups identical and `ModifierSet` has no fields yet.
pub fn resolve_combat(
    attacker_units: u32,
    attacker_faction: Faction,
    defender_units: u32,
    defender_faction: Faction,
    defense_bonus: u32,
    modifiers: &FactionModifiers,
) -> CombatOutcome {
    let _attacker_modifiers = modifiers.get(attacker_faction);
    let _defender_modifiers = modifiers.get(defender_faction);

    let defender_effective = defender_units.saturating_add(defense_bonus);
    let exchanged = attacker_units.min(defender_effective);

    let attacker_remaining = attacker_units - exchanged;
    let defender_effective_remaining = defender_effective - exchanged;

    if defender_effective_remaining == 0 {
        CombatOutcome {
            attacker_remaining,
            defender_remaining: 0,
            territory_captured: true,
        }
    } else {
        CombatOutcome {
            attacker_remaining,
            defender_remaining: defender_effective_remaining.saturating_sub(defense_bonus),
            territory_captured: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_forces_with_no_bonus_favor_the_attacker() {
        let modifiers = FactionModifiers::uniform();
        let outcome = resolve_combat(
            5,
            Faction::SiliconSwarm,
            5,
            Faction::SporeColony,
            0,
            &modifiers,
        );

        assert_eq!(
            outcome,
            CombatOutcome {
                attacker_remaining: 0,
                defender_remaining: 0,
                territory_captured: true,
            }
        );
    }

    #[test]
    fn attacker_vastly_outnumbered_bounces_off() {
        let modifiers = FactionModifiers::uniform();
        let outcome = resolve_combat(
            2,
            Faction::SiliconSwarm,
            20,
            Faction::SporeColony,
            5,
            &modifiers,
        );

        assert_eq!(
            outcome,
            CombatOutcome {
                attacker_remaining: 0,
                defender_remaining: 18,
                territory_captured: false,
            }
        );
    }

    #[test]
    fn defender_vastly_outnumbered_is_captured_with_survivors() {
        let modifiers = FactionModifiers::uniform();
        let outcome = resolve_combat(
            20,
            Faction::SiliconSwarm,
            2,
            Faction::SporeColony,
            1,
            &modifiers,
        );

        assert_eq!(
            outcome,
            CombatOutcome {
                attacker_remaining: 17,
                defender_remaining: 0,
                territory_captured: true,
            }
        );
    }

    #[test]
    fn minimum_legal_attack_size_does_not_panic() {
        let modifiers = FactionModifiers::uniform();
        let outcome = resolve_combat(
            1,
            Faction::SiliconSwarm,
            3,
            Faction::SporeColony,
            1,
            &modifiers,
        );

        assert_eq!(
            outcome,
            CombatOutcome {
                attacker_remaining: 0,
                defender_remaining: 2,
                territory_captured: false,
            }
        );
    }

    #[test]
    fn full_capture_defender_reduced_to_zero() {
        let modifiers = FactionModifiers::uniform();
        let outcome = resolve_combat(
            10,
            Faction::SiliconSwarm,
            3,
            Faction::SporeColony,
            1,
            &modifiers,
        );

        assert_eq!(
            outcome,
            CombatOutcome {
                attacker_remaining: 6,
                defender_remaining: 0,
                territory_captured: true,
            }
        );
    }

    #[test]
    fn pyrrhic_capture_consumes_entire_attacking_force() {
        let modifiers = FactionModifiers::uniform();
        let outcome = resolve_combat(
            6,
            Faction::SiliconSwarm,
            3,
            Faction::SporeColony,
            3,
            &modifiers,
        );

        assert_eq!(
            outcome,
            CombatOutcome {
                attacker_remaining: 0,
                defender_remaining: 0,
                territory_captured: true,
            }
        );
    }

    #[test]
    fn defense_bonus_at_u32_max_does_not_panic() {
        let modifiers = FactionModifiers::uniform();
        let outcome = resolve_combat(
            10,
            Faction::SiliconSwarm,
            3,
            Faction::SporeColony,
            u32::MAX,
            &modifiers,
        );

        assert_eq!(
            outcome,
            CombatOutcome {
                attacker_remaining: 0,
                defender_remaining: 0,
                territory_captured: false,
            }
        );
    }

    #[test]
    fn same_inputs_called_twice_produce_identical_outputs() {
        let modifiers = FactionModifiers::uniform();
        let first = resolve_combat(
            7,
            Faction::CryoArchitects,
            4,
            Faction::MagmaForge,
            2,
            &modifiers,
        );
        let second = resolve_combat(
            7,
            Faction::CryoArchitects,
            4,
            Faction::MagmaForge,
            2,
            &modifiers,
        );

        assert_eq!(first, second);
    }

    #[test]
    fn outcome_is_uniform_across_faction_pairings() {
        let modifiers = FactionModifiers::uniform();
        let baseline = resolve_combat(
            5,
            Faction::SiliconSwarm,
            3,
            Faction::SporeColony,
            1,
            &modifiers,
        );
        let other = resolve_combat(
            5,
            Faction::MagmaForge,
            3,
            Faction::CryoArchitects,
            1,
            &modifiers,
        );

        assert_eq!(baseline, other);
    }
}
