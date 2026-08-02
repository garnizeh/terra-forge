# 1.9 — Win/loss condition resolution

## Goal

Implement Victory, Elimination, Concession, and the Forfeit-trigger's shared territory-release transition, per RFC-001 §6 and UC8/UC9 — the Engine-level mechanics that end a player's participation or the match itself.

## Context

- RFC-001 §6: "**Victory:** A player achieves total planetary dominance by controlling 100% of the map's territories (fully compiled, not merely captured). Note that neutral territories left behind by a concession or forfeit still have to be conquered." "**Player exit:** ...Elimination..., Concession..., or Forfeit... All three release the player's territories to neutral and leave them connected in a read-only spectator capacity for the remainder of the match; they differ only in what `MatchHistory.end_reason` records." "**Match closure:** ...once only one active player remains by any combination of the exits above, `MatchInstance.status` transitions to `Closed`..."
- RFC-001 UC9 (Concede): "The conceding player's territories become **neutral**, retaining their existing unit garrisons as an unowned defending force — they are *not* redistributed to remaining players... The conceding player's own status transitions the same way as Elimination: read-only spectator for the remainder of the match."
- RFC-001 UC8 point 3: repeated timeouts escalate to forfeit, which "resolves the board identically to a voluntary concession... but is recorded distinctly: it is emitted as a `SystemEvent` rather than a player-submitted `GameAction`." The board-state mechanics must be the *same* Engine function for both — only the Phase 3/EventLog-level record of *why* differs.

## Acceptance criteria

- [ ] A single `release_to_neutral(player_id)` transition is shared by both the `Concede` `GameAction` path and whatever Phase 3 will later call directly for a timeout-driven forfeit, per UC8's explicit requirement that the two "resolve the board identically." This task must not implement two separate versions of territory release.
- [ ] `release_to_neutral`: every territory owned by the player becomes ownerless (`owner_id = None`); **retains its existing `unit_count`** as an unowned defending garrison (RFC-001 UC9 — explicitly not redistributed to remaining players); the player transitions to a read-only/eliminated status for the remainder of the match — task 1.7's validation must reject any subsequent action from a released player. Territories that are `Compiling` become neutral and remain `Compiling` with `faction = None`, but they never auto-complete via the wait-path (only attackable completion path remains available; the owner-dependent automatic completion-at-next-Compile no longer applies).
- [ ] `Concede` (pre-validated as legal by task 1.7) invokes `release_to_neutral` for the submitting actor.
- [ ] Elimination is detected automatically when a player's last territory is captured via combat (tasks 1.5/1.8's capture path) — read as: a player controlling zero territories (`Active` or `Compiling`) is eliminated, and the same `release_to_neutral` transition applies (in this case a no-op release, since they hold nothing, but the status transition to read-only/eliminated still applies).
- [ ] Victory is detected when a player controls 100% of the map's territories **and** every one of them is `Active` (fully compiled) — a territory that is `Compiling`, or a neutral (unowned) territory left over from a concession/forfeit, blocks Victory even if no other player holds it. This is the "fully compiled, not merely captured" requirement from RFC-001 §6.
- [ ] Match closure: once Victory is detected, or once only one active (non-released) player remains through any combination of exits, the match transitions to a closed state that task 1.4's phase machine stops advancing past.
- [ ] Unit tests: concession releasing territories with garrisons intact and unredistributed; elimination via combat capturing a player's last territory; Victory correctly **not** triggering when a player holds 100% of territories but one is still `Compiling`; match closure once only one player remains after two of three are released.

## Open questions

Not blocking, but flagged for a second look during implementation: whether a player whose only remaining territory is captured but goes `Compiling` (rather than instantly `Active` under the new owner) should be treated as eliminated the instant they hit zero owned territories, or only once no path remains for them to ever hold ground again. This document assumes the former (zero territories, `Active` or `Compiling`, triggers elimination) since RFC-001 doesn't describe any "grace" state — flag if that reading is wrong.

## Out of scope

`TimedOut`/`PlayerForfeited` `SystemEvent` emission and the timer/deadline logic that decides *when* to call this task's forfeit path (Phase 3). `MatchHistory.end_reason` recording (`Dominance`/`Eliminated`/`Conceded`/`TimedOut`) — a platform/DB-layer concern, Phase 3+.

## Depends on

1.2, 1.4, 1.7 (for `Concede` legality), 1.8 (a released player's `Compiling` territories need consistent handling).

## Status

Not started
**PR:** (none yet)
