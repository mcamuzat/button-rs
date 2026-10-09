use std::cmp::Ordering;

use crate::event::GameEvent;
use crate::hooks;
use crate::ids::PlayerId;
use crate::state::{GameState, WaitingFor};

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
    start_attacks(state);
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
