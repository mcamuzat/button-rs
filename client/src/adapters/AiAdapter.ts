import type { Action } from "../engine/generated/Action";
import type { Level } from "../engine/generated/Level";
import type { PlayerId } from "../engine/generated/PlayerId";
import type { AiBrain } from "../ai/brain";
import { applyAction, viewAs } from "./engine";
import type { EngineMatch, GameAdapter, Snapshot } from "./GameAdapter";

const HUMAN: PlayerId = "One";
const AI: PlayerId = "Two";

export function levelLabel(level: Level): string {
  switch (level) {
    case "Easy":
      return "Facile";
    case "Normal":
      return "Normal";
    case "Hard":
      return "Difficile";
  }
}

const randomSeed = () => crypto.getRandomValues(new Uint32Array(1))[0];

/** The human (seat One) against the engine's AI (seat Two), which runs in a worker. */
export class AiAdapter implements GameAdapter {
  readonly handoff = false;
  readonly mySeat = HUMAN;
  readonly names: Record<PlayerId, string>;
  private snapshot: Snapshot;
  private readonly listeners = new Set<() => void>();
  private disposed = false;

  constructor(
    private readonly engine: EngineMatch,
    private readonly brain: AiBrain,
    private readonly level: Level,
    private readonly nextSeed: () => number = randomSeed,
  ) {
    this.names = { One: "Vous", Two: `IA (${levelLabel(level)})` };
    this.snapshot = { view: viewAs(engine, { type: "Seat", player: HUMAN }), lastEvents: [], version: 0, thinking: false, failure: null };
    this.askAiIfNeeded();
  }

  getSnapshot = (): Snapshot => this.snapshot;

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  dispatch(player: PlayerId, action: Action): string | null {
    const error = this.play(player, action);
    if (error === null) this.askAiIfNeeded();
    return error;
  }

  dispose(): void {
    this.disposed = true;
    this.brain.dispose();
    this.engine.free();
  }

  private play(player: PlayerId, action: Action): string | null {
    const outcome = applyAction(this.engine, player, action);
    if ("error" in outcome) return outcome.error;
    const view = viewAs(this.engine, { type: "Seat", player: HUMAN });
    // The decision panels are keyed on `version`: an AI move that leaves the human's
    // decision unchanged (both choose their Swing at once) must not reset the human's input.
    const humanDecisionChanged =
      player === HUMAN || JSON.stringify(view.legal) !== JSON.stringify(this.snapshot.view.legal);
    this.update({
      view,
      lastEvents: outcome.events,
      version: this.snapshot.version + (humanDecisionChanged ? 1 : 0),
    });
    return null;
  }

  private update(change: Partial<Snapshot>) {
    this.snapshot = { ...this.snapshot, ...change };
    this.listeners.forEach((listener) => listener());
  }

  /** When the AI must act (the engine lists it among the actors), ask the brain. */
  private askAiIfNeeded() {
    if (this.disposed || this.snapshot.thinking || !this.snapshot.view.actors.includes(AI)) return;
    this.update({ thinking: true });
    this.brain
      .think({ stateJson: this.engine.state_json(), player: AI, level: this.level, seed: this.nextSeed() })
      .then((action) => {
        if (this.disposed) return;
        this.update({ thinking: false });
        if (action === null) return;
        const error = this.play(AI, action);
        if (error !== null) this.update({ failure: error });
        else this.askAiIfNeeded();
      })
      .catch((e: unknown) => {
        if (this.disposed) return;
        this.update({ thinking: false, failure: e instanceof Error ? e.message : String(e) });
      });
  }
}
