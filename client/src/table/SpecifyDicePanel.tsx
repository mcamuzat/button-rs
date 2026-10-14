import { useState } from "react";
import type { Action } from "../engine/generated/Action";
import type { OptionRequest } from "../engine/generated/OptionRequest";
import type { SwingRequest } from "../engine/generated/SwingRequest";

interface Props {
  swing: SwingRequest[];
  options: OptionRequest[];
  act: (action: Action) => void;
}

export function SpecifyDicePanel({ swing, options, act }: Props) {
  const [sizes, setSizes] = useState(() => swing.map((r) => r.current ?? r.min));
  const [picks, setPicks] = useState(() => options.map((r) => r.current ?? r.a));

  function submit() {
    act({
      type: "SpecifyDice",
      choice: {
        swing: swing.map((r, i) => [r.letter, sizes[i]]),
        options: options.map((r, i) => [r.die, picks[i]]),
      },
    });
  }

  return (
    <div className="flex flex-col gap-3">
      <p>Choisissez la taille de vos dés (en secret).</p>
      {swing.map((r, i) => (
        <label key={r.letter} className="flex items-center gap-3">
          <span className="w-24 font-mono">Swing {r.letter}</span>
          <input
            type="range"
            min={r.min}
            max={r.max}
            value={sizes[i]}
            onChange={(e) => setSizes(sizes.map((s, j) => (j === i ? Number(e.target.value) : s)))}
            className="flex-1 accent-accent"
          />
          <span className="w-10 text-right font-display text-xl">{sizes[i]}</span>
        </label>
      ))}
      {options.map((r, i) => (
        <div key={r.die} className="flex items-center gap-3">
          {/* Option dice have no die view before the first roll: show their two sizes. */}
          <span className="w-24 font-mono">{`(${r.a}/${r.b})`}</span>
          {[r.a, r.b].map((size) => (
            <button
              key={size}
              type="button"
              onClick={() => setPicks(picks.map((p, j) => (j === i ? size : p)))}
              className={`rounded-lg px-3 py-1 ${picks[i] === size ? "bg-accent text-white" : "border-2 border-paper"}`}
            >
              {size}
            </button>
          ))}
        </div>
      ))}
      <button type="button" onClick={submit} className="btn-primary self-start">
        Valider
      </button>
    </div>
  );
}
