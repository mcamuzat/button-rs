import type { Action } from "../engine/generated/Action";
import type { DieId } from "../engine/generated/DieId";

interface Props {
  dice: DieId[];
  notation: (id: DieId) => string;
  act: (action: Action) => void;
}

export function ReservePanel({ dice, notation, act }: Props) {
  return (
    <div className="flex flex-wrap items-center gap-2">
      <span>Vous avez perdu la manche : ajouter un dé de réserve ?</span>
      {dice.map((die) => (
        <button
          key={die}
          type="button"
          onClick={() => act({ type: "ChooseReserve", die })}
          className="rounded-lg bg-gold px-3 py-1 font-mono text-ink"
        >
          {notation(die)}
        </button>
      ))}
      <button type="button" onClick={() => act({ type: "ChooseReserve", die: null })} className="rounded-lg bg-felt px-3 py-1">
        Non merci
      </button>
    </div>
  );
}
