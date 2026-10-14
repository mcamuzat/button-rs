import type { GameView } from "../engine/generated/GameView";
import type { PlayerId } from "../engine/generated/PlayerId";
import { winnerText } from "./labels";

export function MatchEnd({
  view,
  names,
  mySeat,
  onLeave,
}: {
  view: GameView;
  names: Record<PlayerId, string>;
  mySeat: PlayerId | null;
  onLeave: () => void;
}) {
  if (view.waiting_for.type !== "GameOver") return null;
  const { winner, reason } = view.waiting_for.result;
  const champion = view.players.find((p) => p.seat === winner);
  const [one, two] = view.players;
  return (
    <div className="fixed inset-0 z-10 flex flex-col items-center justify-center gap-4 bg-paper/95">
      <p className="font-display text-4xl">
        {winnerText(names, mySeat, winner)} avec {champion?.button}
      </p>
      <p className="text-muted">
        {reason === "Surrender" ? "Par abandon." : `${one.record.wins} manche(s) à ${two.record.wins}.`}
      </p>
      <button type="button" onClick={onLeave} className="btn-primary px-6 py-3">
        Retour à l'accueil
      </button>
    </div>
  );
}
