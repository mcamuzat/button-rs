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
    state_json: () => "{}",
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

  it("numbers each decision so panels start fresh for every new decision", () => {
    const adapter = new LocalAdapter(fakeMatch());
    const first = adapter.getSnapshot().version;
    adapter.dispatch("One", { type: "Pass" });
    expect(adapter.getSnapshot().version).toBe(first);
    adapter.dispatch("Two", { type: "Pass" });
    expect(adapter.getSnapshot().version).toBe(first + 1);
  });

  it("turns an engine exception into a message instead of throwing", () => {
    const engine = fakeMatch();
    engine.apply = () => {
      throw new Error("invalid type: null");
    };
    const adapter = new LocalAdapter(engine);
    expect(adapter.dispatch("Two", { type: "Pass" })).toBe("invalid type: null");
  });

  it("uses the handoff screen and seat names of a shared screen", () => {
    const adapter = new LocalAdapter(fakeMatch());
    expect(adapter.handoff).toBe(true);
    expect(adapter.mySeat).toBeNull();
    expect(adapter.names).toEqual({ One: "Joueur 1", Two: "Joueur 2" });
    expect(adapter.getSnapshot().thinking).toBe(false);
  });

  it("frees the engine on dispose", () => {
    const engine = fakeMatch();
    new LocalAdapter(engine).dispose();
    expect(engine.free).toHaveBeenCalled();
  });
});
