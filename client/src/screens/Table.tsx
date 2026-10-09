import type { GameAdapter } from "../adapters/GameAdapter";
import { useGame } from "../adapters/useGame";

interface Props {
  adapter: GameAdapter;
  onLeave: () => void;
}

export function Table({ adapter, onLeave }: Props) {
  const { view } = useGame(adapter);
  return (
    <main className="p-6">
      <p>Manche {view.round} — en attente : {view.waiting_for.type}</p>
      <button type="button" onClick={onLeave}>Quitter</button>
    </main>
  );
}
