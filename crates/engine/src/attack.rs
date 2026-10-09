use std::collections::BTreeSet;

use crate::action::{AttackKind, DeclaredAttack};
use crate::die::Die;
use crate::error::AttackError;
use crate::hooks;
use crate::ids::DieId;
use crate::ids::PlayerId;
use crate::state::GameState;

struct Participants<'a> {
    attackers: Vec<&'a Die>,
    targets: Vec<&'a Die>,
}

fn gather<'a>(
    state: &'a GameState,
    player: PlayerId,
    attack: &DeclaredAttack,
) -> Result<Participants<'a>, AttackError> {
    let mut seen = BTreeSet::new();
    for &die in attack.attackers.iter().chain(&attack.targets) {
        if !seen.insert(die) {
            return Err(AttackError::DuplicateDie { die });
        }
    }
    let attackers = attack
        .attackers
        .iter()
        .map(|&die| {
            state
                .player(player)
                .die(die)
                .ok_or(AttackError::NotYourDie { die })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let targets = attack
        .targets
        .iter()
        .map(|&die| {
            state
                .player(player.other())
                .die(die)
                .ok_or(AttackError::NotOpponentDie { die })
        })
        .collect::<Result<Vec<_>, _>>()?;
    // R 6.4 / R 7.4.
    if let Some(d) = attackers.iter().find(|d| d.dizzy) {
        return Err(AttackError::Dizzy { die: d.id });
    }
    Ok(Participants { attackers, targets })
}

/// How many attackers and targets each kind takes (R 4.2, R 4.3, R 6.1, R 6.9, R 6.10, R 6.14).
fn check_counts(kind: AttackKind, p: &Participants) -> Result<(), AttackError> {
    let (a, t) = (p.attackers.len(), p.targets.len());
    let ok = match kind {
        AttackKind::Power | AttackKind::Shadow | AttackKind::Trip => a == 1 && t == 1,
        AttackKind::Skill => a >= 1 && t == 1,
        AttackKind::Speed | AttackKind::Berserk => a == 1 && t >= 1,
    };
    if ok {
        Ok(())
    } else {
        Err(AttackError::WrongDiceCount)
    }
}

fn check_skills(kind: AttackKind, p: &Participants) -> Result<(), AttackError> {
    let n = p.attackers.len();
    for a in &p.attackers {
        let unavailable = !hooks::attack_kinds(a).contains(&kind)
            || (kind == AttackKind::Skill && n < 2 && hooks::needs_partner(a));
        if unavailable {
            return Err(AttackError::KindUnavailable { die: a.id, kind });
        }
    }
    for t in &p.targets {
        if !hooks::can_be_captured(t, kind, n) {
            return Err(AttackError::TargetProtected { die: t.id });
        }
    }
    Ok(())
}

/// R 4.3 + R 6.5 + R 6.13: every total the attackers can make together.
pub(crate) fn skill_sums(attackers: &[&Die]) -> BTreeSet<i32> {
    let mut sums = BTreeSet::from([0]);
    for die in attackers {
        let values = hooks::skill_values(die);
        sums = sums
            .iter()
            .flat_map(|s| values.iter().map(move |v| s + v))
            .collect();
    }
    sums
}

fn values_allow(kind: AttackKind, p: &Participants) -> bool {
    let first_attacker = p.attackers[0];
    let first_target = p.targets[0];
    match kind {
        // R 4.2.
        AttackKind::Power => first_attacker.value >= first_target.value,
        // R 6.10.
        AttackKind::Shadow => {
            first_target.value >= first_attacker.value && first_target.value <= first_attacker.max()
        }
        // R 6.14: illegal only without any chance to capture.
        AttackKind::Trip => first_attacker.max() >= first_target.min(),
        // R 6.9 / R 6.1.
        AttackKind::Speed | AttackKind::Berserk => {
            p.targets.iter().map(|t| u32::from(t.value)).sum::<u32>()
                == u32::from(first_attacker.value)
        }
        // R 4.3.
        AttackKind::Skill => skill_sums(&p.attackers).contains(&i32::from(first_target.value)),
    }
}

/// Is `attack` legal for `player` right now (dice, counts, skills and values)?
pub fn check_attack(
    state: &GameState,
    player: PlayerId,
    attack: &DeclaredAttack,
) -> Result<(), AttackError> {
    let p = gather(state, player, attack)?;
    check_counts(attack.kind, &p)?;
    check_skills(attack.kind, &p)?;
    if values_allow(attack.kind, &p) {
        Ok(())
    } else {
        Err(AttackError::ValuesDoNotMatch)
    }
}

/// Non-empty subsets of `dice`, in a stable order (capped at 16 dice: no button comes close).
fn subsets(dice: &[&Die]) -> Vec<Vec<DieId>> {
    let n = dice.len().min(16);
    (1u32..(1u32 << n))
        .map(|mask| {
            (0..n)
                .filter(|i| mask & (1 << i) != 0)
                .map(|i| dice[i].id)
                .collect()
        })
        .collect()
}

/// Every attack worth checking for `player` right now (not filtered for legality).
pub(crate) fn attack_shapes(state: &GameState, player: PlayerId) -> Vec<DeclaredAttack> {
    let mine: Vec<&Die> = state
        .player(player)
        .active
        .iter()
        .filter(|d| !d.dizzy)
        .collect();
    let theirs: Vec<&Die> = state.player(player.other()).active.iter().collect();
    let mut candidates = Vec::new();
    for a in &mine {
        let kinds = hooks::attack_kinds(a);
        for t in &theirs {
            for kind in [AttackKind::Power, AttackKind::Shadow, AttackKind::Trip] {
                if kinds.contains(&kind) {
                    candidates.push(DeclaredAttack {
                        kind,
                        attackers: vec![a.id],
                        targets: vec![t.id],
                    });
                }
            }
        }
        for kind in [AttackKind::Speed, AttackKind::Berserk] {
            if kinds.contains(&kind) {
                for targets in subsets(&theirs) {
                    candidates.push(DeclaredAttack {
                        kind,
                        attackers: vec![a.id],
                        targets,
                    });
                }
            }
        }
    }
    for t in &theirs {
        for attackers in subsets(&mine) {
            candidates.push(DeclaredAttack {
                kind: AttackKind::Skill,
                attackers,
                targets: vec![t.id],
            });
        }
    }
    candidates
}

/// Every legal attack for `player` right now, without Fire assistance.
pub(crate) fn possible_attacks(state: &GameState, player: PlayerId) -> Vec<DeclaredAttack> {
    let mut attacks = attack_shapes(state, player);
    attacks.retain(|attack| check_attack(state, player, attack).is_ok());
    attacks
}
