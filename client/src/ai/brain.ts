import type { Action } from "../engine/generated/Action";
import type { Level } from "../engine/generated/Level";
import type { PlayerId } from "../engine/generated/PlayerId";

export interface AiRequest {
  stateJson: string;
  player: PlayerId;
  level: Level;
  seed: number;
}

/** Where the AI's moves come from (a Web Worker in the app, a fake in tests). */
export interface AiBrain {
  think(request: AiRequest): Promise<Action | null>;
  dispose(): void;
}

/** Messages between `WorkerBrain` and `ai.worker.ts`. */
export type WorkerRequest = AiRequest & { id: number };
export type WorkerResponse = { id: number; actionJson: string | null } | { id: number; error: string };

/** Runs the engine's AI in a Web Worker, so the page stays responsive while it thinks. */
export class WorkerBrain implements AiBrain {
  private readonly worker: Worker;
  private readonly pending = new Map<number, { resolve: (a: Action | null) => void; reject: (e: Error) => void }>();
  private nextId = 0;

  constructor(
    createWorker: () => Worker = () => new Worker(new URL("./ai.worker.ts", import.meta.url), { type: "module" }),
  ) {
    this.worker = createWorker();
    // A worker that cannot load or crashes never answers: fail every pending request.
    this.worker.onerror = (event: ErrorEvent) => this.failAll(event.message || "le calcul de l'IA a échoué");
    this.worker.onmessageerror = () => this.failAll("réponse illisible du calcul de l'IA");
    this.worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
      const answer = event.data;
      const waiter = this.pending.get(answer.id);
      if (!waiter) return;
      this.pending.delete(answer.id);
      if ("error" in answer) waiter.reject(new Error(answer.error));
      else waiter.resolve(answer.actionJson === null ? null : (JSON.parse(answer.actionJson) as Action));
    };
  }

  think(request: AiRequest): Promise<Action | null> {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.worker.postMessage({ ...request, id } satisfies WorkerRequest);
    });
  }

  private failAll(message: string) {
    for (const waiter of this.pending.values()) waiter.reject(new Error(message));
    this.pending.clear();
  }

  dispose(): void {
    this.worker.terminate();
    this.pending.clear();
  }
}
