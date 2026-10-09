import type { Action } from "../engine/generated/Action";
import type { ApplyOutcome } from "../engine/generated/ApplyOutcome";
import type { GameEvent } from "../engine/generated/GameEvent";
import type { GameView } from "../engine/generated/GameView";
import type { PlayerId } from "../engine/generated/PlayerId";
import type { Viewer } from "../engine/generated/Viewer";
import type { EngineMatch } from "./GameAdapter";

export function viewAs(engine: EngineMatch, viewer: Viewer): GameView {
  return JSON.parse(engine.view(JSON.stringify(viewer))) as GameView;
}

/** Applies an action; a rule refusal or a malformed input both come back as `error`. */
export function applyAction(
  engine: EngineMatch,
  player: PlayerId,
  action: Action,
): { events: GameEvent[] } | { error: string } {
  let outcome: ApplyOutcome;
  try {
    outcome = JSON.parse(engine.apply(JSON.stringify(player), JSON.stringify(action))) as ApplyOutcome;
  } catch (e) {
    // Malformed input is refused by the bridge with an exception.
    return { error: e instanceof Error ? e.message : String(e) };
  }
  return outcome.type === "Rejected" ? { error: outcome.message } : { events: outcome.events };
}
