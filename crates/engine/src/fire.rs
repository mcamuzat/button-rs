use std::collections::BTreeSet;

use crate::action::{AttackKind, DeclaredAttack};
use crate::attack::{attack_shapes, check_attack, skill_sums};
use crate::error::{AttackError, FireError, RuleError};
use crate::event::GameEvent;
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

/// R 6.3: is `attack` a Power or Skill attack that fails only on values, and that
/// Fire can make legal?
pub(crate) fn can_help(state: &GameState, player: PlayerId, attack: &DeclaredAttack) -> bool {
    if !matches!(attack.kind, AttackKind::Power | AttackKind::Skill) {
        return false;
    }
    if check_attack(state, player, attack) != Err(AttackError::ValuesDoNotMatch) {
        return false;
    }
    let capacity: u32 = fire_dice(state, player, attack)
        .iter()
        .map(|(_, g)| u32::from(*g))
        .sum();
    let room: u32 = attacker_room(state, player, attack)
        .iter()
        .map(|(_, r)| u32::from(*r))
        .sum();
    let budget = i32::try_from(capacity.min(room)).unwrap_or(i32::MAX);
    let me = state.player(player);
    let target = state
        .player(player.other())
        .die(attack.targets[0])
        .expect("checked by check_attack")
        .value;
    let target = i32::from(target);
    match attack.kind {
        AttackKind::Power => {
            let a = i32::from(me.die(attack.attackers[0]).expect("checked").value);
            target - a <= budget
        }
        AttackKind::Skill => {
            let attackers: Vec<_> = attack
                .attackers
                .iter()
                .filter_map(|&id| me.die(id))
                .collect();
            skill_sums(&attackers)
                .iter()
                .any(|s| *s < target && target - s <= budget)
        }
        AttackKind::Shadow | AttackKind::Speed | AttackKind::Trip | AttackKind::Berserk => false,
    }
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
    resolve::resolve_attack(state, player, attack, events);
    round::end_turn(state, player, events);
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
