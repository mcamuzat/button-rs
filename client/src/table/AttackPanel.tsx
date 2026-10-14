import type { Action } from "../engine/generated/Action";
import type { AttackOption } from "../engine/generated/AttackOption";
import { attackLabel } from "./labels";
import { matchingAttacks, type Selection } from "./selection";

interface Props {
  attacks: AttackOption[];
  canPass: boolean;
  selection: Selection;
  act: (action: Action) => void;
  onClear: () => void;
}

export function AttackPanel({ attacks, canPass, selection, act, onClear }: Props) {
  const matching = matchingAttacks(attacks, selection);
  const empty = selection.attackers.length === 0 && selection.targets.length === 0;
  return (
    <div className="flex flex-wrap items-center gap-2">
      {canPass ? (
        <>
          <span>Aucune attaque possible.</span>
          <button type="button" onClick={() => act({ type: "Pass" })} className="btn-primary">
            Passer
          </button>
        </>
      ) : empty ? (
        <span>Choisissez vos dés attaquants puis la ou les cibles (dés surlignés).</span>
      ) : matching.length === 0 ? (
        <span>Complétez la sélection…</span>
      ) : (
        matching.map((o) => (
          <button
            key={o.attack.kind}
            type="button"
            onClick={() => act({ type: "Attack", attack: o.attack })}
            className="btn-primary"
          >
            {attackLabel(o.attack.kind)}
            {o.needs_fire ? " (avec Fire)" : ""}
          </button>
        ))
      )}
      {!empty && (
        <button type="button" onClick={onClear} className="btn-secondary">
          Effacer
        </button>
      )}
    </div>
  );
}
