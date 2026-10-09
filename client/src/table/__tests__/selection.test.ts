import { describe, expect, it } from "vitest";
import type { AttackOption } from "../../engine/generated/AttackOption";
import { matchingAttacks, selectableDice, toggle } from "../selection";

const option = (kind: AttackOption["attack"]["kind"], attackers: number[], targets: number[]): AttackOption => ({
  attack: { kind, attackers, targets },
  needs_fire: false,
});

const attacks = [option("Power", [0], [5]), option("Skill", [0, 1], [6]), option("Skill", [0], [5])];

describe("selection", () => {
  it("matches attacks with exactly the selected dice, in any order", () => {
    expect(matchingAttacks(attacks, { attackers: [0], targets: [5] }).map((o) => o.attack.kind)).toEqual([
      "Power",
      "Skill",
    ]);
    expect(matchingAttacks(attacks, { attackers: [1, 0], targets: [6] })).toHaveLength(1);
    expect(matchingAttacks(attacks, { attackers: [1], targets: [6] })).toHaveLength(0);
  });

  it("offers only dice that extend the current selection to some attack", () => {
    const none = selectableDice(attacks, { attackers: [], targets: [] });
    expect([...none.attackers].sort()).toEqual([0, 1]);
    expect([...none.targets].sort()).toEqual([5, 6]);
    const afterOne = selectableDice(attacks, { attackers: [1], targets: [] });
    expect([...afterOne.attackers]).toEqual([0, 1]);
    expect([...afterOne.targets]).toEqual([6]);
  });

  it("toggles ids", () => {
    expect(toggle([1, 2], 2)).toEqual([1]);
    expect(toggle([1], 3)).toEqual([1, 3]);
  });
});
