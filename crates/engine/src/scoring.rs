use crate::die::Die;
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
