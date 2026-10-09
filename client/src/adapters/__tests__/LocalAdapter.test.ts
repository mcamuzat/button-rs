import { describe, expect, it, vi } from "vitest";
import type { EngineMatch } from "../GameAdapter";
import { LocalAdapter } from "../LocalAdapter";

/** A fake engine: Two must act; One's actions are rejected. */
function fakeMatch(): EngineMatch & { viewers: string[] } {
  const viewers: string[] = [];
  return {
    viewers,
    view(viewerJson) {
      viewers.push(viewerJson);
      return JSON.stringify({ viewer: JSON.parse(viewerJson), actors: ["Two"], round: 1 });
    },
    apply(playerJson) {
      return JSON.parse(playerJson) === "Two"
        ? JSON.stringify({ type: "Applied", events: [{ type: "Passed", player: "Two" }] })
        : JSON.stringify({ type: "Rejected", error: { type: "NotYourTurn" }, message: "ce n'est pas à ce joueur d'agir" });
    },
    free: vi.fn(),
  };
}

describe("LocalAdapter", () => {
  it("shows the view of the first player who must act", () => {
    const engine = fakeMatch();
    const adapter = new LocalAdapter(engine);
    expect(adapter.getSnapshot().view.viewer).toEqual({ type: "Seat", player: "Two" });
  });

  it("notifies listeners and keeps events on success, returns the message on rejection", () => {
    const adapter = new LocalAdapter(fakeMatch());
    const listener = vi.fn();
    adapter.subscribe(listener);
    expect(adapter.dispatch("One", { type: "Pass" })).toBe("ce n'est pas à ce joueur d'agir");
    expect(listener).not.toHaveBeenCalled();
    expect(adapter.dispatch("Two", { type: "Pass" })).toBeNull();
    expect(listener).toHaveBeenCalledOnce();
    expect(adapter.getSnapshot().lastEvents).toEqual([{ type: "Passed", player: "Two" }]);
  });

  it("frees the engine on dispose", () => {
    const engine = fakeMatch();
    new LocalAdapter(engine).dispose();
    expect(engine.free).toHaveBeenCalled();
  });
});
