use std::cmp::Ordering;

use crate::error::{ReactionError, RuleError};
use crate::event::{GameEvent, ResizeCause};
use crate::hooks;
use crate::ids::DieId;
use crate::ids::PlayerId;
use crate::skill::DieSkill;
use crate::state::{GameState, WaitingFor};
use std::collections::BTreeSet;

/// R 3.3: sorted values of `player`'s dice that count for initiative.
pub(crate) fn initiative_values(state: &GameState, player: PlayerId) -> Vec<u8> {
    let mut values: Vec<u8> = state
        .player(player)
        .active
        .iter()
        .filter(|d| hooks::counts_for_initiative(d))
        .map(|d| d.value)
        .collect();
    values.sort_unstable();
    values
}

/// R 3.3: `Less` when `a` goes first. Lowest die first, then the next one; if one
/// list runs out while the other goes on, the longer list goes first (buttonweavers
/// pads each list with a sentinel lower than "no die").
pub(crate) fn compare(a: &[u8], b: &[u8]) -> Ordering {
    for (x, y) in a.iter().zip(b) {
        if x != y {
            return x.cmp(y);
        }
    }
    b.len().cmp(&a.len())
}

/// R 3.3 + R 3.4: decides who starts the round.
pub(crate) fn determine(state: &mut GameState, events: &mut Vec<GameEvent>) {
    let order = compare(
        &initiative_values(state, PlayerId::One),
        &initiative_values(state, PlayerId::Two),
    );
    let (winner, by_coin) = match order {
        Ordering::Less => (PlayerId::One, false),
        Ordering::Greater => (PlayerId::Two, false),
        // R 3.4: complete tie, coin toss.
        Ordering::Equal => (
            if state.rng.coin() {
                PlayerId::One
            } else {
                PlayerId::Two
            },
            true,
        ),
    };
    state.initiative = Some(winner);
    events.push(GameEvent::InitiativeDecided {
        player: winner,
        by_coin,
    });
    offer_reaction(state, winner.other());
}

/// R 6.4 / R 7.4: only the starting player's lowered Focus dice stay dizzy.
pub(crate) fn start_attacks(state: &mut GameState) {
    let first = state
        .initiative
        .expect("initiative is decided before attacks");
    for die in &mut state.player_mut(first.other()).active {
        die.dizzy = false;
    }
    state.waiting_for = WaitingFor::Attack { player: first };
}

fn can_react(state: &GameState, player: PlayerId) -> bool {
    state
        .player(player)
        .active
        .iter()
        .any(|d| d.has(DieSkill::Focus) || d.has(DieSkill::Chance))
}

/// R 3.5: the player without initiative may react if they have Focus or Chance dice.
pub(crate) fn offer_reaction(state: &mut GameState, player: PlayerId) {
    if can_react(state, player) {
        state.waiting_for = WaitingFor::ReactToInitiative { player };
    } else {
        start_attacks(state);
    }
}

fn expect_reacting(state: &GameState, player: PlayerId) -> Result<(), RuleError> {
    match state.waiting_for {
        WaitingFor::ReactToInitiative { player: p } if p == player => Ok(()),
        WaitingFor::ReactToInitiative { .. } => Err(RuleError::NotYourTurn),
        _ => Err(RuleError::WrongPhase),
    }
}

fn reaction(reason: ReactionError) -> RuleError {
    RuleError::InvalidReaction { reason }
}

/// R 6.4: lower one or more Focus dice, only to gain the initiative.
pub(crate) fn focus_turn_down(
    state: &mut GameState,
    player: PlayerId,
    dice: Vec<(DieId, u8)>,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    expect_reacting(state, player)?;
    if dice.is_empty() {
        return Err(reaction(ReactionError::NoDice));
    }
    let mut seen = BTreeSet::new();
    for &(id, value) in &dice {
        let die = state
            .player(player)
            .die(id)
            .ok_or(RuleError::UnknownDie { die: id })?;
        if !die.has(DieSkill::Focus) {
            return Err(reaction(ReactionError::NotFocus { die: id }));
        }
        if !seen.insert(id) || value >= die.value || value < die.min() {
            return Err(reaction(ReactionError::ValueNotLowered { die: id }));
        }
    }
    for &(id, value) in &dice {
        let die = state.player_mut(player).die_mut(id).expect("checked above");
        die.value = value;
        // R 6.4: unusable for the first attack if this player starts.
        die.dizzy = true;
    }
    let order = compare(
        &initiative_values(state, player),
        &initiative_values(state, player.other()),
    );
    if order != Ordering::Less {
        return Err(reaction(ReactionError::DoesNotGainInitiative));
    }
    events.push(GameEvent::FocusTurnedDown { player, dice });
    state.initiative = Some(player);
    offer_reaction(state, player.other());
    Ok(())
}

/// R 6.2: reroll one Chance die; a failed attempt ends the reactions.
pub(crate) fn chance_reroll(
    state: &mut GameState,
    player: PlayerId,
    id: DieId,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    expect_reacting(state, player)?;
    let GameState { players, rng, .. } = state;
    let die = players[player.index()]
        .die_mut(id)
        .ok_or(RuleError::UnknownDie { die: id })?;
    if !die.has(DieSkill::Chance) {
        return Err(reaction(ReactionError::NotChance { die: id }));
    }
    // R 6.17 + R 7.5: a Chance reroll makes a Mighty die grow.
    if let Some(from) = hooks::reroll(die, rng, hooks::RerollCause::Normal) {
        events.push(GameEvent::DieResized {
            die: id,
            from,
            to: die.sides,
            cause: ResizeCause::Mighty,
        });
    }
    let value = die.value;
    events.push(GameEvent::ChanceRerolled {
        player,
        die: id,
        value,
    });
    let order = compare(
        &initiative_values(state, player),
        &initiative_values(state, player.other()),
    );
    let (gains, by_coin) = match order {
        Ordering::Less => (true, false),
        // R 6.2: a tie is a failed attempt (BMGame::react_to_initiative_chance).
        Ordering::Greater | Ordering::Equal => (false, false),
    };
    if gains {
        state.initiative = Some(player);
        events.push(GameEvent::InitiativeDecided { player, by_coin });
        offer_reaction(state, player.other());
    } else {
        start_attacks(state);
    }
    Ok(())
}

/// R 3.5: let the opponent start.
pub(crate) fn pass_initiative(
    state: &mut GameState,
    player: PlayerId,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    expect_reacting(state, player)?;
    events.push(GameEvent::InitiativePassed { player });
    start_attacks(state);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowest_first_die_wins() {
        assert_eq!(compare(&[1, 9], &[2, 3]), Ordering::Less);
        assert_eq!(compare(&[3], &[2, 20]), Ordering::Greater);
    }

    #[test]
    fn ties_compare_the_next_die() {
        assert_eq!(compare(&[1, 4], &[1, 3]), Ordering::Greater);
        assert_eq!(compare(&[2, 2], &[2, 2]), Ordering::Equal);
    }

    #[test]
    fn longer_list_wins_an_equal_prefix() {
        assert_eq!(compare(&[1, 2], &[1, 2, 3]), Ordering::Greater);
        assert_eq!(compare(&[1, 2, 3], &[1, 2]), Ordering::Less);
        assert_eq!(compare(&[], &[5]), Ordering::Greater);
        assert_eq!(compare(&[], &[]), Ordering::Equal);
    }
}
