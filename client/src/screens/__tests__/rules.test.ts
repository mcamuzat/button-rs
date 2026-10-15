import { describe, expect, it } from "vitest";
import type { DieSkill } from "../../engine/generated/DieSkill";
import type { SkillInfo } from "../../engine/generated/SkillInfo";
import { skillsByName } from "../rules";

const info = (skill: DieSkill, name: string, letter: string): SkillInfo => ({
  skill,
  letter,
  name,
  rule: "R 6.1",
  description: "…",
});

describe("skillsByName", () => {
  it("lists the engine-provided skills alphabetically, accents ignored", () => {
    const skills = new Map<DieSkill, SkillInfo>([
      ["Value", info("Value", "Value", "v")],
      ["Boom", info("Boom", "Boom", "b")],
      ["Berserk", info("Berserk", "Berserk", "B")],
    ]);
    expect(skillsByName(skills).map((s) => s.name)).toEqual(["Berserk", "Boom", "Value"]);
  });
});
