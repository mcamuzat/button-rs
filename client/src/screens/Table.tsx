import { useEffect, useState } from "react";
import type { GameAdapter } from "../adapters/GameAdapter";
import { useGame } from "../adapters/useGame";
import type { Action } from "../engine/generated/Action";
import type { DieId } from "../engine/generated/DieId";
import { ActionArea } from "../table/ActionArea";
import { PlayerArea } from "../table/PlayerArea";
import { seatLabel } from "../table/seats";
import { selectableDice, toggle, type Selection } from "../table/selection";

interface Props {
  adapter: GameAdapter;
  onLeave: () => void;
}

const EMPTY: Selection = { attackers: [], targets: [] };

export function Table({ adapter, onLeave }: Props) {
  const { view } = useGame(adapter);
  const [selection, setSelection] = useState<Selection>(EMPTY);
  const [error, setError] = useState<string | null>(null);
  const seat = view.viewer.type === "Seat" ? view.viewer.player : null;

  // A new view (after any action) starts a fresh selection.
  useEffect(() => setSelection(EMPTY), [view]);

  function act(action: Action) {
    if (!seat) return;
    setError(adapter.dispatch(seat, action));
  }

  const attacks = view.legal.type === "Attack" ? view.legal.attacks : [];
  const selectable = selectableDice(attacks, selection);

  function onDieClick(owner: "mine" | "theirs") {
    return (id: DieId) =>
      setSelection((s) =>
        owner === "mine" ? { ...s, attackers: toggle(s.attackers, id) } : { ...s, targets: toggle(s.targets, id) },
      );
  }

  // Fixed layout: player 2 at the top, player 1 at the bottom.
  const [one, two] = view.players;
  const area = (p: typeof one) => {
    const mine = p.seat === seat;
    return (
      <PlayerArea
        player={p}
        label={seatLabel(p.seat)}
        acting={view.actors.includes(p.seat)}
        selectable={mine ? selectable.attackers : selectable.targets}
        selected={mine ? selection.attackers : selection.targets}
        onDieClick={attacks.length > 0 ? onDieClick(mine ? "mine" : "theirs") : undefined}
      />
    );
  };

  return (
    <main className="mx-auto flex min-h-screen max-w-5xl flex-col gap-4 p-4">
      <header className="flex items-center gap-4">
        <h1 className="font-display text-2xl">Manche {view.round}</h1>
        <span className="text-sm text-ivory/70">Premier à {view.max_wins} manches</span>
        <div className="ml-auto flex gap-2">
          {seat && view.legal.type !== "GameOver" && (
            <button type="button" onClick={() => act({ type: "Surrender" })} className="rounded-lg bg-danger/80 px-3 py-1">
              Abandonner
            </button>
          )}
          <button type="button" onClick={onLeave} className="rounded-lg bg-felt-dark px-3 py-1">
            Quitter
          </button>
        </div>
      </header>
      {area(two)}
      <ActionArea view={view} act={act} selection={selection} onClearSelection={() => setSelection(EMPTY)} />
      {area(one)}
      {error && (
        <p role="alert" className="rounded-lg bg-danger/90 px-4 py-2">
          {error}
        </p>
      )}
    </main>
  );
}
