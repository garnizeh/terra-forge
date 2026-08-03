//! Adjacency and phase-legality validation for every `GameAction` variant
//! (RFC-002 §5.1, RFC-001 §7) — the rules a move must satisfy to be legal at
//! all, independent of *who* is allowed to submit it. Session/actor-identity
//! authorization (is this really that player's session) is Phase 3's job,
//! layered on top of these pure checks; channel admission never authorizes
//! an action on its own.
//!
//! Two open questions blocked this task and are resolved here as tuning
//! values, not architectural ones (mirrors `combat.rs`'s
//! `DEFAULT_DEFENSE_BONUS`/ADR-0001 treatment — no ADR needed, free to
//! retune after Phase 2 CLI playtesting):
//! - [`ACCELERATE_COMPILE_COST`]: RFC-001 UC3 names a "fixed number" of
//!   units without specifying it.
//! - [`MINIMUM_GARRISON`]: neither RFC states whether `Attack`/`Fortify` may
//!   empty a source territory to 0 units; this crate assumes not.

use std::collections::{HashSet, VecDeque};

use crate::match_state::{MatchState, Phase};
use crate::protocol::{ActionType, GameAction, PlayerId, Territory, TerritoryId, TerritoryStatus};
use crate::unit_generation::UnitPool;

/// `AccelerateCompile`'s fixed unit cost (resolved open question, task 1.7).
pub const ACCELERATE_COMPILE_COST: u32 = 3;

/// Minimum units that must remain behind at an `Attack`/`Fortify` source
/// territory (resolved open question, task 1.7) — a source may commit at
/// most `unit_count - MINIMUM_GARRISON`.
pub const MINIMUM_GARRISON: u32 = 1;

/// Why a `GameAction` was rejected as illegal. Every variant is a specific,
/// distinguishable reason rather than a bare boolean, so a server can relay
/// *why* an action failed and a client can pre-validate with the same
/// specificity for optimistic UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegalityError {
    NotActivePlayer,
    WrongPhase,
    MissingTarget,
    UnknownTerritory,
    NotOwner,
    TerritoryNotActive,
    TerritoryNotCompiling,
    NotAdjacent,
    TargetOwnedByActor,
    NoContiguousPath,
    FortifyAlreadyUsed,
    ZeroUnitCount,
    ExceedsGarrison { requested: u32, max_allowed: u32 },
    InsufficientPool { requested: u32, available: u32 },
    IncorrectAccelerateCost { required: u32 },
    NotAParticipant,
    AlreadyReleased,
}

fn find_territory(territories: &[Territory], id: TerritoryId) -> Option<&Territory> {
    territories.iter().find(|t| t.id == id)
}

/// `Deploy` is legal only during the actor's own Compile phase, targeting an
/// `Active` territory the actor owns, for no more units than remain in that
/// Compile phase's generated pool (task 1.6).
pub fn validate_deploy(
    action: &GameAction,
    match_state: &MatchState,
    territories: &[Territory],
    pool: &UnitPool,
) -> Result<(), LegalityError> {
    debug_assert_eq!(action.action_type, ActionType::Deploy);

    if match_state.active_player() != action.actor_id {
        return Err(LegalityError::NotActivePlayer);
    }
    if match_state.current_phase() != Phase::Compile {
        return Err(LegalityError::WrongPhase);
    }

    let target_id = action
        .target_territory_id
        .ok_or(LegalityError::MissingTarget)?;
    let target = find_territory(territories, target_id).ok_or(LegalityError::UnknownTerritory)?;

    if target.owner_id != Some(action.actor_id) {
        return Err(LegalityError::NotOwner);
    }
    if target.status != TerritoryStatus::Active {
        return Err(LegalityError::TerritoryNotActive);
    }
    if action.unit_count == 0 {
        return Err(LegalityError::ZeroUnitCount);
    }
    if action.unit_count > pool.remaining() {
        return Err(LegalityError::InsufficientPool {
            requested: action.unit_count,
            available: pool.remaining(),
        });
    }

    Ok(())
}

/// `Attack` is legal only during the actor's own Execute phase, from an
/// `Active` territory the actor owns, against an adjacent territory the
/// actor does not own (neutral or enemy — RFC-001: "surrendered ground
/// still has to be taken by force"), committing at most
/// `source.unit_count - MINIMUM_GARRISON` units. A `Compiling` territory can
/// never be a source (RFC-001 UC3), but a `Compiling` territory can still be
/// attacked as a target — its zero-defense handling is a combat/task 1.8
/// concern, not a legality one.
pub fn validate_attack(
    action: &GameAction,
    match_state: &MatchState,
    territories: &[Territory],
) -> Result<(), LegalityError> {
    debug_assert_eq!(action.action_type, ActionType::Attack);

    if match_state.active_player() != action.actor_id {
        return Err(LegalityError::NotActivePlayer);
    }
    if match_state.current_phase() != Phase::Execute {
        return Err(LegalityError::WrongPhase);
    }

    let source = find_territory(territories, action.source_territory_id)
        .ok_or(LegalityError::UnknownTerritory)?;
    if source.owner_id != Some(action.actor_id) {
        return Err(LegalityError::NotOwner);
    }
    if source.status != TerritoryStatus::Active {
        return Err(LegalityError::TerritoryNotActive);
    }

    let target_id = action
        .target_territory_id
        .ok_or(LegalityError::MissingTarget)?;
    let target = find_territory(territories, target_id).ok_or(LegalityError::UnknownTerritory)?;

    if !source.adjacent_territory_ids.contains(&target_id) {
        return Err(LegalityError::NotAdjacent);
    }
    if target.owner_id == Some(action.actor_id) {
        return Err(LegalityError::TargetOwnedByActor);
    }
    if action.unit_count == 0 {
        return Err(LegalityError::ZeroUnitCount);
    }

    let max_allowed = source.unit_count.saturating_sub(MINIMUM_GARRISON);
    if action.unit_count > max_allowed {
        return Err(LegalityError::ExceedsGarrison {
            requested: action.unit_count,
            max_allowed,
        });
    }

    Ok(())
}

/// `Fortify` is legal only during the actor's own Optimize phase, at most
/// once per Optimize phase, between two `Active` territories the actor
/// owns, connected via a path of the actor's own contiguous `Active`
/// territories — not merely direct adjacency (RFC-001 UC3) — committing at
/// most `source.unit_count - MINIMUM_GARRISON` units.
pub fn validate_fortify(
    action: &GameAction,
    match_state: &MatchState,
    territories: &[Territory],
) -> Result<(), LegalityError> {
    debug_assert_eq!(action.action_type, ActionType::Fortify);

    if match_state.active_player() != action.actor_id {
        return Err(LegalityError::NotActivePlayer);
    }
    if match_state.current_phase() != Phase::Optimize {
        return Err(LegalityError::WrongPhase);
    }
    if match_state.fortify_used() {
        return Err(LegalityError::FortifyAlreadyUsed);
    }

    let source = find_territory(territories, action.source_territory_id)
        .ok_or(LegalityError::UnknownTerritory)?;
    if source.owner_id != Some(action.actor_id) {
        return Err(LegalityError::NotOwner);
    }
    if source.status != TerritoryStatus::Active {
        return Err(LegalityError::TerritoryNotActive);
    }

    let target_id = action
        .target_territory_id
        .ok_or(LegalityError::MissingTarget)?;
    let target = find_territory(territories, target_id).ok_or(LegalityError::UnknownTerritory)?;
    if target.owner_id != Some(action.actor_id) {
        return Err(LegalityError::NotOwner);
    }
    if target.status != TerritoryStatus::Active {
        return Err(LegalityError::TerritoryNotActive);
    }

    if !territories_connected(
        territories,
        action.actor_id,
        action.source_territory_id,
        target_id,
    ) {
        return Err(LegalityError::NoContiguousPath);
    }

    if action.unit_count == 0 {
        return Err(LegalityError::ZeroUnitCount);
    }

    let max_allowed = source.unit_count.saturating_sub(MINIMUM_GARRISON);
    if action.unit_count > max_allowed {
        return Err(LegalityError::ExceedsGarrison {
            requested: action.unit_count,
            max_allowed,
        });
    }

    Ok(())
}

/// Whether `target` is reachable from `source` through a path of
/// `owner`-owned, `Active` territories (RFC-001 UC3's "connected,
/// contiguous territories they control" — not mere direct adjacency).
fn territories_connected(
    territories: &[Territory],
    owner: PlayerId,
    source: TerritoryId,
    target: TerritoryId,
) -> bool {
    if source == target {
        return true;
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(source);
    queue.push_back(source);

    while let Some(current) = queue.pop_front() {
        let Some(current_territory) = find_territory(territories, current) else {
            continue;
        };
        for &adjacent in &current_territory.adjacent_territory_ids {
            if adjacent == target {
                return true;
            }
            if visited.contains(&adjacent) {
                continue;
            }
            let Some(adjacent_territory) = find_territory(territories, adjacent) else {
                continue;
            };
            if adjacent_territory.owner_id == Some(owner)
                && adjacent_territory.status == TerritoryStatus::Active
            {
                visited.insert(adjacent);
                queue.push_back(adjacent);
            }
        }
    }

    false
}

/// `Concede` is legal in any phase, from any active participant, exactly
/// once per match per actor — an already-conceded or already-eliminated
/// actor (both surface as `released`, per `match_state.rs`) cannot submit it
/// again, and a non-participant never could.
pub fn validate_concede(
    action: &GameAction,
    match_state: &MatchState,
) -> Result<(), LegalityError> {
    debug_assert_eq!(action.action_type, ActionType::Concede);

    if !match_state.is_participant(action.actor_id) {
        return Err(LegalityError::NotAParticipant);
    }
    if match_state.is_player_released(action.actor_id) {
        return Err(LegalityError::AlreadyReleased);
    }

    Ok(())
}

/// `AccelerateCompile` is legal only during the actor's own Compile phase,
/// targeting a `Compiling` territory the actor owns, for exactly
/// [`ACCELERATE_COMPILE_COST`] units drawn from that Compile phase's
/// freshly-generated pool (task 1.6) — never from any territory's existing
/// `unit_count`.
pub fn validate_accelerate_compile(
    action: &GameAction,
    match_state: &MatchState,
    territories: &[Territory],
    pool: &UnitPool,
) -> Result<(), LegalityError> {
    debug_assert_eq!(action.action_type, ActionType::AccelerateCompile);

    if match_state.active_player() != action.actor_id {
        return Err(LegalityError::NotActivePlayer);
    }
    if match_state.current_phase() != Phase::Compile {
        return Err(LegalityError::WrongPhase);
    }

    let target_id = action
        .target_territory_id
        .ok_or(LegalityError::MissingTarget)?;
    let target = find_territory(territories, target_id).ok_or(LegalityError::UnknownTerritory)?;

    if target.owner_id != Some(action.actor_id) {
        return Err(LegalityError::NotOwner);
    }
    if target.status != TerritoryStatus::Compiling {
        return Err(LegalityError::TerritoryNotCompiling);
    }
    if action.unit_count != ACCELERATE_COMPILE_COST {
        return Err(LegalityError::IncorrectAccelerateCost {
            required: ACCELERATE_COMPILE_COST,
        });
    }
    if pool.remaining() < ACCELERATE_COMPILE_COST {
        return Err(LegalityError::InsufficientPool {
            requested: ACCELERATE_COMPILE_COST,
            available: pool.remaining(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{ContinentId, Ulid};

    fn player(n: u128) -> PlayerId {
        PlayerId(Ulid::from_u128(n))
    }

    fn territory_id(n: u128) -> TerritoryId {
        TerritoryId(Ulid::from_u128(n))
    }

    fn territory(
        id: TerritoryId,
        owner: Option<PlayerId>,
        status: TerritoryStatus,
        unit_count: u32,
        adjacent: Vec<TerritoryId>,
    ) -> Territory {
        Territory {
            id,
            continent_id: ContinentId(Ulid::from_u128(1000)),
            owner_id: owner,
            faction: None,
            unit_count,
            status,
            adjacent_territory_ids: adjacent,
        }
    }

    fn deploy_action(actor: PlayerId, target: TerritoryId, unit_count: u32) -> GameAction {
        GameAction {
            action_type: ActionType::Deploy,
            actor_id: actor,
            source_territory_id: territory_id(0),
            target_territory_id: Some(target),
            unit_count,
        }
    }

    fn attack_action(
        actor: PlayerId,
        source: TerritoryId,
        target: TerritoryId,
        unit_count: u32,
    ) -> GameAction {
        GameAction {
            action_type: ActionType::Attack,
            actor_id: actor,
            source_territory_id: source,
            target_territory_id: Some(target),
            unit_count,
        }
    }

    fn fortify_action(
        actor: PlayerId,
        source: TerritoryId,
        target: TerritoryId,
        unit_count: u32,
    ) -> GameAction {
        GameAction {
            action_type: ActionType::Fortify,
            actor_id: actor,
            source_territory_id: source,
            target_territory_id: Some(target),
            unit_count,
        }
    }

    fn concede_action(actor: PlayerId) -> GameAction {
        GameAction {
            action_type: ActionType::Concede,
            actor_id: actor,
            source_territory_id: territory_id(0),
            target_territory_id: None,
            unit_count: 0,
        }
    }

    fn accelerate_action(actor: PlayerId, target: TerritoryId, unit_count: u32) -> GameAction {
        GameAction {
            action_type: ActionType::AccelerateCompile,
            actor_id: actor,
            source_territory_id: territory_id(0),
            target_territory_id: Some(target),
            unit_count,
        }
    }

    // ---- Deploy ----

    #[test]
    fn deploy_legal_case() {
        let p0 = player(1);
        let t0 = territory_id(1);
        let state = MatchState::new(vec![p0]);
        let territories = vec![territory(t0, Some(p0), TerritoryStatus::Active, 0, vec![])];
        let pool = UnitPool::new(5);

        assert_eq!(
            validate_deploy(&deploy_action(p0, t0, 3), &state, &territories, &pool),
            Ok(())
        );
    }

    #[test]
    fn deploy_wrong_phase() {
        let p0 = player(1);
        let t0 = territory_id(1);
        let mut state = MatchState::new(vec![p0]);
        state.advance(); // Execute
        let territories = vec![territory(t0, Some(p0), TerritoryStatus::Active, 0, vec![])];
        let pool = UnitPool::new(5);

        assert_eq!(
            validate_deploy(&deploy_action(p0, t0, 3), &state, &territories, &pool),
            Err(LegalityError::WrongPhase)
        );
    }

    #[test]
    fn deploy_not_active_player() {
        let p0 = player(1);
        let p1 = player(2);
        let t0 = territory_id(1);
        let state = MatchState::new(vec![p0, p1]);
        let territories = vec![territory(t0, Some(p1), TerritoryStatus::Active, 0, vec![])];
        let pool = UnitPool::new(5);

        assert_eq!(
            validate_deploy(&deploy_action(p1, t0, 3), &state, &territories, &pool),
            Err(LegalityError::NotActivePlayer)
        );
    }

    #[test]
    fn deploy_not_owner() {
        let p0 = player(1);
        let p1 = player(2);
        let t0 = territory_id(1);
        let state = MatchState::new(vec![p0]);
        let territories = vec![territory(t0, Some(p1), TerritoryStatus::Active, 0, vec![])];
        let pool = UnitPool::new(5);

        assert_eq!(
            validate_deploy(&deploy_action(p0, t0, 3), &state, &territories, &pool),
            Err(LegalityError::NotOwner)
        );
    }

    #[test]
    fn deploy_target_compiling() {
        let p0 = player(1);
        let t0 = territory_id(1);
        let state = MatchState::new(vec![p0]);
        let territories = vec![territory(
            t0,
            Some(p0),
            TerritoryStatus::Compiling,
            0,
            vec![],
        )];
        let pool = UnitPool::new(5);

        assert_eq!(
            validate_deploy(&deploy_action(p0, t0, 3), &state, &territories, &pool),
            Err(LegalityError::TerritoryNotActive)
        );
    }

    #[test]
    fn deploy_insufficient_pool() {
        let p0 = player(1);
        let t0 = territory_id(1);
        let state = MatchState::new(vec![p0]);
        let territories = vec![territory(t0, Some(p0), TerritoryStatus::Active, 0, vec![])];
        let pool = UnitPool::new(2);

        assert_eq!(
            validate_deploy(&deploy_action(p0, t0, 3), &state, &territories, &pool),
            Err(LegalityError::InsufficientPool {
                requested: 3,
                available: 2,
            })
        );
    }

    #[test]
    fn deploy_zero_unit_count() {
        let p0 = player(1);
        let t0 = territory_id(1);
        let state = MatchState::new(vec![p0]);
        let territories = vec![territory(t0, Some(p0), TerritoryStatus::Active, 0, vec![])];
        let pool = UnitPool::new(5);

        assert_eq!(
            validate_deploy(&deploy_action(p0, t0, 0), &state, &territories, &pool),
            Err(LegalityError::ZeroUnitCount)
        );
    }

    #[test]
    fn deploy_missing_target() {
        let p0 = player(1);
        let state = MatchState::new(vec![p0]);
        let pool = UnitPool::new(5);
        let action = GameAction {
            action_type: ActionType::Deploy,
            actor_id: p0,
            source_territory_id: territory_id(0),
            target_territory_id: None,
            unit_count: 3,
        };

        assert_eq!(
            validate_deploy(&action, &state, &[], &pool),
            Err(LegalityError::MissingTarget)
        );
    }

    #[test]
    fn deploy_unknown_territory() {
        let p0 = player(1);
        let t0 = territory_id(1);
        let state = MatchState::new(vec![p0]);
        let pool = UnitPool::new(5);

        assert_eq!(
            validate_deploy(&deploy_action(p0, t0, 3), &state, &[], &pool),
            Err(LegalityError::UnknownTerritory)
        );
    }

    // ---- Attack ----

    #[test]
    fn attack_legal_case() {
        let p0 = player(1);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let mut state = MatchState::new(vec![p0]);
        state.advance(); // Execute
        let territories = vec![
            territory(
                source_id,
                Some(p0),
                TerritoryStatus::Active,
                5,
                vec![target_id],
            ),
            territory(target_id, None, TerritoryStatus::Active, 2, vec![]),
        ];

        assert_eq!(
            validate_attack(
                &attack_action(p0, source_id, target_id, 4),
                &state,
                &territories
            ),
            Ok(())
        );
    }

    #[test]
    fn attack_wrong_phase() {
        let p0 = player(1);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let state = MatchState::new(vec![p0]); // still Compile
        let territories = vec![
            territory(
                source_id,
                Some(p0),
                TerritoryStatus::Active,
                5,
                vec![target_id],
            ),
            territory(target_id, None, TerritoryStatus::Active, 2, vec![]),
        ];

        assert_eq!(
            validate_attack(
                &attack_action(p0, source_id, target_id, 4),
                &state,
                &territories
            ),
            Err(LegalityError::WrongPhase)
        );
    }

    #[test]
    fn attack_not_active_player() {
        let p0 = player(1);
        let p1 = player(2);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let mut state = MatchState::new(vec![p0, p1]);
        state.advance(); // p0's Execute

        let territories = vec![
            territory(
                source_id,
                Some(p1),
                TerritoryStatus::Active,
                5,
                vec![target_id],
            ),
            territory(target_id, None, TerritoryStatus::Active, 2, vec![]),
        ];

        assert_eq!(
            validate_attack(
                &attack_action(p1, source_id, target_id, 4),
                &state,
                &territories
            ),
            Err(LegalityError::NotActivePlayer)
        );
    }

    #[test]
    fn attack_source_not_owner() {
        let p0 = player(1);
        let p1 = player(2);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let mut state = MatchState::new(vec![p0]);
        state.advance();
        let territories = vec![
            territory(
                source_id,
                Some(p1),
                TerritoryStatus::Active,
                5,
                vec![target_id],
            ),
            territory(target_id, None, TerritoryStatus::Active, 2, vec![]),
        ];

        assert_eq!(
            validate_attack(
                &attack_action(p0, source_id, target_id, 4),
                &state,
                &territories
            ),
            Err(LegalityError::NotOwner)
        );
    }

    #[test]
    fn attack_source_compiling() {
        let p0 = player(1);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let mut state = MatchState::new(vec![p0]);
        state.advance();
        let territories = vec![
            territory(
                source_id,
                Some(p0),
                TerritoryStatus::Compiling,
                5,
                vec![target_id],
            ),
            territory(target_id, None, TerritoryStatus::Active, 2, vec![]),
        ];

        assert_eq!(
            validate_attack(
                &attack_action(p0, source_id, target_id, 4),
                &state,
                &territories
            ),
            Err(LegalityError::TerritoryNotActive)
        );
    }

    #[test]
    fn attack_target_compiling_is_still_a_legal_target() {
        let p0 = player(1);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let mut state = MatchState::new(vec![p0]);
        state.advance();
        let territories = vec![
            territory(
                source_id,
                Some(p0),
                TerritoryStatus::Active,
                5,
                vec![target_id],
            ),
            territory(target_id, None, TerritoryStatus::Compiling, 2, vec![]),
        ];

        assert_eq!(
            validate_attack(
                &attack_action(p0, source_id, target_id, 4),
                &state,
                &territories
            ),
            Ok(())
        );
    }

    #[test]
    fn attack_not_adjacent() {
        let p0 = player(1);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let mut state = MatchState::new(vec![p0]);
        state.advance();
        let territories = vec![
            territory(source_id, Some(p0), TerritoryStatus::Active, 5, vec![]),
            territory(target_id, None, TerritoryStatus::Active, 2, vec![]),
        ];

        assert_eq!(
            validate_attack(
                &attack_action(p0, source_id, target_id, 4),
                &state,
                &territories
            ),
            Err(LegalityError::NotAdjacent)
        );
    }

    #[test]
    fn attack_target_owned_by_actor() {
        let p0 = player(1);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let mut state = MatchState::new(vec![p0]);
        state.advance();
        let territories = vec![
            territory(
                source_id,
                Some(p0),
                TerritoryStatus::Active,
                5,
                vec![target_id],
            ),
            territory(target_id, Some(p0), TerritoryStatus::Active, 2, vec![]),
        ];

        assert_eq!(
            validate_attack(
                &attack_action(p0, source_id, target_id, 4),
                &state,
                &territories
            ),
            Err(LegalityError::TargetOwnedByActor)
        );
    }

    #[test]
    fn attack_exceeds_garrison() {
        let p0 = player(1);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let mut state = MatchState::new(vec![p0]);
        state.advance();
        let territories = vec![
            territory(
                source_id,
                Some(p0),
                TerritoryStatus::Active,
                5,
                vec![target_id],
            ),
            territory(target_id, None, TerritoryStatus::Active, 2, vec![]),
        ];

        // source has 5 units; garrison=1 means max committable is 4.
        assert_eq!(
            validate_attack(
                &attack_action(p0, source_id, target_id, 5),
                &state,
                &territories
            ),
            Err(LegalityError::ExceedsGarrison {
                requested: 5,
                max_allowed: 4,
            })
        );
    }

    #[test]
    fn attack_zero_unit_count() {
        let p0 = player(1);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let mut state = MatchState::new(vec![p0]);
        state.advance();
        let territories = vec![
            territory(
                source_id,
                Some(p0),
                TerritoryStatus::Active,
                5,
                vec![target_id],
            ),
            territory(target_id, None, TerritoryStatus::Active, 2, vec![]),
        ];

        assert_eq!(
            validate_attack(
                &attack_action(p0, source_id, target_id, 0),
                &state,
                &territories
            ),
            Err(LegalityError::ZeroUnitCount)
        );
    }

    // ---- Fortify ----

    #[test]
    fn fortify_legal_direct_adjacency() {
        let p0 = player(1);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let mut state = MatchState::new(vec![p0]);
        state.advance();
        state.advance(); // Optimize
        let territories = vec![
            territory(
                source_id,
                Some(p0),
                TerritoryStatus::Active,
                5,
                vec![target_id],
            ),
            territory(
                target_id,
                Some(p0),
                TerritoryStatus::Active,
                1,
                vec![source_id],
            ),
        ];

        assert_eq!(
            validate_fortify(
                &fortify_action(p0, source_id, target_id, 3),
                &state,
                &territories
            ),
            Ok(())
        );
    }

    #[test]
    fn fortify_legal_multi_hop_contiguous_path() {
        let p0 = player(1);
        let a = territory_id(1);
        let b = territory_id(2);
        let c = territory_id(3);
        let mut state = MatchState::new(vec![p0]);
        state.advance();
        state.advance();
        // a - b - c, only directly adjacent pairs are (a,b) and (b,c); a and
        // c are not directly adjacent, only contiguously connected via b.
        let territories = vec![
            territory(a, Some(p0), TerritoryStatus::Active, 5, vec![b]),
            territory(b, Some(p0), TerritoryStatus::Active, 1, vec![a, c]),
            territory(c, Some(p0), TerritoryStatus::Active, 1, vec![b]),
        ];

        assert_eq!(
            validate_fortify(&fortify_action(p0, a, c, 2), &state, &territories),
            Ok(())
        );
    }

    #[test]
    fn fortify_no_contiguous_path_blocked_by_enemy_territory() {
        let p0 = player(1);
        let p1 = player(2);
        let a = territory_id(1);
        let b = territory_id(2);
        let c = territory_id(3);
        let mut state = MatchState::new(vec![p0]);
        state.advance();
        state.advance();
        // a - b(enemy) - c: b breaks the path since it isn't p0's Active territory.
        let territories = vec![
            territory(a, Some(p0), TerritoryStatus::Active, 5, vec![b]),
            territory(b, Some(p1), TerritoryStatus::Active, 1, vec![a, c]),
            territory(c, Some(p0), TerritoryStatus::Active, 1, vec![b]),
        ];

        assert_eq!(
            validate_fortify(&fortify_action(p0, a, c, 2), &state, &territories),
            Err(LegalityError::NoContiguousPath)
        );
    }

    #[test]
    fn fortify_wrong_phase() {
        let p0 = player(1);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let state = MatchState::new(vec![p0]); // Compile
        let territories = vec![
            territory(
                source_id,
                Some(p0),
                TerritoryStatus::Active,
                5,
                vec![target_id],
            ),
            territory(
                target_id,
                Some(p0),
                TerritoryStatus::Active,
                1,
                vec![source_id],
            ),
        ];

        assert_eq!(
            validate_fortify(
                &fortify_action(p0, source_id, target_id, 3),
                &state,
                &territories
            ),
            Err(LegalityError::WrongPhase)
        );
    }

    #[test]
    fn fortify_not_active_player() {
        let p0 = player(1);
        let p1 = player(2);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let mut state = MatchState::new(vec![p0, p1]);
        state.advance();
        state.advance(); // p0's Optimize
        let territories = vec![
            territory(
                source_id,
                Some(p1),
                TerritoryStatus::Active,
                5,
                vec![target_id],
            ),
            territory(
                target_id,
                Some(p1),
                TerritoryStatus::Active,
                1,
                vec![source_id],
            ),
        ];

        assert_eq!(
            validate_fortify(
                &fortify_action(p1, source_id, target_id, 3),
                &state,
                &territories
            ),
            Err(LegalityError::NotActivePlayer)
        );
    }

    #[test]
    fn fortify_already_used_this_phase() {
        let p0 = player(1);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let mut state = MatchState::new(vec![p0]);
        state.advance();
        state.advance();
        state.mark_fortify_used();
        let territories = vec![
            territory(
                source_id,
                Some(p0),
                TerritoryStatus::Active,
                5,
                vec![target_id],
            ),
            territory(
                target_id,
                Some(p0),
                TerritoryStatus::Active,
                1,
                vec![source_id],
            ),
        ];

        assert_eq!(
            validate_fortify(
                &fortify_action(p0, source_id, target_id, 3),
                &state,
                &territories
            ),
            Err(LegalityError::FortifyAlreadyUsed)
        );
    }

    #[test]
    fn fortify_target_not_owner() {
        let p0 = player(1);
        let p1 = player(2);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let mut state = MatchState::new(vec![p0]);
        state.advance();
        state.advance();
        let territories = vec![
            territory(
                source_id,
                Some(p0),
                TerritoryStatus::Active,
                5,
                vec![target_id],
            ),
            territory(
                target_id,
                Some(p1),
                TerritoryStatus::Active,
                1,
                vec![source_id],
            ),
        ];

        assert_eq!(
            validate_fortify(
                &fortify_action(p0, source_id, target_id, 3),
                &state,
                &territories
            ),
            Err(LegalityError::NotOwner)
        );
    }

    #[test]
    fn fortify_exceeds_garrison() {
        let p0 = player(1);
        let source_id = territory_id(1);
        let target_id = territory_id(2);
        let mut state = MatchState::new(vec![p0]);
        state.advance();
        state.advance();
        let territories = vec![
            territory(
                source_id,
                Some(p0),
                TerritoryStatus::Active,
                5,
                vec![target_id],
            ),
            territory(
                target_id,
                Some(p0),
                TerritoryStatus::Active,
                1,
                vec![source_id],
            ),
        ];

        assert_eq!(
            validate_fortify(
                &fortify_action(p0, source_id, target_id, 5),
                &state,
                &territories
            ),
            Err(LegalityError::ExceedsGarrison {
                requested: 5,
                max_allowed: 4,
            })
        );
    }

    // ---- Concede ----

    #[test]
    fn concede_legal_in_any_phase() {
        let p0 = player(1);
        let mut state = MatchState::new(vec![p0]);
        assert_eq!(validate_concede(&concede_action(p0), &state), Ok(()));

        state.advance(); // Execute
        assert_eq!(validate_concede(&concede_action(p0), &state), Ok(()));

        state.advance(); // Optimize
        assert_eq!(validate_concede(&concede_action(p0), &state), Ok(()));
    }

    #[test]
    fn concede_not_a_participant() {
        let p0 = player(1);
        let stranger = player(99);
        let state = MatchState::new(vec![p0]);

        assert_eq!(
            validate_concede(&concede_action(stranger), &state),
            Err(LegalityError::NotAParticipant)
        );
    }

    #[test]
    fn concede_already_released() {
        let p0 = player(1);
        let mut state = MatchState::new(vec![p0]);
        state.mark_released_for_test(p0);

        assert_eq!(
            validate_concede(&concede_action(p0), &state),
            Err(LegalityError::AlreadyReleased)
        );
    }

    // ---- AccelerateCompile ----

    #[test]
    fn accelerate_compile_legal_case() {
        let p0 = player(1);
        let t0 = territory_id(1);
        let state = MatchState::new(vec![p0]);
        let territories = vec![territory(
            t0,
            Some(p0),
            TerritoryStatus::Compiling,
            0,
            vec![],
        )];
        let pool = UnitPool::new(5);

        assert_eq!(
            validate_accelerate_compile(
                &accelerate_action(p0, t0, ACCELERATE_COMPILE_COST),
                &state,
                &territories,
                &pool
            ),
            Ok(())
        );
    }

    #[test]
    fn accelerate_compile_wrong_phase() {
        let p0 = player(1);
        let t0 = territory_id(1);
        let mut state = MatchState::new(vec![p0]);
        state.advance(); // Execute
        let territories = vec![territory(
            t0,
            Some(p0),
            TerritoryStatus::Compiling,
            0,
            vec![],
        )];
        let pool = UnitPool::new(5);

        assert_eq!(
            validate_accelerate_compile(
                &accelerate_action(p0, t0, ACCELERATE_COMPILE_COST),
                &state,
                &territories,
                &pool
            ),
            Err(LegalityError::WrongPhase)
        );
    }

    #[test]
    fn accelerate_compile_target_not_compiling() {
        let p0 = player(1);
        let t0 = territory_id(1);
        let state = MatchState::new(vec![p0]);
        let territories = vec![territory(t0, Some(p0), TerritoryStatus::Active, 0, vec![])];
        let pool = UnitPool::new(5);

        assert_eq!(
            validate_accelerate_compile(
                &accelerate_action(p0, t0, ACCELERATE_COMPILE_COST),
                &state,
                &territories,
                &pool
            ),
            Err(LegalityError::TerritoryNotCompiling)
        );
    }

    #[test]
    fn accelerate_compile_not_owner() {
        let p0 = player(1);
        let p1 = player(2);
        let t0 = territory_id(1);
        let state = MatchState::new(vec![p0]);
        let territories = vec![territory(
            t0,
            Some(p1),
            TerritoryStatus::Compiling,
            0,
            vec![],
        )];
        let pool = UnitPool::new(5);

        assert_eq!(
            validate_accelerate_compile(
                &accelerate_action(p0, t0, ACCELERATE_COMPILE_COST),
                &state,
                &territories,
                &pool
            ),
            Err(LegalityError::NotOwner)
        );
    }

    #[test]
    fn accelerate_compile_incorrect_cost() {
        let p0 = player(1);
        let t0 = territory_id(1);
        let state = MatchState::new(vec![p0]);
        let territories = vec![territory(
            t0,
            Some(p0),
            TerritoryStatus::Compiling,
            0,
            vec![],
        )];
        let pool = UnitPool::new(5);

        assert_eq!(
            validate_accelerate_compile(&accelerate_action(p0, t0, 1), &state, &territories, &pool),
            Err(LegalityError::IncorrectAccelerateCost {
                required: ACCELERATE_COMPILE_COST,
            })
        );
    }

    #[test]
    fn accelerate_compile_insufficient_pool() {
        let p0 = player(1);
        let t0 = territory_id(1);
        let state = MatchState::new(vec![p0]);
        let territories = vec![territory(
            t0,
            Some(p0),
            TerritoryStatus::Compiling,
            0,
            vec![],
        )];
        let pool = UnitPool::new(1);

        assert_eq!(
            validate_accelerate_compile(
                &accelerate_action(p0, t0, ACCELERATE_COMPILE_COST),
                &state,
                &territories,
                &pool
            ),
            Err(LegalityError::InsufficientPool {
                requested: ACCELERATE_COMPILE_COST,
                available: 1,
            })
        );
    }

    #[test]
    fn accelerate_compile_missing_target() {
        let p0 = player(1);
        let state = MatchState::new(vec![p0]);
        let pool = UnitPool::new(5);
        let action = GameAction {
            action_type: ActionType::AccelerateCompile,
            actor_id: p0,
            source_territory_id: territory_id(0),
            target_territory_id: None,
            unit_count: ACCELERATE_COMPILE_COST,
        };

        assert_eq!(
            validate_accelerate_compile(&action, &state, &[], &pool),
            Err(LegalityError::MissingTarget)
        );
    }
}
