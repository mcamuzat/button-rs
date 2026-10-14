use crate::action::{AttackKind, DeclaredAttack};
use crate::die::Die;
use crate::hooks::capture_scoring;
use crate::ids::PlayerId;
use crate::skill::DieSkill;
use crate::state::{CaptureScoring, GameState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Holding {
    Kept,
    Captured(CaptureScoring),
}

/// R 5.1 + R 6.6 + R 6.7 + R 6.15 + R 7.1 + R 7.2: points of one die, in half-points.
pub(crate) fn die_points(die: &Die, holding: Holding) -> i32 {
    let mut null = false;
    let mut poison = false;
    let mut value = false;
    for skill in die.skills.iter() {
        match skill {
            DieSkill::Null => null = true,
            DieSkill::Poison => poison = true,
            DieSkill::Value => value = true,
            DieSkill::Berserk
            | DieSkill::Chance
            | DieSkill::Fire
            | DieSkill::Focus
            | DieSkill::Konstant
            | DieSkill::Queer
            | DieSkill::Reserve
            | DieSkill::Shadow
            | DieSkill::Speed
            | DieSkill::Stealth
            | DieSkill::Stinger
            | DieSkill::Mighty
            | DieSkill::Slow
            | DieSkill::Ornery
            | DieSkill::TimeAndSpace
            | DieSkill::Rage
            | DieSkill::Rush
            | DieSkill::Boom
            | DieSkill::Trip => {}
        }
    }
    // R 6.6 + R 7.1.
    if null {
        return 0;
    }
    let basis = match holding {
        Holding::Captured(CaptureScoring::Zero) => return 0,
        Holding::Captured(CaptureScoring::ByValue(v)) => i32::from(v),
        Holding::Kept | Holding::Captured(CaptureScoring::BySize) => {
            if value {
                i32::from(die.value)
            } else {
                i32::from(die.max())
            }
        }
    };
    let captured = matches!(holding, Holding::Captured(_));
    match (captured, poison) {
        // R 5.1: half the size kept, the full size captured.
        (false, false) => basis,
        (true, false) => 2 * basis,
        // R 6.7: minus the full size kept, minus half the size captured.
        (false, true) => -2 * basis,
        (true, true) => -basis,
    }
}

/// R 5.1 + R 6.6 + R 6.7 + R 6.15 + R 6.20: how many half-points capturing `target` with
/// `attackers` swings towards the capturer: the points it gains, plus the kept points
/// the target's owner loses.
pub fn capture_swing(attackers: &[&Die], target: &Die) -> i32 {
    let scoring = capture_scoring(attackers, target);
    // R 6.20: a Rage target leaves its owner a clone worth the same kept points.
    let kept_lost = if crate::hooks::leaves_a_clone(target) {
        0
    } else {
        die_points(target, Holding::Kept)
    };
    die_points(target, Holding::Captured(scoring)) + kept_lost
}

/// Half-points `attack` swings towards `player` if it succeeds, before any reroll: the
/// captured dice (R 5.1, R 6.20), or the Boom die leaving `player`'s kept dice (R 6.22).
/// A Trip counts as if it captured; callers weigh its chance.
pub fn attack_swing(state: &GameState, player: PlayerId, attack: &DeclaredAttack) -> i32 {
    let attackers: Vec<&Die> = attack
        .attackers
        .iter()
        .filter_map(|&id| state.player(player).die(id))
        .collect();
    match attack.kind {
        AttackKind::Boom => -attackers
            .iter()
            .map(|d| die_points(d, Holding::Kept))
            .sum::<i32>(),
        AttackKind::Power
        | AttackKind::Skill
        | AttackKind::Shadow
        | AttackKind::Speed
        | AttackKind::Trip
        | AttackKind::Berserk
        | AttackKind::Rush => attack
            .targets
            .iter()
            .filter_map(|&id| state.player(player.other()).die(id))
            .map(|target| capture_swing(&attackers, target))
            .sum(),
    }
}

/// R 5.1: a player's current round score, in half-points.
pub fn round_score(state: &GameState, player: PlayerId) -> i32 {
    let p = state.player(player);
    let kept: i32 = p.active.iter().map(|d| die_points(d, Holding::Kept)).sum();
    let captured: i32 = p
        .captured
        .iter()
        .map(|c| die_points(&c.die, Holding::Captured(c.scoring)))
        .sum();
    kept + captured
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::die::Sides;
    use crate::ids::DieId;
    use crate::skill::DieSkill;

    fn d(sides: u8, value: u8, skills: &[DieSkill]) -> Die {
        Die {
            id: DieId(0),
            skills: skills.iter().copied().collect(),
            sides: Sides::Single(sides),
            value,
            dizzy: false,
        }
    }

    #[test]
    fn kept_dice_score_half_and_captured_dice_score_full() {
        assert_eq!(die_points(&d(7, 3, &[]), Holding::Kept), 7); // 3.5 points
        assert_eq!(
            die_points(&d(7, 3, &[]), Holding::Captured(CaptureScoring::BySize)),
            14
        );
    }

    #[test]
    fn poison_is_negative() {
        let p = d(6, 2, &[DieSkill::Poison]);
        assert_eq!(die_points(&p, Holding::Kept), -12);
        assert_eq!(
            die_points(&p, Holding::Captured(CaptureScoring::BySize)),
            -6
        );
    }

    #[test]
    fn null_is_worth_nothing() {
        let n = d(20, 2, &[DieSkill::Null, DieSkill::Poison]);
        assert_eq!(die_points(&n, Holding::Kept), 0);
        assert_eq!(
            die_points(&d(20, 2, &[]), Holding::Captured(CaptureScoring::Zero)),
            0
        );
    }

    #[test]
    fn capture_swing_follows_the_scoring_of_both_sides() {
        let plain = d(10, 4, &[]);
        let null = d(6, 1, &[DieSkill::Null]);
        let value = d(6, 2, &[DieSkill::Value]);
        // R 5.1: the owner loses 5 kept, the capturer gains 10 (in half-points: 10 + 20).
        assert_eq!(capture_swing(&[&plain], &plain), 30);
        // R 6.7: capturing Poison swings the other way.
        assert_eq!(
            capture_swing(&[&plain], &d(10, 4, &[DieSkill::Poison])),
            -30
        );
        // R 6.6: a Null target is worth nothing to anyone.
        assert_eq!(capture_swing(&[&plain], &null), 0);
        // R 6.6: captured by a Null attack, the target only leaves its owner's dice.
        assert_eq!(capture_swing(&[&null, &plain], &plain), 10);
        // R 6.15: a Value target counts its face; a Value attack captures at the face shown.
        assert_eq!(capture_swing(&[&plain], &value), 6);
        assert_eq!(capture_swing(&[&value], &plain), 10 + 8);
    }

    #[test]
    fn value_dice_score_their_face() {
        let v = d(20, 3, &[DieSkill::Value]);
        assert_eq!(die_points(&v, Holding::Kept), 3);
        assert_eq!(die_points(&v, Holding::Captured(CaptureScoring::BySize)), 6);
        assert_eq!(
            die_points(
                &d(20, 5, &[]),
                Holding::Captured(CaptureScoring::ByValue(5))
            ),
            10
        );
        // R 7.2: Poison + Value.
        assert_eq!(
            die_points(
                &d(20, 3, &[DieSkill::Value, DieSkill::Poison]),
                Holding::Kept
            ),
            -6
        );
    }
}
