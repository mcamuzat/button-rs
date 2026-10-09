import type { Action } from "../engine/generated/Action";
import type { ApplyOutcome } from "../engine/generated/ApplyOutcome";
import type { GameView } from "../engine/generated/GameView";
import type { PlayerId } from "../engine/generated/PlayerId";
import type { Viewer } from "../engine/generated/Viewer";
import type { EngineMatch, GameAdapter, Snapshot } from "./GameAdapter";

/** Hotseat: both players share the screen, which shows the seat that must act. */
export class LocalAdapter implements GameAdapter {
  private snapshot: Snapshot;
  private readonly listeners = new Set<() => void>();

  constructor(private readonly engine: EngineMatch) {
    this.snapshot = { view: this.currentView(), lastEvents: [], version: 0 };
  }

  private viewAs(viewer: Viewer): GameView {
    return JSON.parse(this.engine.view(JSON.stringify(viewer))) as GameView;
  }

  private currentView(): GameView {
    const spectator = this.viewAs({ type: "Spectator" });
    const actor = spectator.actors[0];
    return actor === undefined ? spectator : this.viewAs({ type: "Seat", player: actor });
  }

  getSnapshot = (): Snapshot => this.snapshot;

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  dispatch(player: PlayerId, action: Action): string | null {
    let outcome: ApplyOutcome;
    try {
      outcome = JSON.parse(this.engine.apply(JSON.stringify(player), JSON.stringify(action))) as ApplyOutcome;
    } catch (e) {
      // Malformed input is refused by the bridge with an exception: report it like a rule refusal.
      return e instanceof Error ? e.message : String(e);
    }
    if (outcome.type === "Rejected") return outcome.message;
    this.snapshot = { view: this.currentView(), lastEvents: outcome.events, version: this.snapshot.version + 1 };
    this.listeners.forEach((listener) => listener());
    return null;
  }

  dispose(): void {
    this.engine.free();
  }
}
