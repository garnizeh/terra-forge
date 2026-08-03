//! The Compile → Execute → Optimize turn/phase state machine (RFC-001 UC3,
//! RFC-002 §5.1). This is the "internal Engine match-state representation"
//! task 1.2 deferred to this task — the minimal slice of `MatchInstance`
//! (RFC-001 §7) that pure phase-transition functions need: current
//! turn/phase, ordered player list, per-player released status, and the
//! per-Optimize-phase Fortify-used flag. It deliberately does **not** live
//! under `protocol/`: it never crosses the Engine boundary at MVP (that's
//! `state_blob`/persistence, a Phase 3 concern), so it carries no
//! `serde`/`ts-rs` derives.
//!
//! No wall-clock time anywhere in this module: `advance`/`skip_*` only ever
//! know "what phase is it," never "how long until the deadline" (RFC-002
//! §5.1's UC8 note — the Platform Server decides when a timeout occurred,
//! but calls into these same transitions to apply it).
//!
//! Turn order is round-robin in **lobby-join order**, i.e. the order
//! `MatchState::new` receives the player list in — the simplest of the
//! task's two candidate conventions, with no PRNG dependency.
//!
//! Setting a player's `released` flag is task 1.9's `release_to_neutral`
//! transition, not this module's; only a read query is exposed here.

use crate::protocol::PlayerId;

/// A match's current phase within one player's turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    Compile,
    Execute,
    Optimize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlayerSlot {
    id: PlayerId,
    released: bool,
}

/// The minimal in-memory turn/phase state a match needs, per this task's
/// acceptance criteria. Not a wire type — see the module doc comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchState {
    current_turn: u32,
    current_phase: Phase,
    players: Vec<PlayerSlot>,
    active_player_index: usize,
    fortify_used: bool,
}

impl MatchState {
    /// Starts at turn 1, `Compile`, the first player in `players` (lobby-join
    /// order) active, nobody released, Fortify unused.
    ///
    /// Panics if `players` is empty — a match needs at least one seat.
    pub fn new(players: Vec<PlayerId>) -> Self {
        assert!(
            !players.is_empty(),
            "MatchState requires at least one player"
        );
        Self {
            current_turn: 1,
            current_phase: Phase::Compile,
            players: players
                .into_iter()
                .map(|id| PlayerSlot {
                    id,
                    released: false,
                })
                .collect(),
            active_player_index: 0,
            fortify_used: false,
        }
    }

    pub fn current_turn(&self) -> u32 {
        self.current_turn
    }

    pub fn current_phase(&self) -> Phase {
        self.current_phase
    }

    pub fn active_player(&self) -> PlayerId {
        self.players[self.active_player_index].id
    }

    pub fn is_active_player_released(&self) -> bool {
        self.players[self.active_player_index].released
    }

    /// Whether the given player (not necessarily the active one) has been
    /// released — e.g. task 1.7's `Concede` legality check is
    /// participant-gated, not turn-gated, so it needs any actor's status.
    pub fn is_player_released(&self, player_id: PlayerId) -> bool {
        self.players
            .iter()
            .find(|slot| slot.id == player_id)
            .is_some_and(|slot| slot.released)
    }

    pub fn fortify_used(&self) -> bool {
        self.fortify_used
    }

    /// Records that the active player has submitted their one Fortify move
    /// for this Optimize phase. Called by whichever later task applies a
    /// validated `Fortify` action.
    pub fn mark_fortify_used(&mut self) {
        self.fortify_used = true;
    }

    /// Advances Compile → Execute → Optimize → the next player's Compile,
    /// round-robin in lobby-join order. `current_turn` increments only on
    /// wraparound back to the first player. The Fortify-used flag resets
    /// on entering Optimize (not on leaving it). Reads no clock.
    pub fn advance(&mut self) {
        match self.current_phase {
            Phase::Compile => {
                self.current_phase = Phase::Execute;
            }
            Phase::Execute => {
                self.current_phase = Phase::Optimize;
                self.fortify_used = false;
            }
            Phase::Optimize => {
                self.advance_to_next_player();
            }
        }
    }

    fn advance_to_next_player(&mut self) {
        let next_index = self.active_player_index + 1;
        if next_index >= self.players.len() {
            self.active_player_index = 0;
            self.current_turn += 1;
        } else {
            self.active_player_index = next_index;
        }
        self.current_phase = Phase::Compile;
    }

    /// Advances as if the active player took no action during Compile —
    /// callable directly by whatever later decides a timeout occurred
    /// (Phase 3); takes no deadline/time input.
    pub fn skip_compile(&mut self) {
        debug_assert_eq!(self.current_phase, Phase::Compile);
        self.advance();
    }

    /// Advances as if the active player took no action during Execute.
    pub fn skip_execute(&mut self) {
        debug_assert_eq!(self.current_phase, Phase::Execute);
        self.advance();
    }

    /// Advances as if the active player took no action during Optimize.
    pub fn skip_optimize(&mut self) {
        debug_assert_eq!(self.current_phase, Phase::Optimize);
        self.advance();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Ulid;

    fn player(n: u128) -> PlayerId {
        PlayerId(Ulid::from_u128(n))
    }

    #[test]
    fn normal_progression_through_all_three_phases_for_one_player() {
        let solo = player(1);
        let mut state = MatchState::new(vec![solo]);

        assert_eq!(state.current_phase(), Phase::Compile);
        assert_eq!(state.current_turn(), 1);
        assert_eq!(state.active_player(), solo);

        state.advance();
        assert_eq!(state.current_phase(), Phase::Execute);
        assert_eq!(state.current_turn(), 1);

        state.advance();
        assert_eq!(state.current_phase(), Phase::Optimize);
        assert_eq!(state.current_turn(), 1);

        state.advance();
        assert_eq!(state.current_phase(), Phase::Compile);
        assert_eq!(state.current_turn(), 2);
        assert_eq!(state.active_player(), solo);
    }

    #[test]
    fn wraparound_from_last_players_optimize_back_to_first_players_compile() {
        let p0 = player(1);
        let p1 = player(2);
        let p2 = player(3);
        let mut state = MatchState::new(vec![p0, p1, p2]);

        // P0: Compile -> Execute -> Optimize -> next player's Compile.
        state.advance();
        state.advance();
        assert_eq!(state.active_player(), p0);
        state.advance();
        assert_eq!(state.active_player(), p1);
        assert_eq!(state.current_phase(), Phase::Compile);
        assert_eq!(state.current_turn(), 1, "no wraparound yet");

        // P1: Compile -> Execute -> Optimize -> next player's Compile.
        state.advance();
        state.advance();
        state.advance();
        assert_eq!(state.active_player(), p2);
        assert_eq!(state.current_phase(), Phase::Compile);
        assert_eq!(state.current_turn(), 1, "still no wraparound");

        // P2 (last player): Compile -> Execute -> Optimize -> wraparound to P0.
        state.advance();
        state.advance();
        state.advance();
        assert_eq!(state.active_player(), p0);
        assert_eq!(state.current_phase(), Phase::Compile);
        assert_eq!(state.current_turn(), 2, "turn increments on wraparound");
    }

    #[test]
    fn skip_phase_advances_exactly_like_advance() {
        let solo = player(1);

        let mut skipped = MatchState::new(vec![solo]);
        skipped.skip_compile();
        skipped.skip_execute();
        skipped.skip_optimize();

        let mut advanced = MatchState::new(vec![solo]);
        advanced.advance();
        advanced.advance();
        advanced.advance();

        assert_eq!(skipped, advanced);
    }

    #[test]
    fn fortify_used_flag_set_and_reset_across_phase_boundaries() {
        let p0 = player(1);
        let p1 = player(2);
        let mut state = MatchState::new(vec![p0, p1]);

        // Reach P0's Optimize phase.
        state.advance();
        state.advance();
        assert_eq!(state.current_phase(), Phase::Optimize);
        assert!(!state.fortify_used(), "unused entering Optimize");

        state.mark_fortify_used();
        assert!(state.fortify_used());

        // Move to P1's Compile, then P1's Execute -> Optimize: the flag
        // must reset on (re-)entering Optimize, for the new active player.
        state.advance();
        assert_eq!(state.active_player(), p1);
        assert_eq!(state.current_phase(), Phase::Compile);

        state.advance();
        assert_eq!(state.current_phase(), Phase::Execute);

        state.advance();
        assert_eq!(state.current_phase(), Phase::Optimize);
        assert!(
            !state.fortify_used(),
            "reset entering the new Optimize phase"
        );
    }

    #[test]
    fn is_player_released_defaults_to_false_for_everyone() {
        let p0 = player(1);
        let p1 = player(2);
        let state = MatchState::new(vec![p0, p1]);

        assert!(!state.is_active_player_released());
        assert!(!state.is_player_released(p0));
        assert!(!state.is_player_released(p1));
    }
}
