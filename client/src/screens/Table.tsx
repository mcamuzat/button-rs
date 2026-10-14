import { useEffect, useState } from "react";
import type { GameAdapter } from "../adapters/GameAdapter";
import { useGame } from "../adapters/useGame";
import type { Action } from "../engine/generated/Action";
import type { DieId } from "../engine/generated/DieId";
import { ActionArea } from "../table/ActionArea";
import { Handoff } from "../table/Handoff";
import { MatchEnd } from "../table/MatchEnd";
import { PlayerArea } from "../table/PlayerArea";
import { extraTurnText } from "../table/labels";
import { selectableDice, toggle, type Selection } from "../table/selection";

interface Props {
  adapter: GameAdapter;
  onLeave: () => void;
}

const EMPTY: Selection = { attackers: [], targets: [] };

export function Table({ adapter, onLeave }: Props) {
  const { view, version, thinking, failure, lastEvents } = useGame(adapter);
  const [selection, setSelection] = useState<Selection>(EMPTY);
  const [error, setError] = useState<string | null>(null);
  const seat = view.viewer.type === "Seat" ? view.viewer.player : null;

  // A new view (after any action) starts a fresh selection.
  useEffect(() => setSelection(EMPTY), [view]);

  function act(action: Action) {
    if (!seat) return;
    setError(adapter.dispatch(seat, action));
  }

  // Hotseat: before a secret choice, the screen waits for the right player.
  const [readySeat, setReadySeat] = useState<string | null>(null);
  const secret = view.legal.type === "SpecifyDice";
  const needsHandoff = adapter.handoff && secret && seat !== null && readySeat !== `${view.round}-${seat}`;

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
        label={adapter.names[p.seat]}
        acting={view.actors.includes(p.seat)}
        selectable={mine ? selectable.attackers : selectable.targets}
        selected={mine ? selection.attackers : selection.targets}
        onDieClick={attacks.length > 0 ? onDieClick(mine ? "mine" : "theirs") : undefined}
      />
    );
  };

  return (
    <main className="mx-auto flex min-h-screen max-w-5xl flex-col gap-4 p-4">
      <header className="flex flex-wrap items-center gap-x-4 gap-y-2">
        <h1 className="font-display text-2xl">Manche {view.round}</h1>
        <span className="text-sm text-muted">Premier à {view.max_wins} manches</span>
        <div className="ml-auto flex gap-2">
          {seat && view.legal.type !== "GameOver" && (
            <button type="button" onClick={() => act({ type: "Surrender" })} className="btn-secondary border-danger py-1 text-danger">
              Abandonner
            </button>
          )}
          <button type="button" onClick={onLeave} className="btn-secondary py-1">
            Quitter
          </button>
        </div>
      </header>
      {area(two)}
      {extraTurnText(lastEvents, adapter.names, adapter.mySeat) && (
        <p role="status" className="border-2 border-ink bg-accent px-4 py-2 font-display text-white">
          {extraTurnText(lastEvents, adapter.names, adapter.mySeat)}
        </p>
      )}
      <ActionArea
        key={version}
        view={view}
        act={act}
        selection={selection}
        onClearSelection={() => setSelection(EMPTY)}
        names={adapter.names}
        mySeat={adapter.mySeat}
        thinking={thinking}
      />
      {area(one)}
      {error && (
        <p role="alert" className="border-2 border-ink bg-danger px-4 py-2 font-extrabold text-white">
          {error}
        </p>
      )}
      {failure && (
        <p role="alert" className="border-2 border-ink bg-danger px-4 py-2 font-extrabold text-white">
          L'IA n'a pas pu jouer : {failure}
        </p>
      )}
      {needsHandoff && seat && <Handoff name={adapter.names[seat]} onReady={() => setReadySeat(`${view.round}-${seat}`)} />}
      <MatchEnd view={view} names={adapter.names} mySeat={adapter.mySeat} onLeave={onLeave} />
    </main>
  );
}
