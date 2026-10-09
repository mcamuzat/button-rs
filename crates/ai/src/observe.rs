use buttonmen_engine::{GameState, PlayerId, SeededRng, WaitingFor};

/// What `me` may know of `state` (spec §7):
/// - the opponent's unrevealed Swing/Option choice is forgotten (R 3.1), and the
///   opponent is asked again, so simulations sample it instead of reading it;
/// - the random generator is replaced by one seeded with `seed`, so future rolls are unknown.
pub fn observe(state: &GameState, me: PlayerId, seed: u64) -> GameState {
    let mut seen = reseeded(state, seed);
    let opponent = me.other();
    if seen.player_mut(opponent).pending_choice.take().is_some()
        && let WaitingFor::SpecifyDice { pending } = &mut seen.waiting_for
        && !pending.contains(&opponent)
    {
        pending.push(opponent);
    }
    seen
}

/// A copy of `state` whose future rolls come from `seed`.
pub fn reseeded(state: &GameState, seed: u64) -> GameState {
    let mut copy = state.clone();
    copy.rng = SeededRng::new(seed);
    copy
}
