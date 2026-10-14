use std::collections::{BTreeMap, BTreeSet};

use crate::action::{AttackKind, DeclaredAttack};
use crate::attack::{attack_shapes, check_attack};
use crate::die::Die;
use crate::error::{AttackError, FireError, RuleError};
use crate::event::GameEvent;
use crate::hooks;
use crate::ids::{DieId, PlayerId};
use crate::resolve;
use crate::round;
use crate::skill::DieSkill;
use crate::state::{GameState, WaitingFor};

/// R 6.3: Fire dice not taking part in `attack`, with how much each can give.
pub(crate) fn fire_dice(
    state: &GameState,
    player: PlayerId,
    attack: &DeclaredAttack,
) -> Vec<(DieId, u8)> {
    state
        .player(player)
        .active
        .iter()
        .filter(|d| d.has(DieSkill::Fire) && !attack.attackers.contains(&d.id))
        .map(|d| (d.id, d.value - d.min()))
        .filter(|(_, give)| *give > 0)
        .collect()
}

/// R 6.3: how much each attacker can be raised without leaving its range.
pub(crate) fn attacker_room(
    state: &GameState,
    player: PlayerId,
    attack: &DeclaredAttack,
) -> Vec<(DieId, u8)> {
    attack
        .attackers
        .iter()
        .filter_map(|&id| state.player(player).die(id))
        .map(|d| (d.id, d.max() - d.value))
        .collect()
}

/// R 6.3: the smallest boosts on the attackers, paid by Fire dice, that make `attack`
/// legal — or `None` when Fire cannot. Only Power and Skill attacks that fail on values
/// alone qualify. Each attacker's options are computed on a raised copy of the die, so a
/// Konstant attacker (counted with either sign, R 6.5) is handled exactly.
pub(crate) fn plan_boosts(
    state: &GameState,
    player: PlayerId,
    attack: &DeclaredAttack,
) -> Option<Vec<(DieId, u8)>> {
    if !matches!(attack.kind, AttackKind::Power | AttackKind::Skill)
        || check_attack(state, player, attack) != Err(AttackError::ValuesDoNotMatch)
    {
        return None;
    }
    let capacity: u32 = fire_dice(state, player, attack)
        .iter()
        .map(|(_, g)| u32::from(*g))
        .sum();
    if capacity == 0 {
        return None;
    }
    let me = state.player(player);
    let attackers: Vec<&Die> = attack
        .attackers
        .iter()
        .filter_map(|&id| me.die(id))
        .collect();
    let target = i32::from(
        state
            .player(player.other())
            .die(attack.targets[0])
            .expect("checked by check_attack")
            .value,
    );
    // (sum reached, boost spent) -> one way to get there.
    let mut reachable: BTreeMap<(i32, u32), Vec<(DieId, u8)>> =
        BTreeMap::from([((0, 0), Vec::new())]);
    for die in &attackers {
        let mut next = BTreeMap::new();
        for ((sum, spent), boosts) in &reachable {
            for boost in 0..=(die.max() - die.value) {
                let spent = spent + u32::from(boost);
                if spent > capacity {
                    break;
                }
                let mut raised = (*die).clone();
                raised.value += boost;
                let values: Vec<i32> = match attack.kind {
                    AttackKind::Power => vec![i32::from(raised.value)],
                    _ => hooks::skill_values(&raised),
                };
                for v in values {
                    next.entry((sum + v, spent)).or_insert_with(|| {
                        let mut boosts = boosts.clone();
                        if boost > 0 {
                            boosts.push((die.id, boost));
                        }
                        boosts
                    });
                }
            }
        }
        reachable = next;
    }
    let reaches = |sum: i32| match attack.kind {
        // R 4.2: Power needs at least the target's value.
        AttackKind::Power => sum >= target,
        _ => sum == target,
    };
    reachable
        .into_iter()
        .filter(|((sum, spent), _)| *spent > 0 && reaches(*sum))
        .min_by_key(|((_, spent), _)| *spent)
        .map(|(_, boosts)| boosts)
}

/// R 6.3: can Fire make `attack` legal?
pub(crate) fn can_help(state: &GameState, player: PlayerId, attack: &DeclaredAttack) -> bool {
    plan_boosts(state, player, attack).is_some()
}

/// Attacks that are possible only with Fire assistance (they count for R 4.1).
pub(crate) fn assisted_attacks(state: &GameState, player: PlayerId) -> Vec<DeclaredAttack> {
    let mut attacks = attack_shapes(state, player);
    attacks.retain(|attack| can_help(state, player, attack));
    attacks
}

fn fire_error(reason: FireError) -> RuleError {
    RuleError::InvalidFire { reason }
}

/// Handles `Action::FireTurnDown` (R 6.3).
pub(crate) fn fire_turn_down(
    state: &mut GameState,
    player: PlayerId,
    fire: Vec<(DieId, u8)>,
    boosts: Vec<(DieId, u8)>,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    let attack = match &state.waiting_for {
        WaitingFor::AdjustFire { player: p, attack } if *p == player => attack.clone(),
        WaitingFor::AdjustFire { .. } => return Err(RuleError::NotYourTurn),
        _ => return Err(RuleError::WrongPhase),
    };
    let mut seen = BTreeSet::new();
    for &(id, amount) in &fire {
        let die = state
            .player(player)
            .die(id)
            .ok_or(RuleError::UnknownDie { die: id })?;
        if !die.has(DieSkill::Fire) || attack.attackers.contains(&id) || !seen.insert(id) {
            return Err(fire_error(FireError::NotFire { die: id }));
        }
        if amount == 0 || u16::from(die.value) < u16::from(die.min()) + u16::from(amount) {
            return Err(fire_error(FireError::OutOfRange { die: id }));
        }
    }
    for &(id, amount) in &boosts {
        if !attack.attackers.contains(&id) || !seen.insert(id) {
            return Err(fire_error(FireError::NotAttacker { die: id }));
        }
        let die = state
            .player(player)
            .die(id)
            .expect("attackers are the player's dice");
        if amount == 0 || u16::from(die.value) + u16::from(amount) > u16::from(die.max()) {
            return Err(fire_error(FireError::OutOfRange { die: id }));
        }
    }
    let total = |list: &[(DieId, u8)]| list.iter().map(|(_, a)| u32::from(*a)).sum::<u32>();
    if total(&fire) != total(&boosts) {
        return Err(fire_error(FireError::AmountsDiffer));
    }
    let me = state.player_mut(player);
    for &(id, amount) in &fire {
        me.die_mut(id).expect("checked").value -= amount;
    }
    for &(id, amount) in &boosts {
        me.die_mut(id).expect("checked").value += amount;
    }
    if check_attack(state, player, &attack).is_err() {
        return Err(fire_error(FireError::AttackStillInvalid));
    }
    events.push(GameEvent::FireUsed {
        player,
        fire,
        boosts,
    });
    let again = resolve::resolve_attack(state, player, attack, events);
    round::end_turn(state, player, again, events);
    Ok(())
}

/// Handles `Action::CancelFire`: back to choosing an attack.
pub(crate) fn cancel_fire(state: &mut GameState, player: PlayerId) -> Result<(), RuleError> {
    match state.waiting_for {
        WaitingFor::AdjustFire { player: p, .. } if p == player => {
            state.waiting_for = WaitingFor::Attack { player };
            Ok(())
        }
        WaitingFor::AdjustFire { .. } => Err(RuleError::NotYourTurn),
        _ => Err(RuleError::WrongPhase),
    }
}
