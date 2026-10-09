import { describe, expect, it } from "vitest";
import type { DieView } from "../../engine/generated/DieView";
import type { SkillInfo } from "../../engine/generated/SkillInfo";
import { attackLabel, skillTitle } from "../labels";

describe("labels", () => {
  it("names attack kinds", () => {
    expect(attackLabel("Power")).toBe("Power");
    expect(attackLabel("Skill")).toBe("Skill");
  });

  it("builds a die tooltip from engine-provided skill texts", () => {
    const die: DieView = {
      id: 3,
      notation: "p(6)",
      skills: ["Poison"],
      sides: { Single: 6 },
      min: 1,
      max: 6,
      value: 4,
      dizzy: false,
    };
    const skills = new Map<SkillInfo["skill"], SkillInfo>([
      ["Poison", { skill: "Poison", letter: "p", name: "Poison", rule: "R 6.7", description: "Points négatifs." }],
    ]);
    expect(skillTitle(die, skills)).toBe("p(6) — 4 sur 1–6\nPoison : Points négatifs.");
  });
});
