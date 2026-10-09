import type { Action } from "../engine/generated/Action";
import type { GameView } from "../engine/generated/GameView";
import { AttackPanel } from "./AttackPanel";
import { seatLabel } from "./seats";
import type { Selection } from "./selection";

interface Props {
  view: GameView;
  act: (action: Action) => void;
  selection: Selection;
  onClearSelection: () => void;
}

/** The decision the acting player faces, chosen from `view.legal` (engine-provided). */
export function ActionArea({ view, act, selection, onClearSelection }: Props) {
  const legal = view.legal;
  const who = view.viewer.type === "Seat" ? seatLabel(view.viewer.player) : "";
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
    case "ChooseReserve":
    case "ReactToInitiative":
    case "AdjustFire":
      // Panels added in Task 18.
      body = <span>{legal.type}</span>;
      break;
    case "Wait":
    case "GameOver":
      body = null;
      break;
  }
  return (
    <section className="rounded-2xl bg-ink/30 p-4">
      {who && <p className="mb-2 font-display text-lg">À {who}</p>}
      {body}
    </section>
  );
}
