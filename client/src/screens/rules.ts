import type { DieSkill } from "../engine/generated/DieSkill";
import type { SkillInfo } from "../engine/generated/SkillInfo";

/** The engine-provided skills, by name (display order of the rules screen). */
export function skillsByName(skills: Map<DieSkill, SkillInfo>): SkillInfo[] {
  return [...skills.values()].sort((a, b) => a.name.localeCompare(b.name, "fr", { sensitivity: "base" }));
}
