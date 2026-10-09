import type { GameView } from "../engine/generated/GameView";
import { seatLabel } from "./seats";

export function MatchEnd({ view, onLeave }: { view: GameView; onLeave: () => void }) {
  if (view.waiting_for.type !== "GameOver") return null;
  const { winner, reason } = view.waiting_for.result;
  const champion = view.players.find((p) => p.seat === winner);
  const [one, two] = view.players;
  return (
    <div className="fixed inset-0 z-10 flex flex-col items-center justify-center gap-4 bg-felt-dark/95">
      <p className="font-display text-4xl">
        {seatLabel(winner)} gagne avec {champion?.button}
      </p>
      <p className="text-ivory/80">
        {reason === "Surrender" ? "Par abandon." : `${one.record.wins} manche(s) à ${two.record.wins}.`}
      </p>
      <button type="button" onClick={onLeave} className="rounded-lg bg-gold px-6 py-3 font-semibold text-ink">
        Retour à l'accueil
      </button>
    </div>
  );
}
