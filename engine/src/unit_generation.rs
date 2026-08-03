//! Compile-phase unit generation and the per-Compile-phase unit pool
//! (RFC-001 UC3 step 1, RFC-001 §7). Generation is a flat 1:1 ratio of
//! `Active` (non-`Compiling`) territories controlled, plus `control_bonus`
//! for every fully-controlled `Continent` — a continent with even one
//! `Compiling` or foreign-owned territory withholds its bonus entirely.
//!
//! `UnitPool` does not itself live under `engine::protocol`: like
//! `match_state.rs` and `combat.rs`, it never crosses the Engine boundary at
//! MVP — `Deploy` (task 1.7) and `AccelerateCompile` (task 1.8) consume from
//! it directly, in-process. This module provides `UnitPool::consume` as a
//! mechanism only; no action handler calls it here.

use crate::protocol::{Continent, ContinentId, PlayerId, Territory, TerritoryStatus};

/// Computes `player_id`'s unit generation for one Compile phase: the count
/// of their `Active` territories, plus `control_bonus` for every `Continent`
/// they fully and actively control.
pub fn calculate_generation(
    player_id: PlayerId,
    territories: &[Territory],
    continents: &[Continent],
) -> u32 {
    let territory_count = territories
        .iter()
        .filter(|t| t.owner_id == Some(player_id) && t.status == TerritoryStatus::Active)
        .count() as u32;

    let continent_bonus = continents
        .iter()
        .filter(|c| continent_fully_controlled(c.id, player_id, territories))
        .fold(0u32, |acc, c| acc.saturating_add(c.control_bonus));

    territory_count.saturating_add(continent_bonus)
}

/// A continent is fully controlled by `player_id` when it has at least one
/// child territory and every child territory is both owned by `player_id`
/// and `Active` — an empty/malformed continent never grants a bonus (guards
/// the vacuous-truth case of an empty match on `Iterator::all`).
fn continent_fully_controlled(
    continent_id: ContinentId,
    player_id: PlayerId,
    territories: &[Territory],
) -> bool {
    let mut has_territory = false;
    for territory in territories
        .iter()
        .filter(|t| t.continent_id == continent_id)
    {
        has_territory = true;
        if territory.owner_id != Some(player_id) || territory.status != TerritoryStatus::Active {
            return false;
        }
    }
    has_territory
}

/// Returned by [`UnitPool::consume`] when `amount` exceeds the pool's
/// remaining balance; the pool is left unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InsufficientUnitsError {
    pub requested: u32,
    pub remaining: u32,
}

/// The freshly-generated unit pool for one Compile phase, shared by `Deploy`
/// (task 1.7) and `AccelerateCompile` (task 1.8). Resets each Compile phase
/// structurally — a caller constructs a fresh `UnitPool::new(generated)` on
/// entering the phase and discards the previous instance; there is no
/// `reset` method.
///
/// Deliberately not `Copy`: this represents one shared, mutable balance for
/// the phase, and callers (task 1.7/1.8) must hold and mutate a single
/// instance by reference (or explicit `.clone()`, never implicitly) so a
/// `consume` deduction is visible to every consumer of the same pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitPool {
    remaining: u32,
}

impl UnitPool {
    /// Initializes the pool with a Compile phase's freshly-generated amount.
    pub fn new(generated: u32) -> Self {
        Self {
            remaining: generated,
        }
    }

    pub fn remaining(&self) -> u32 {
        self.remaining
    }

    /// Atomically deducts `amount` from the remaining balance. Rejects
    /// (leaving the pool unchanged) rather than saturating when `amount`
    /// exceeds what remains — the pool must never go negative.
    pub fn consume(&mut self, amount: u32) -> Result<(), InsufficientUnitsError> {
        if amount > self.remaining {
            return Err(InsufficientUnitsError {
                requested: amount,
                remaining: self.remaining,
            });
        }
        self.remaining -= amount;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{MapId, TerritoryId, Ulid};

    fn player(n: u128) -> PlayerId {
        PlayerId(Ulid::from_u128(n))
    }

    fn continent_id(n: u128) -> ContinentId {
        ContinentId(Ulid::from_u128(n))
    }

    fn territory(
        id: u128,
        continent: ContinentId,
        owner: Option<PlayerId>,
        status: TerritoryStatus,
    ) -> Territory {
        Territory {
            id: TerritoryId(Ulid::from_u128(id)),
            continent_id: continent,
            owner_id: owner,
            faction: None,
            unit_count: 0,
            status,
            adjacent_territory_ids: vec![],
        }
    }

    fn continent(id: ContinentId, control_bonus: u32) -> Continent {
        Continent {
            id,
            map_id: MapId(Ulid::from_u128(999)),
            name: "Test Continent".to_string(),
            control_bonus,
        }
    }

    #[test]
    fn zero_territories_generates_zero_and_does_not_panic() {
        let p0 = player(1);
        let generated = calculate_generation(p0, &[], &[]);
        assert_eq!(generated, 0);
    }

    #[test]
    fn one_fully_controlled_continent_adds_territory_count_and_bonus() {
        let p0 = player(1);
        let c0 = continent_id(1);
        let territories = vec![
            territory(1, c0, Some(p0), TerritoryStatus::Active),
            territory(2, c0, Some(p0), TerritoryStatus::Active),
        ];
        let continents = vec![continent(c0, 3)];

        let generated = calculate_generation(p0, &territories, &continents);
        assert_eq!(generated, 2 + 3);
    }

    #[test]
    fn continent_minus_one_compiling_territory_withholds_bonus() {
        let p0 = player(1);
        let c0 = continent_id(1);
        let territories = vec![
            territory(1, c0, Some(p0), TerritoryStatus::Active),
            territory(2, c0, Some(p0), TerritoryStatus::Compiling),
        ];
        let continents = vec![continent(c0, 5)];

        let generated = calculate_generation(p0, &territories, &continents);
        // Only the one Active territory counts toward the base; the
        // Compiling territory contributes neither to the base nor the bonus.
        assert_eq!(generated, 1);
    }

    #[test]
    fn continent_minus_one_foreign_owned_territory_withholds_bonus() {
        let p0 = player(1);
        let p1 = player(2);
        let c0 = continent_id(1);
        let territories = vec![
            territory(1, c0, Some(p0), TerritoryStatus::Active),
            territory(2, c0, Some(p1), TerritoryStatus::Active),
        ];
        let continents = vec![continent(c0, 5)];

        let generated = calculate_generation(p0, &territories, &continents);
        assert_eq!(generated, 1);
    }

    #[test]
    fn multiple_fully_controlled_continents_sum_their_bonuses() {
        let p0 = player(1);
        let c0 = continent_id(1);
        let c1 = continent_id(2);
        let territories = vec![
            territory(1, c0, Some(p0), TerritoryStatus::Active),
            territory(2, c1, Some(p0), TerritoryStatus::Active),
            territory(3, c1, Some(p0), TerritoryStatus::Active),
        ];
        let continents = vec![continent(c0, 2), continent(c1, 4)];

        let generated = calculate_generation(p0, &territories, &continents);
        assert_eq!(generated, 3 + 2 + 4);
    }

    #[test]
    fn territories_outside_any_continent_data_still_count_toward_base() {
        let p0 = player(1);
        let c0 = continent_id(1);
        let territories = vec![territory(1, c0, Some(p0), TerritoryStatus::Active)];
        // No matching Continent entry at all — base count still applies,
        // just no bonus is possible.
        let generated = calculate_generation(p0, &territories, &[]);
        assert_eq!(generated, 1);
    }

    #[test]
    fn unit_pool_new_sets_remaining_to_generated_amount() {
        let pool = UnitPool::new(7);
        assert_eq!(pool.remaining(), 7);
    }

    #[test]
    fn unit_pool_consume_within_balance_deducts() {
        let mut pool = UnitPool::new(10);
        assert_eq!(pool.consume(4), Ok(()));
        assert_eq!(pool.remaining(), 6);
    }

    #[test]
    fn unit_pool_consume_exact_remaining_balance_succeeds_and_zeroes_out() {
        let mut pool = UnitPool::new(5);
        assert_eq!(pool.consume(5), Ok(()));
        assert_eq!(pool.remaining(), 0);
    }

    #[test]
    fn unit_pool_consume_exceeding_balance_is_rejected_and_leaves_pool_unchanged() {
        let mut pool = UnitPool::new(3);
        let result = pool.consume(4);
        assert_eq!(
            result,
            Err(InsufficientUnitsError {
                requested: 4,
                remaining: 3,
            })
        );
        assert_eq!(pool.remaining(), 3);
    }

    #[test]
    fn unit_pool_sequential_partial_consumptions_track_correctly() {
        let mut pool = UnitPool::new(10);
        assert_eq!(pool.consume(3), Ok(()));
        assert_eq!(pool.remaining(), 7);
        assert_eq!(pool.consume(2), Ok(()));
        assert_eq!(pool.remaining(), 5);
        assert!(pool.consume(6).is_err());
        assert_eq!(pool.remaining(), 5, "failed consumption must not mutate");
        assert_eq!(pool.consume(5), Ok(()));
        assert_eq!(pool.remaining(), 0);
    }
}
