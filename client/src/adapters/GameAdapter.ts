import type { Action } from "../engine/generated/Action";
import type { GameEvent } from "../engine/generated/GameEvent";
import type { GameView } from "../engine/generated/GameView";
import type { PlayerId } from "../engine/generated/PlayerId";

/** The part of `WasmMatch` the adapters use (lets tests use a fake). */
export interface EngineMatch {
  view(viewerJson: string): string;
  apply(playerJson: string, actionJson: string): string;
  /** The full state, handed to the AI worker (never parsed or displayed by the client). */
  state_json(): string;
  free(): void;
}

export interface Snapshot {
  view: GameView;
  lastEvents: GameEvent[];
  /** Grows with every accepted action: decision panels are keyed on it so their input
   * state never carries over from one decision (or one player) to the next. */
  version: number;
  /** The AI is computing its move. */
  thinking: boolean;
  /** The AI could not play (worker error): the match cannot go on. */
  failure: string | null;
}

/** One interface for every way of playing: hotseat, against the AI, online (later). */
export interface GameAdapter {
  /** Shared screen: hide secret choices behind a "pass the screen" overlay. */
  readonly handoff: boolean;
  readonly names: Record<PlayerId, string>;
  /** The seat of the person in front of this screen, when there is exactly one. */
  readonly mySeat: PlayerId | null;
  getSnapshot(): Snapshot;
  subscribe(listener: () => void): () => void;
  /** Returns the engine's message when the action is refused, `null` otherwise. */
  dispatch(player: PlayerId, action: Action): string | null;
  dispose(): void;
}
