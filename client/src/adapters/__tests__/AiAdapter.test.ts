import { describe, expect, it, vi } from "vitest";
import type { Action } from "../../engine/generated/Action";
import type { AiBrain, AiRequest } from "../../ai/brain";
import { AiAdapter } from "../AiAdapter";
import type { EngineMatch } from "../GameAdapter";

/** A fake match where players alternate; it ends after `turns` accepted actions. */
function fakeMatch(first: "One" | "Two", turns = 10): EngineMatch {
  let turn: "One" | "Two" = first;
  let played = 0;
  return {
    view(viewerJson) {
      const over = played >= turns;
      return JSON.stringify({
        viewer: JSON.parse(viewerJson),
        actors: over ? [] : [turn],
        waiting_for: over ? { type: "GameOver", result: { winner: "One", reason: "Rounds" } } : { type: "Attack", player: turn },
        round: 1,
      });
    },
    apply(playerJson) {
      if (JSON.parse(playerJson) !== turn) {
        return JSON.stringify({ type: "Rejected", error: { type: "NotYourTurn" }, message: "pas votre tour" });
      }
      turn = turn === "One" ? "Two" : "One";
      played += 1;
      return JSON.stringify({ type: "Applied", events: [] });
    },
    state_json: () => `{"played":${played}}`,
    free: vi.fn(),
  };
}

/** A brain answering `Pass` when its promise is resolved by the test. */
function manualBrain() {
  const requests: AiRequest[] = [];
  const resolvers: ((a: Action | null) => void)[] = [];
  const brain: AiBrain = {
    think(request) {
      requests.push(request);
      return new Promise((resolve) => resolvers.push(resolve));
    },
    dispose: vi.fn(),
  };
  const answer = (action: Action | null = { type: "Pass" }) => resolvers.shift()!(action);
  return { brain, requests, answer };
}

const tick = () => new Promise((r) => setTimeout(r, 0));

describe("AiAdapter", () => {
  it("always shows the human seat and names the AI", () => {
    const { brain } = manualBrain();
    const adapter = new AiAdapter(fakeMatch("One"), brain, "Normal", () => 7);
    expect(adapter.getSnapshot().view.viewer).toEqual({ type: "Seat", player: "One" });
    expect(adapter.names).toEqual({ One: "Vous", Two: "IA (Normal)" });
    expect(adapter.handoff).toBe(false);
  });

  it("asks the AI after the human's move and plays its answer", async () => {
    const { brain, requests, answer } = manualBrain();
    const adapter = new AiAdapter(fakeMatch("One"), brain, "Hard", () => 7);
    expect(adapter.dispatch("One", { type: "Pass" })).toBeNull();
    expect(adapter.getSnapshot().thinking).toBe(true);
    expect(requests).toEqual([{ stateJson: '{"played":1}', player: "Two", level: "Hard", seed: 7 }]);
    answer();
    await tick();
    expect(adapter.getSnapshot().thinking).toBe(false);
    expect(adapter.getSnapshot().version).toBe(2);
    expect(adapter.getSnapshot().view.actors).toEqual(["One"]);
  });

  it("lets the AI start when it must act first", () => {
    const { brain, requests } = manualBrain();
    new AiAdapter(fakeMatch("Two"), brain, "Easy", () => 1);
    expect(requests).toHaveLength(1);
  });

  it("refuses human actions out of turn with the engine's message", () => {
    const { brain } = manualBrain();
    const adapter = new AiAdapter(fakeMatch("Two"), brain, "Easy", () => 1);
    expect(adapter.dispatch("One", { type: "Pass" })).toBe("pas votre tour");
  });

  it("reports a brain failure", async () => {
    const brain: AiBrain = { think: () => Promise.reject(new Error("worker crashed")), dispose: vi.fn() };
    const adapter = new AiAdapter(fakeMatch("Two"), brain, "Easy", () => 1);
    await tick();
    expect(adapter.getSnapshot().thinking).toBe(false);
    expect(adapter.getSnapshot().failure).toBe("worker crashed");
  });

  it("ignores_a_late_answer_after_dispose", async () => {
    // Review Focus 3.
    const { brain, answer } = manualBrain();
    const engine = fakeMatch("Two");
    const adapter = new AiAdapter(engine, brain, "Easy", () => 1);
    adapter.dispose();
    answer();
    await tick();
    expect(engine.free).toHaveBeenCalledOnce();
    expect(brain.dispose).toHaveBeenCalledOnce();
  });

  it("stops_asking_once_the_match_is_over", async () => {
    // Review Focus 4: the match ends on the AI's move.
    const { brain, requests, answer } = manualBrain();
    const adapter = new AiAdapter(fakeMatch("One", 2), brain, "Easy", () => 1);
    adapter.dispatch("One", { type: "Pass" });
    answer();
    await tick();
    expect(requests).toHaveLength(1);
    expect(adapter.getSnapshot().view.waiting_for.type).toBe("GameOver");
    expect(adapter.getSnapshot().thinking).toBe(false);
  });
});
