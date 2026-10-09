import type { Action } from "../engine/generated/Action";
import type { GameView } from "../engine/generated/GameView";
import type { PlayerId } from "../engine/generated/PlayerId";
import { applyAction, viewAs } from "./engine";
import type { EngineMatch, GameAdapter, Snapshot } from "./GameAdapter";

/** Hotseat: both players share the screen, which shows the seat that must act. */
export class LocalAdapter implements GameAdapter {
  readonly handoff = true;
  readonly names: Record<PlayerId, string> = { One: "Joueur 1", Two: "Joueur 2" };
  private snapshot: Snapshot;
  private readonly listeners = new Set<() => void>();

  constructor(private readonly engine: EngineMatch) {
    this.snapshot = { view: this.currentView(), lastEvents: [], version: 0, thinking: false, failure: null };
  }

  private currentView(): GameView {
    const spectator = viewAs(this.engine, { type: "Spectator" });
    const actor = spectator.actors[0];
    return actor === undefined ? spectator : viewAs(this.engine, { type: "Seat", player: actor });
  }

  getSnapshot = (): Snapshot => this.snapshot;

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  dispatch(player: PlayerId, action: Action): string | null {
    const outcome = applyAction(this.engine, player, action);
    if ("error" in outcome) return outcome.error;
    this.snapshot = {
      ...this.snapshot,
      view: this.currentView(),
      lastEvents: outcome.events,
      version: this.snapshot.version + 1,
    };
    this.listeners.forEach((listener) => listener());
    return null;
  }

  dispose(): void {
    this.engine.free();
  }
}
