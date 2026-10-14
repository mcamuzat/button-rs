import type { Action } from "../engine/generated/Action";
import type { GameView } from "../engine/generated/GameView";
import type { PlayerId } from "../engine/generated/PlayerId";
import { AttackPanel } from "./AttackPanel";
import { turnText } from "./labels";
import { FirePanel } from "./FirePanel";
import { InitiativePanel } from "./InitiativePanel";
import { ReservePanel } from "./ReservePanel";
import type { Selection } from "./selection";
import { SpecifyDicePanel } from "./SpecifyDicePanel";

interface Props {
  view: GameView;
  act: (action: Action) => void;
  selection: Selection;
  onClearSelection: () => void;
  names: Record<PlayerId, string>;
  mySeat: PlayerId | null;
  thinking: boolean;
}

/** The decision the acting player faces, chosen from `view.legal` (engine-provided). */
export function ActionArea({ view, act, selection, onClearSelection, names, mySeat, thinking }: Props) {
  const legal = view.legal;
  const who = view.viewer.type === "Seat" ? turnText(names, mySeat, view.viewer.player) : "";
  // Recipe notation of any die id, from the engine-provided views.
  const notation = (id: number) => {
    for (const p of view.players) {
      const die = [...p.active, ...p.captured.map((c) => c.die)].find((d) => d.id === id);
      if (die) return die.notation;
      const slot = p.reserve.find((r) => r.id === id);
      if (slot) return slot.notation;
    }
    return `#${id}`;
  };
  let body;
  switch (legal.type) {
    case "Attack":
      body = (
        <AttackPanel
          attacks={legal.attacks}
          canPass={legal.can_pass}
          selection={selection}
          act={act}
          onClear={onClearSelection}
        />
      );
      break;
    case "SpecifyDice":
      body = <SpecifyDicePanel swing={legal.swing} options={legal.options} act={act} />;
      break;
    case "ChooseReserve":
      body = <ReservePanel dice={legal.dice} notation={notation} act={act} />;
      break;
    case "ReactToInitiative":
      body = <InitiativePanel focus={legal.focus} chance={legal.chance} notation={notation} act={act} />;
      break;
    case "AdjustFire":
      body = <FirePanel fire={legal.fire} boosts={legal.boosts} notation={notation} act={act} />;
      break;
    case "Wait":
      body = thinking ? <span className="animate-pulse">L'IA réfléchit…</span> : null;
      break;
    case "GameOver":
      body = null;
      break;
  }
  return (
    <section className="on-ink bg-ink p-4 text-paper">
      {who && legal.type !== "Wait" && legal.type !== "GameOver" && (
        <p className="mb-2 font-display text-lg">{who}</p>
      )}
      {body}
    </section>
  );
}
