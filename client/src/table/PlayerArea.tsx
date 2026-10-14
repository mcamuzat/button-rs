import type { DieId } from "../engine/generated/DieId";
import type { PlayerView } from "../engine/generated/PlayerView";
import { formatPoints } from "../engine/format";
import { DieChip, type ChipState } from "./DieChip";

interface Props {
  player: PlayerView;
  label: string;
  acting: boolean;
  selectable: Set<DieId>;
  selected: DieId[];
  onDieClick?: (id: DieId) => void;
}

export function PlayerArea({ player, label, acting, selectable, selected, onDieClick }: Props) {
  const chipState = (id: DieId): ChipState =>
    selected.includes(id) ? "selected" : selectable.has(id) ? "selectable" : "idle";
  return (
    <section className={`print-card p-4 ${acting ? "border-accent shadow-[4px_4px_0_var(--color-accent)]" : ""}`}>
      <header className="mb-3 flex flex-wrap items-baseline gap-x-4 gap-y-1">
        <span className="font-display text-xl">{player.button}</span>
        <span className="text-xs font-extrabold tracking-wider uppercase">{label}</span>
        <span className="font-mono text-xs text-muted">{player.recipe}</span>
        <span className="ml-auto text-sm">
          Manches : <strong>{player.record.wins}</strong> · Score : <strong>{formatPoints(player.round_score)}</strong>
        </span>
      </header>
      <div className="flex min-h-16 flex-wrap gap-3">
        {player.active.map((die) => (
          <DieChip
            key={die.id}
            die={die}
            state={chipState(die.id)}
            onClick={onDieClick && (selectable.has(die.id) || selected.includes(die.id)) ? () => onDieClick(die.id) : undefined}
          />
        ))}
      </div>
      {(player.captured.length > 0 || player.reserve.length > 0) && (
        <div className="mt-3 flex flex-wrap items-center gap-2 text-sm text-muted">
          {player.captured.length > 0 && <span>Capturés :</span>}
          {player.captured.map((c) => (
            <DieChip key={c.die.id} die={c.die} state="idle" small />
          ))}
          {player.reserve.length > 0 && (
            <span className="ml-4">Réserve : {player.reserve.map((r) => r.notation).join(" ")}</span>
          )}
        </div>
      )}
    </section>
  );
}
