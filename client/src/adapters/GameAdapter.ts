import type { Action } from "../engine/generated/Action";
import type { GameEvent } from "../engine/generated/GameEvent";
import type { GameView } from "../engine/generated/GameView";
import type { PlayerId } from "../engine/generated/PlayerId";

/** The part of `WasmMatch` the adapters use (lets tests use a fake). */
export interface EngineMatch {
  view(viewerJson: string): string;
  apply(playerJson: string, actionJson: string): string;
  free(): void;
}

export interface Snapshot {
  view: GameView;
  lastEvents: GameEvent[];
  /** Grows with every accepted action: decision panels are keyed on it so their input
   * state never carries over from one decision (or one player) to the next. */
  version: number;
}

/** One interface for every way of playing (hotseat now; AI and online in later milestones). */
export interface GameAdapter {
  getSnapshot(): Snapshot;
  subscribe(listener: () => void): () => void;
  /** Returns the engine's message when the action is refused, `null` otherwise. */
  dispatch(player: PlayerId, action: Action): string | null;
  dispose(): void;
}
