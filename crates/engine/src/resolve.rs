use crate::action::{AttackKind, DeclaredAttack};
use crate::attack::check_attack;
use crate::error::RuleError;
use crate::event::{GameEvent, ResizeCause};
use crate::hooks;
use crate::ids::{DieId, PlayerId};
use crate::round;
use crate::skill::DieSkill;
use crate::state::{CapturedDie, GameState, WaitingFor};

/// Handles `Action::Attack`.
pub(crate) fn declare_attack(
    state: &mut GameState,
    player: PlayerId,
    attack: DeclaredAttack,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    match state.waiting_for {
        WaitingFor::Attack { player: p } if p == player => {}
        WaitingFor::Attack { .. } => return Err(RuleError::NotYourTurn),
        _ => return Err(RuleError::WrongPhase),
    }
    if let Err(reason) = check_attack(state, player, &attack) {
        // R 6.3: Fire may still make a Power or Skill attack legal.
        if crate::fire::can_help(state, player, &attack) {
            state.waiting_for = WaitingFor::AdjustFire { player, attack };
            return Ok(());
        }
        return Err(RuleError::IllegalAttack { reason });
    }
    let again = resolve_attack(state, player, attack, events);
    round::end_turn(state, player, again, events);
    Ok(())
}

/// R 4.4 + R 6.1 + R 6.14 + R 6.18 + R 6.19: carries out an attack already known to be
/// legal. Returns whether the attacker plays again (R 6.19).
pub(crate) fn resolve_attack(
    state: &mut GameState,
    player: PlayerId,
    attack: DeclaredAttack,
    events: &mut Vec<GameEvent>,
) -> bool {
    events.push(GameEvent::Attacked {
        player,
        attack: attack.clone(),
    });
    state.consecutive_passes = 0;
    match attack.kind {
        AttackKind::Trip => resolve_trip(
            state,
            player,
            attack.attackers[0],
            attack.targets[0],
            events,
        ),
        AttackKind::Power
        | AttackKind::Skill
        | AttackKind::Shadow
        | AttackKind::Speed
        | AttackKind::Berserk => {
            for &target in &attack.targets {
                capture(state, player, target, &attack.attackers, events);
            }
            for &attacker in &attack.attackers {
                if attack.kind == AttackKind::Berserk {
                    shrink(state, player, attacker, events);
                }
                reroll(state, player, attacker, hooks::RerollCause::Normal, events);
            }
        }
    }
    // R 6.20: a Rage die that took part in the attack loses Rage.
    for &a in &attack.attackers {
        if let Some(die) = state.player_mut(player).die_mut(a) {
            die.skills.remove(DieSkill::Rage);
        }
    }
    // R 6.19: one extra turn at most, when a rerolled TimeAndSpace attacker shows odd.
    let plays_again = attack.attackers.iter().any(|&a| {
        state
            .player(player)
            .die(a)
            .is_some_and(hooks::grants_extra_turn)
    });
    // R 6.18: the owner's other Ornery dice reroll after the attack.
    let ornery: Vec<DieId> = state
        .player(player)
        .active
        .iter()
        .filter(|d| !attack.attackers.contains(&d.id) && hooks::rerolls_after_any_attack(d))
        .map(|d| d.id)
        .collect();
    for id in ornery {
        reroll(state, player, id, hooks::RerollCause::Normal, events);
    }
    plays_again
}

/// R 4.4: the target joins the attacker's score pile.
fn capture(
    state: &mut GameState,
    player: PlayerId,
    target: DieId,
    attackers: &[DieId],
    events: &mut Vec<GameEvent>,
) {
    let defender = player.other();
    let scoring = {
        let attacker_dice: Vec<_> = attackers
            .iter()
            .filter_map(|&a| state.player(player).die(a))
            .collect();
        let target_die = state.player(defender).die(target).expect("legal target");
        hooks::capture_scoring(&attacker_dice, target_die)
    };
    let index = state
        .player(defender)
        .active
        .iter()
        .position(|d| d.id == target)
        .expect("legal target");
    let die = state.player_mut(defender).active.remove(index);
    // R 6.20: a captured Rage die leaves a clone without Rage, rolled, in its place.
    if hooks::leaves_a_clone(&die) {
        let mut clone = die.clone();
        clone.id = state.fresh_die_id();
        clone.skills.remove(DieSkill::Rage);
        clone.dizzy = false;
        clone.value = clone.sides.roll(&mut state.rng);
        events.push(GameEvent::RageReplaced {
            player: defender,
            captured: target,
            replacement: clone.id,
        });
        state.player_mut(defender).active.insert(index, clone);
    }
    state
        .player_mut(player)
        .captured
        .push(CapturedDie { die, scoring });
    events.push(GameEvent::DieCaptured {
        by: player,
        die: target,
    });
}

fn reroll(
    state: &mut GameState,
    owner: PlayerId,
    id: DieId,
    cause: hooks::RerollCause,
    events: &mut Vec<GameEvent>,
) {
    let GameState { players, rng, .. } = state;
    if let Some(die) = players[owner.index()].die_mut(id) {
        if let Some(from) = hooks::reroll(die, rng, cause) {
            events.push(GameEvent::DieResized {
                die: id,
                from,
                to: die.sides,
                cause: ResizeCause::Mighty,
            });
        }
        events.push(GameEvent::DieRerolled {
            die: id,
            value: die.value,
        });
    }
}

/// R 6.1: half the sides (rounded up), and the die stops being Berserk.
fn shrink(state: &mut GameState, owner: PlayerId, id: DieId, events: &mut Vec<GameEvent>) {
    let die = state.player_mut(owner).die_mut(id).expect("legal attacker");
    die.sides = die.sides.halved();
    die.skills.remove(DieSkill::Berserk);
    events.push(GameEvent::DieShrunk {
        die: id,
        sides: die.sides,
    });
}

/// R 6.14: both dice are rerolled; the target is captured if the Trip die shows at least as much.
fn resolve_trip(
    state: &mut GameState,
    player: PlayerId,
    attacker: DieId,
    target: DieId,
    events: &mut Vec<GameEvent>,
) {
    reroll(
        state,
        player,
        attacker,
        hooks::RerollCause::TripAttacker,
        events,
    );
    reroll(
        state,
        player.other(),
        target,
        hooks::RerollCause::Normal,
        events,
    );
    let attacker_value = state
        .player(player)
        .die(attacker)
        .expect("legal attacker")
        .value;
    let target_value = state
        .player(player.other())
        .die(target)
        .expect("legal target")
        .value;
    let captured = attacker_value >= target_value;
    events.push(GameEvent::TripRolled {
        attacker,
        attacker_value,
        target,
        target_value,
        captured,
    });
    if captured {
        capture(state, player, target, &[attacker], events);
    }
}
