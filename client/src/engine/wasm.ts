import init, { WasmMatch, catalog_json, skills_json, swing_ranges_json } from "../wasm/buttonmen";
import type { EngineMatch } from "../adapters/GameAdapter";
import type { Catalog } from "./generated/Catalog";
import type { DieSkill } from "./generated/DieSkill";
import type { MatchSetup } from "./generated/MatchSetup";
import type { SkillInfo } from "./generated/SkillInfo";
import type { SwingRange } from "./generated/SwingRange";

let ready: Promise<void> | null = null;

export function loadEngine(): Promise<void> {
  ready ??= init().then(() => undefined);
  return ready;
}

/** Original buttons, plus the optional local buttonweavers import (public/buttonweavers/buttons.json). */
export async function loadCatalog(): Promise<Catalog> {
  await loadEngine();
  // Without the import, Vite answers with index.html: keep the response only if it is JSON.
  const imported = await fetch("./buttonweavers/buttons.json")
    .then((r) => (r.ok && (r.headers.get("content-type") ?? "").includes("json") ? r.text() : null))
    .catch(() => null);
  return JSON.parse(catalog_json(imported)) as Catalog;
}

/** R 1.2: swing letters and their ranges, from the engine. */
export async function loadSwingRanges(): Promise<SwingRange[]> {
  await loadEngine();
  return JSON.parse(swing_ranges_json()) as SwingRange[];
}

export async function loadSkills(): Promise<Map<DieSkill, SkillInfo>> {
  await loadEngine();
  const skills = JSON.parse(skills_json()) as SkillInfo[];
  return new Map(skills.map((s) => [s.skill, s]));
}

/** Requires `loadEngine()` to have resolved. */
export function createMatch(setup: MatchSetup): EngineMatch {
  return new WasmMatch(JSON.stringify(setup));
}
