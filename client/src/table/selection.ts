import type { AttackOption } from "../engine/generated/AttackOption";
import type { DieId } from "../engine/generated/DieId";

/** The dice the player has clicked so far (UI state only). */
export interface Selection {
  attackers: DieId[];
  targets: DieId[];
}

const sameSet = (a: DieId[], b: DieId[]) => a.length === b.length && a.every((id) => b.includes(id));
const contains = (all: DieId[], some: DieId[]) => some.every((id) => all.includes(id));

/** The engine-listed attacks made of exactly the selected dice. */
export function matchingAttacks(attacks: AttackOption[], selection: Selection): AttackOption[] {
  return attacks.filter(
    (o) => sameSet(o.attack.attackers, selection.attackers) && sameSet(o.attack.targets, selection.targets),
  );
}

/** Dice worth highlighting: those of an engine-listed attack that contains the current selection. */
export function selectableDice(attacks: AttackOption[], selection: Selection) {
  const attackers = new Set<DieId>();
  const targets = new Set<DieId>();
  for (const o of attacks) {
    if (contains(o.attack.attackers, selection.attackers) && contains(o.attack.targets, selection.targets)) {
      o.attack.attackers.forEach((id) => attackers.add(id));
      o.attack.targets.forEach((id) => targets.add(id));
    }
  }
  return { attackers, targets };
}

export function toggle(list: DieId[], id: DieId): DieId[] {
  return list.includes(id) ? list.filter((x) => x !== id) : [...list, id];
}
