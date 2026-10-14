import { useState } from "react";
import type { Action } from "../engine/generated/Action";
import type { DieId } from "../engine/generated/DieId";
import type { FireCapacity } from "../engine/generated/FireCapacity";
import { fireAction } from "./fireTotals";

interface Props {
  fire: FireCapacity[];
  boosts: FireCapacity[];
  notation: (id: DieId) => string;
  act: (action: Action) => void;
}

function Amounts({ title, list, values, onChange, notation }: {
  title: string;
  list: FireCapacity[];
  values: Record<DieId, number>;
  onChange: (die: DieId, n: number) => void;
  notation: (id: DieId) => string;
}) {
  return (
    <fieldset className="flex flex-col gap-2">
      <legend className="text-sm text-paper/70">{title}</legend>
      {list.map((c) => (
        <label key={c.die} className="flex items-center gap-3">
          <span className="w-24 font-mono">{notation(c.die)}</span>
          <input
            type="number"
            min={0}
            max={c.max_amount}
            value={values[c.die] ?? 0}
            onChange={(e) => onChange(c.die, Math.max(0, Math.min(c.max_amount, Number(e.target.value))))}
            className="field w-20"
          />
          <span className="text-xs text-paper/60">max {c.max_amount}</span>
        </label>
      ))}
    </fieldset>
  );
}

export function FirePanel({ fire, boosts, notation, act }: Props) {
  const [taken, setTaken] = useState<Record<DieId, number>>({});
  const [added, setAdded] = useState<Record<DieId, number>>({});
  const action = fireAction(taken, added);
  return (
    <div className="flex flex-col gap-3">
      <p>Cette attaque demande l'aide de vos dés Fire : retirez des points aux dés Fire, ajoutez-les aux attaquants.</p>
      <div className="flex flex-wrap gap-8">
        <Amounts title="Retirer (Fire)" list={fire} values={taken} onChange={(d, n) => setTaken({ ...taken, [d]: n })} notation={notation} />
        <Amounts title="Ajouter (attaquants)" list={boosts} values={added} onChange={(d, n) => setAdded({ ...added, [d]: n })} notation={notation} />
      </div>
      <div className="flex gap-2">
        <button
          type="button"
          disabled={!action}
          onClick={() => action && act(action)}
          className="btn-primary"
        >
          Attaquer
        </button>
        <button type="button" onClick={() => act({ type: "CancelFire" })} className="btn-secondary">
          Annuler l'attaque
        </button>
      </div>
    </div>
  );
}
