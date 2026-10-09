import type { AttackKind } from "../engine/generated/AttackKind";
import type { DieSkill } from "../engine/generated/DieSkill";
import type { DieView } from "../engine/generated/DieView";
import type { SkillInfo } from "../engine/generated/SkillInfo";

/** Attack kinds keep their Button Men names (docs/RULES.md). */
export function attackLabel(kind: AttackKind): string {
  return kind;
}

/** Tooltip of a die: notation, value and range, then each skill's engine-provided text. */
export function skillTitle(die: DieView, skills: Map<DieSkill, SkillInfo>): string {
  const head = `${die.notation} — ${die.value} sur ${die.min}–${die.max}`;
  const lines = die.skills.map((s) => {
    const info = skills.get(s);
    return info ? `${info.name} : ${info.description}` : s;
  });
  return [head, ...lines].join("\n");
}
