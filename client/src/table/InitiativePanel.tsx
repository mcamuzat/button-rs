import { useState } from "react";
import type { Action } from "../engine/generated/Action";
import type { DieId } from "../engine/generated/DieId";
import type { FocusRange } from "../engine/generated/FocusRange";

interface Props {
  focus: FocusRange[];
  chance: DieId[];
  notation: (id: DieId) => string;
  act: (action: Action) => void;
}

export function InitiativePanel({ focus, chance, notation, act }: Props) {
  const [lowered, setLowered] = useState<Record<DieId, number>>(() =>
    Object.fromEntries(focus.map((f) => [f.die, f.current])),
  );
  const changed = focus.filter((f) => lowered[f.die] < f.current).map((f) => [f.die, lowered[f.die]] as [DieId, number]);

  return (
    <div className="flex flex-col gap-3">
      <p>Votre adversaire a l'initiative. Voulez-vous réagir ?</p>
      {focus.map((f) => (
        <label key={f.die} className="flex items-center gap-3">
          <span className="w-24 font-mono">{notation(f.die)}</span>
          <select
            value={lowered[f.die]}
            onChange={(e) => setLowered({ ...lowered, [f.die]: Number(e.target.value) })}
            className="field"
          >
            {Array.from({ length: f.current - f.min + 1 }, (_, k) => f.current - k).map((v) => (
              <option key={v} value={v}>
                {v}
              </option>
            ))}
          </select>
        </label>
      ))}
      <div className="flex flex-wrap gap-2">
        {focus.length > 0 && (
          <button
            type="button"
            disabled={changed.length === 0}
            onClick={() => act({ type: "FocusTurnDown", dice: changed })}
            className="btn-primary"
          >
            Baisser (Focus)
          </button>
        )}
        {chance.map((die) => (
          <button
            key={die}
            type="button"
            onClick={() => act({ type: "ChanceReroll", die })}
            className="btn-primary"
          >
            Relancer {notation(die)} (Chance)
          </button>
        ))}
        <button type="button" onClick={() => act({ type: "PassInitiative" })} className="btn-secondary">
          Laisser commencer l'adversaire
        </button>
      </div>
    </div>
  );
}
