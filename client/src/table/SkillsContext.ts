import { createContext } from "react";
import type { DieSkill } from "../engine/generated/DieSkill";
import type { SkillInfo } from "../engine/generated/SkillInfo";

/** Skill names and help texts, as provided by the engine. */
export const SkillsContext = createContext<Map<DieSkill, SkillInfo>>(new Map());
