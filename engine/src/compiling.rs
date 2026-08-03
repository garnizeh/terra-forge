//! The Re-compile Delay (CLAUDE.md's signature mechanic, RFC-001 UC3 step
//! 4): a captured territory doesn't grant immediate full control — it
//! enters `Compiling` until resolved either by waiting (free, at the
//! owner's next Compile phase) or by `AccelerateCompile` (paid, consuming
//! units from that Compile phase's freshly-generated pool rather than
//! depositing them).
//!
//! Three of `Compiling`'s four effects are already enforced elsewhere: no
//! generation / no continent bonus (`unit_generation::calculate_generation`,
//! task 1.6 — excludes non-`Active` territories from both), and unusable as
//! an attack source (`legality::validate_attack`, task 1.7 — rejects a
//! non-`Active` source). This module owns the fourth (zero defense as a
//! target, via [`effective_defense_bonus`]) plus the actual state
//! transitions: applying a capture, and both resolution paths.
//!
//! Neither this module nor anything merged so far tracks which `Faction` a
//! `PlayerId` has chosen — RFC-001 ties that to `PlayerProfile.
//! preferred_faction`, an explicitly deferred, non-MVP entity. Rather than
//! invent a lookup, [`complete_wait`]/[`complete_accelerate`] take the
//! owner's `Faction` as an explicit parameter, mirroring
//! `combat::resolve_combat`'s existing `attacker_faction`/`defender_faction`
//! parameters.

use crate::combat::CombatOutcome;
use crate::legality::ACCELERATE_COMPILE_COST;
use crate::protocol::{Faction, PlayerId, Territory, TerritoryStatus};
use crate::unit_generation::{InsufficientUnitsError, UnitPool};

/// Why a `Compiling`-related transition was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilingError {
    NotCaptured,
    NotCompiling,
    InsufficientPool(InsufficientUnitsError),
}

/// Applies a successful `Attack` capture (task 1.5's `CombatOutcome`) to the
/// captured `Territory`: transfers ownership, enters `Compiling`, and sets
/// `unit_count` to the surviving attacking force that now occupies the
/// ground. `faction` is deliberately left untouched — it only updates once
/// conversion completes (RFC-001 UC3).
///
/// Errors with `NotCaptured`, leaving `territory` unchanged, if `outcome`
/// doesn't actually represent a capture.
pub fn apply_capture(
    territory: &mut Territory,
    new_owner: PlayerId,
    outcome: &CombatOutcome,
) -> Result<(), CompilingError> {
    if !outcome.territory_captured {
        return Err(CompilingError::NotCaptured);
    }

    territory.owner_id = Some(new_owner);
    territory.status = TerritoryStatus::Compiling;
    territory.unit_count = outcome.attacker_remaining;

    Ok(())
}

/// The free resolution path: completes conversion at no unit cost. `status`
/// becomes `Active` and `faction` updates to the owner's.
///
/// Errors with `NotCompiling`, leaving `territory` unchanged, if it isn't
/// currently `Compiling`.
pub fn complete_wait(
    territory: &mut Territory,
    owner_faction: Faction,
) -> Result<(), CompilingError> {
    if territory.status != TerritoryStatus::Compiling {
        return Err(CompilingError::NotCompiling);
    }

    territory.status = TerritoryStatus::Active;
    territory.faction = Some(owner_faction);

    Ok(())
}

/// The paid resolution path: consumes [`ACCELERATE_COMPILE_COST`] units from
/// that Compile phase's freshly-generated `pool` (task 1.6) and completes
/// conversion immediately, same as [`complete_wait`]. The consumed units are
/// never added to `territory.unit_count` — acceleration is a trade-off
/// against board presence, not a free action (RFC-001 UC3).
///
/// The caller (task 1.7's `validate_accelerate_compile`) is assumed to have
/// already confirmed the pool can afford this; `InsufficientPool` is
/// propagated defensively rather than assumed impossible.
///
/// Errors with `NotCompiling`, leaving `territory` and `pool` unchanged, if
/// `territory` isn't currently `Compiling`.
pub fn complete_accelerate(
    territory: &mut Territory,
    owner_faction: Faction,
    pool: &mut UnitPool,
) -> Result<(), CompilingError> {
    if territory.status != TerritoryStatus::Compiling {
        return Err(CompilingError::NotCompiling);
    }

    pool.consume(ACCELERATE_COMPILE_COST)
        .map_err(CompilingError::InsufficientPool)?;

    territory.status = TerritoryStatus::Active;
    territory.faction = Some(owner_faction);

    Ok(())
}

/// A `Compiling` territory confers zero defense bonus if attacked,
/// regardless of the base defense-bonus source (ADR-0001's flat constant or
/// whatever it becomes later) — this is the "combat/task 1.8 concern"
/// `combat.rs` flags rather than resolving itself.
pub fn effective_defense_bonus(territory: &Territory, base_defense_bonus: u32) -> u32 {
    if territory.status == TerritoryStatus::Compiling {
        0
    } else {
        base_defense_bonus
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legality::{self, LegalityError};
    use crate::match_state::{MatchState, Phase};
    use crate::protocol::{
        ActionType, Continent, ContinentId, GameAction, MapId, TerritoryId, Ulid,
    };
    use crate::unit_generation::calculate_generation;

    fn player(n: u128) -> PlayerId {
        PlayerId(Ulid::from_u128(n))
    }

    fn territory_id(n: u128) -> TerritoryId {
        TerritoryId(Ulid::from_u128(n))
    }

    fn territory(
        id: TerritoryId,
        owner: Option<PlayerId>,
        faction: Option<Faction>,
        status: TerritoryStatus,
        unit_count: u32,
        adjacent: Vec<TerritoryId>,
    ) -> Territory {
        Territory {
            id,
            continent_id: ContinentId(Ulid::from_u128(1000)),
            owner_id: owner,
            faction,
            unit_count,
            status,
            adjacent_territory_ids: adjacent,
        }
    }

    fn captured_outcome(attacker_remaining: u32) -> CombatOutcome {
        CombatOutcome {
            attacker_remaining,
            defender_remaining: 0,
            territory_captured: true,
        }
    }

    // ---- apply_capture ----

    #[test]
    fn apply_capture_transfers_ownership_and_enters_compiling() {
        let attacker = player(1);
        let mut t = territory(
            territory_id(1),
            None,
            Some(Faction::SporeColony),
            TerritoryStatus::Active,
            0,
            vec![],
        );
        let outcome = captured_outcome(6);

        assert_eq!(apply_capture(&mut t, attacker, &outcome), Ok(()));
        assert_eq!(t.owner_id, Some(attacker));
        assert_eq!(t.status, TerritoryStatus::Compiling);
        assert_eq!(t.unit_count, 6);
        assert_eq!(
            t.faction,
            Some(Faction::SporeColony),
            "faction must not update until conversion completes"
        );
    }

    #[test]
    fn apply_capture_rejects_a_non_capture_outcome() {
        let attacker = player(1);
        let mut t = territory(
            territory_id(1),
            Some(player(2)),
            Some(Faction::SporeColony),
            TerritoryStatus::Active,
            5,
            vec![],
        );
        let original = t.clone();
        let outcome = CombatOutcome {
            attacker_remaining: 0,
            defender_remaining: 3,
            territory_captured: false,
        };

        assert_eq!(
            apply_capture(&mut t, attacker, &outcome),
            Err(CompilingError::NotCaptured)
        );
        assert_eq!(
            t, original,
            "rejected capture must not mutate the territory"
        );
    }

    // ---- complete_wait ----

    #[test]
    fn complete_wait_activates_and_updates_faction() {
        let owner = player(1);
        let mut t = territory(
            territory_id(1),
            Some(owner),
            None,
            TerritoryStatus::Compiling,
            6,
            vec![],
        );

        assert_eq!(complete_wait(&mut t, Faction::MagmaForge), Ok(()));
        assert_eq!(t.status, TerritoryStatus::Active);
        assert_eq!(t.faction, Some(Faction::MagmaForge));
        assert_eq!(t.unit_count, 6, "unit_count is untouched by resolution");
    }

    #[test]
    fn complete_wait_rejects_a_non_compiling_territory() {
        let owner = player(1);
        let mut t = territory(
            territory_id(1),
            Some(owner),
            Some(Faction::MagmaForge),
            TerritoryStatus::Active,
            6,
            vec![],
        );
        let original = t.clone();

        assert_eq!(
            complete_wait(&mut t, Faction::MagmaForge),
            Err(CompilingError::NotCompiling)
        );
        assert_eq!(t, original);
    }

    #[test]
    fn capture_then_wait_round_trip() {
        let attacker = player(1);
        let mut t = territory(
            territory_id(1),
            None,
            Some(Faction::CryoArchitects),
            TerritoryStatus::Active,
            0,
            vec![],
        );
        let outcome = captured_outcome(4);

        apply_capture(&mut t, attacker, &outcome).unwrap();
        assert_eq!(t.status, TerritoryStatus::Compiling);

        // Simulate reaching the attacker's next Compile phase.
        complete_wait(&mut t, Faction::SiliconSwarm).unwrap();
        assert_eq!(t.status, TerritoryStatus::Active);
        assert_eq!(t.owner_id, Some(attacker));
        assert_eq!(t.faction, Some(Faction::SiliconSwarm));
        assert_eq!(t.unit_count, 4);
    }

    // ---- complete_accelerate ----

    #[test]
    fn complete_accelerate_activates_and_consumes_pool_only() {
        let owner = player(1);
        let mut t = territory(
            territory_id(1),
            Some(owner),
            None,
            TerritoryStatus::Compiling,
            6,
            vec![],
        );
        let mut pool = UnitPool::new(5);

        assert_eq!(
            complete_accelerate(&mut t, Faction::MagmaForge, &mut pool),
            Ok(())
        );
        assert_eq!(t.status, TerritoryStatus::Active);
        assert_eq!(t.faction, Some(Faction::MagmaForge));
        assert_eq!(
            t.unit_count, 6,
            "consumed units are never deposited into unit_count"
        );
        assert_eq!(pool.remaining(), 5 - ACCELERATE_COMPILE_COST);
    }

    #[test]
    fn complete_accelerate_rejects_a_non_compiling_territory() {
        let owner = player(1);
        let mut t = territory(
            territory_id(1),
            Some(owner),
            Some(Faction::MagmaForge),
            TerritoryStatus::Active,
            6,
            vec![],
        );
        let original = t.clone();
        let mut pool = UnitPool::new(5);

        assert_eq!(
            complete_accelerate(&mut t, Faction::MagmaForge, &mut pool),
            Err(CompilingError::NotCompiling)
        );
        assert_eq!(t, original);
        assert_eq!(pool.remaining(), 5, "pool must not be touched");
    }

    #[test]
    fn complete_accelerate_propagates_insufficient_pool() {
        let owner = player(1);
        let mut t = territory(
            territory_id(1),
            Some(owner),
            None,
            TerritoryStatus::Compiling,
            6,
            vec![],
        );
        let mut pool = UnitPool::new(1);

        assert_eq!(
            complete_accelerate(&mut t, Faction::MagmaForge, &mut pool),
            Err(CompilingError::InsufficientPool(InsufficientUnitsError {
                requested: ACCELERATE_COMPILE_COST,
                remaining: 1,
            }))
        );
        assert_eq!(
            t.status,
            TerritoryStatus::Compiling,
            "rejected, must not mutate"
        );
        assert_eq!(pool.remaining(), 1);
    }

    #[test]
    fn capture_then_accelerate_same_turn_round_trip() {
        let attacker = player(1);
        let mut t = territory(
            territory_id(1),
            None,
            Some(Faction::CryoArchitects),
            TerritoryStatus::Active,
            0,
            vec![],
        );
        let outcome = captured_outcome(4);
        let mut pool = UnitPool::new(ACCELERATE_COMPILE_COST);

        apply_capture(&mut t, attacker, &outcome).unwrap();
        assert_eq!(t.status, TerritoryStatus::Compiling);

        complete_accelerate(&mut t, Faction::SiliconSwarm, &mut pool).unwrap();
        assert_eq!(t.status, TerritoryStatus::Active);
        assert_eq!(t.faction, Some(Faction::SiliconSwarm));
        assert_eq!(t.unit_count, 4);
        assert_eq!(pool.remaining(), 0);
    }

    // ---- effective_defense_bonus ----

    #[test]
    fn effective_defense_bonus_returns_base_value_for_active_territory() {
        let t = territory(
            territory_id(1),
            Some(player(1)),
            Some(Faction::MagmaForge),
            TerritoryStatus::Active,
            5,
            vec![],
        );

        assert_eq!(effective_defense_bonus(&t, 1), 1);
        assert_eq!(effective_defense_bonus(&t, u32::MAX), u32::MAX);
    }

    #[test]
    fn effective_defense_bonus_is_zero_for_compiling_territory_regardless_of_base() {
        let t = territory(
            territory_id(1),
            Some(player(1)),
            None,
            TerritoryStatus::Compiling,
            5,
            vec![],
        );

        assert_eq!(effective_defense_bonus(&t, 1), 0);
        assert_eq!(effective_defense_bonus(&t, u32::MAX), 0);
    }

    // ---- Integration: all four Compiling behaviors together ----

    #[test]
    fn compiling_territory_is_excluded_from_generation_bonus_attack_source_and_defense() {
        let owner = player(1);
        let compiling_id = territory_id(1);
        let active_id = territory_id(2);
        let continent = ContinentId(Ulid::from_u128(1));

        let compiling = Territory {
            id: compiling_id,
            continent_id: continent,
            owner_id: Some(owner),
            faction: None,
            unit_count: 5,
            status: TerritoryStatus::Compiling,
            adjacent_territory_ids: vec![active_id],
        };
        let active = Territory {
            id: active_id,
            continent_id: continent,
            owner_id: Some(owner),
            faction: Some(Faction::MagmaForge),
            unit_count: 3,
            status: TerritoryStatus::Active,
            adjacent_territory_ids: vec![compiling_id],
        };
        let territories = vec![compiling.clone(), active.clone()];
        let continents = vec![Continent {
            id: continent,
            map_id: MapId(Ulid::from_u128(9)),
            name: "Test".to_string(),
            control_bonus: 10,
        }];

        // No generation and no continent bonus: only the Active territory
        // counts toward the base, and the continent's bonus is withheld
        // because the Compiling territory breaks full control.
        assert_eq!(calculate_generation(owner, &territories, &continents), 1);

        // Can't be an attack source.
        let mut match_state = MatchState::new(vec![owner]);
        match_state.advance(); // Execute phase
        assert_eq!(match_state.current_phase(), Phase::Execute);
        let attack = GameAction {
            action_type: ActionType::Attack,
            actor_id: owner,
            source_territory_id: compiling_id,
            target_territory_id: Some(active_id),
            unit_count: 1,
        };
        assert_eq!(
            legality::validate_attack(&attack, &match_state, &territories),
            Err(LegalityError::TerritoryNotActive)
        );

        // Confers zero defense bonus if attacked as a target.
        assert_eq!(effective_defense_bonus(&compiling, 1), 0);
    }
}
