import { describe, expect, it } from "vitest";
import type { DieView } from "../../engine/generated/DieView";
import type { SkillInfo } from "../../engine/generated/SkillInfo";
import { attackLabel, extraTurnText, skillTitle, turnText, winnerText } from "../labels";

describe("labels", () => {
  it("names attack kinds", () => {
    expect(attackLabel("Power")).toBe("Power");
    expect(attackLabel("Skill")).toBe("Skill");
  });

  it("announces an extra turn from the engine's events", () => {
    const names = { One: "Vous", Two: "IA (Normal)" } as const;
    expect(extraTurnText([{ type: "ExtraTurn", player: "One" }], names, "One")).toBe("Vous rejouez (TimeAndSpace)");
    expect(extraTurnText([{ type: "ExtraTurn", player: "Two" }], names, "One")).toBe(
      "IA (Normal) rejoue (TimeAndSpace)",
    );
    expect(extraTurnText([{ type: "Passed", player: "Two" }], names, "One")).toBeNull();
  });

  it("keeps the names of the new attack kinds", () => {
    expect(attackLabel("Rush")).toBe("Rush");
    expect(attackLabel("Boom")).toBe("Boom");
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

  it("speaks to the local player in the second person", () => {
    const names = { One: "Vous", Two: "IA (Normal)" } as const;
    expect(winnerText(names, "One", "One")).toBe("Vous gagnez");
    expect(winnerText(names, "One", "Two")).toBe("IA (Normal) gagne");
    expect(turnText(names, "One", "One")).toBe("À vous de jouer");
    const hotseat = { One: "Joueur 1", Two: "Joueur 2" } as const;
    expect(winnerText(hotseat, null, "Two")).toBe("Joueur 2 gagne");
    expect(turnText(hotseat, null, "Two")).toBe("À Joueur 2");
  });
});
