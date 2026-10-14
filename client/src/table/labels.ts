import type { AttackKind } from "../engine/generated/AttackKind";
import type { DieSkill } from "../engine/generated/DieSkill";
import type { DieView } from "../engine/generated/DieView";
import type { GameEvent } from "../engine/generated/GameEvent";
import type { PlayerId } from "../engine/generated/PlayerId";
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

/** "À vous de jouer" for the person in front of the screen, "À <name>" otherwise. */
export function turnText(names: Record<PlayerId, string>, mySeat: PlayerId | null, seat: PlayerId): string {
  return seat === mySeat ? "À vous de jouer" : `À ${names[seat]}`;
}

/** "Vous gagnez" for the person in front of the screen, "<name> gagne" otherwise. */
export function winnerText(names: Record<PlayerId, string>, mySeat: PlayerId | null, winner: PlayerId): string {
  return winner === mySeat ? "Vous gagnez" : `${names[winner]} gagne`;
}

/** "Vous rejouez" / "<name> rejoue" when the last action gave an extra turn (R 6.19). */
export function extraTurnText(
  events: GameEvent[],
  names: Record<PlayerId, string>,
  mySeat: PlayerId | null,
): string | null {
  const extra = events.find((e) => e.type === "ExtraTurn");
  if (!extra || extra.type !== "ExtraTurn") return null;
  return extra.player === mySeat ? "Vous rejouez (TimeAndSpace)" : `${names[extra.player]} rejoue (TimeAndSpace)`;
}
