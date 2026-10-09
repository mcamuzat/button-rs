use buttonmen_engine::{
    AttackKind, DeclaredAttack, Die, DieSkill, GameState, PlayerId, WaitingFor, check_attack,
    round_score,
};

const MATCH_WON: f64 = 1.0e9;
/// One round of lead is worth more than any score difference within a round.
const ROUND_WON: f64 = 1000.0;
/// Tuned in the arena (`hard_beats_easy`): 1.0 won 17 / 18, 0.5 won 12 / 18, 1.5 won 16 / 18.
const THREAT_WEIGHT: f64 = 1.0;

/// How good `state` is for `me` (spec §7): match lead first, then the current round's
/// score difference (half-points), then what each side threatens to capture next.
pub fn evaluate(state: &GameState, me: PlayerId) -> f64 {
    let opponent = me.other();
    if let WaitingFor::GameOver { result } = &state.waiting_for {
        return if result.winner == me {
            MATCH_WON
        } else {
            -MATCH_WON
        };
    }
    let wins = |p: PlayerId| f64::from(state.player(p).record.wins);
    let rounds = wins(me) - wins(opponent);
    let score = f64::from(round_score(state, me) - round_score(state, opponent));
    let threat = threat(state, me) - threat(state, opponent);
    rounds * ROUND_WON + score + THREAT_WEIGHT * threat
}

/// Half-points that capturing `target` swings towards the capturer (R 5.1): it leaves
/// its owner's kept dice (half its size) and joins the capturer's pile (its full size).
/// A Poison die swings the other way (R 6.7).
pub fn capture_gain(target: &Die) -> f64 {
    let size = f64::from(target.max());
    if target.has(DieSkill::Poison) {
        -3.0 * size
    } else {
        3.0 * size
    }
}

/// The best single-die capture `side` could make next (Power or Shadow): a cheap
/// stand-in for the full attack list.
fn threat(state: &GameState, side: PlayerId) -> f64 {
    let mut best = 0.0_f64;
    for attacker in &state.player(side).active {
        for target in &state.player(side.other()).active {
            for kind in [AttackKind::Power, AttackKind::Shadow] {
                let attack = DeclaredAttack {
                    kind,
                    attackers: vec![attacker.id],
                    targets: vec![target.id],
                };
                if check_attack(state, side, &attack).is_ok() {
                    best = best.max(capture_gain(target));
                }
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use PlayerId::{One, Two};
    use buttonmen_engine::testing::{attack_state, die};
    use buttonmen_engine::{EndReason, MatchResult, WaitingFor};

    #[test]
    fn a_won_match_dominates_everything() {
        let mut s = attack_state(vec![die(4, 1)], vec![die(20, 20)], One);
        s.waiting_for = WaitingFor::GameOver {
            result: MatchResult {
                winner: One,
                reason: EndReason::Rounds,
            },
        };
        assert!(evaluate(&s, One) > 1.0e8);
        assert!(evaluate(&s, Two) < -1.0e8);
    }

    #[test]
    fn a_round_won_outweighs_any_score() {
        let mut ahead = attack_state(vec![die(4, 1)], vec![die(20, 20), die(20, 20)], One);
        ahead.player_mut(One).record.wins = 1;
        let behind = attack_state(vec![die(20, 20), die(20, 20)], vec![die(4, 1)], One);
        assert!(evaluate(&ahead, One) > evaluate(&behind, One));
    }

    #[test]
    fn threats_count() {
        // One's d20 showing 20 threatens Two's d20; in the other position nothing is threatened.
        let threatening = attack_state(vec![die(20, 20)], vec![die(20, 19)], Two);
        let quiet = attack_state(vec![die(20, 1)], vec![die(20, 19)], Two);
        assert!(evaluate(&threatening, One) > evaluate(&quiet, One));
    }

    #[test]
    fn capturing_poison_is_a_loss() {
        let s = attack_state(
            vec![die(6, 6)],
            vec![
                die(10, 3).with(buttonmen_engine::DieSkill::Poison),
                die(10, 3),
            ],
            One,
        );
        let two = s.player(Two);
        assert!(capture_gain(&two.active[0]) < 0.0);
        assert_eq!(capture_gain(&two.active[1]), 30.0);
    }
}
