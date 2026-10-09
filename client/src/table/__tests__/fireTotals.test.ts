import { describe, expect, it } from "vitest";
import { fireAction } from "../fireTotals";

describe("fireAction", () => {
  it("builds a FireTurnDown from non-zero amounts when totals match", () => {
    expect(fireAction({ 4: 2, 5: 0 }, { 1: 2 })).toEqual({
      type: "FireTurnDown",
      fire: [[4, 2]],
      boosts: [[1, 2]],
    });
  });

  it("returns null when totals differ or are zero", () => {
    expect(fireAction({ 4: 3 }, { 1: 2 })).toBeNull();
    expect(fireAction({ 4: 0 }, { 1: 0 })).toBeNull();
  });
});
