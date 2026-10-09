import { describe, expect, it } from "vitest";
import { formatPoints } from "../format";

describe("formatPoints", () => {
  it("shows half-points as French decimals", () => {
    expect(formatPoints(28)).toBe("14");
    expect(formatPoints(7)).toBe("3,5");
    expect(formatPoints(-5)).toBe("−2,5");
    expect(formatPoints(0)).toBe("0");
  });
});
