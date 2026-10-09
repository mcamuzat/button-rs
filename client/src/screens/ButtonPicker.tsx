import { useMemo, useState } from "react";
import type { CatalogEntry } from "../engine/generated/CatalogEntry";
import { filterCatalog, type SourceFilter } from "./catalogFilter";

interface Props {
  catalog: CatalogEntry[];
  onStart: (one: CatalogEntry, two: CatalogEntry) => void;
  onBack: () => void;
}

const SOURCES: { value: SourceFilter; label: string }[] = [
  { value: "Original", label: "Originaux" },
  { value: "Buttonweavers", label: "buttonweavers" },
  { value: "all", label: "Tous" },
];

export function ButtonPicker({ catalog, onStart, onBack }: Props) {
  const [query, setQuery] = useState("");
  const [source, setSource] = useState<SourceFilter>("Original");
  const [picks, setPicks] = useState<[CatalogEntry | null, CatalogEntry | null]>([null, null]);
  const [seat, setSeat] = useState<0 | 1>(0);
  const hasImport = catalog.some((e) => e.source === "Buttonweavers");
  const shown = useMemo(() => filterCatalog(catalog, query, source), [catalog, query, source]);

  function pick(entry: CatalogEntry) {
    const next: [CatalogEntry | null, CatalogEntry | null] = [...picks];
    next[seat] = entry;
    setPicks(next);
    if (seat === 0 && !next[1]) setSeat(1);
  }

  function pickRandom() {
    const playable = shown.filter((e) => e.playability.type === "Playable");
    if (playable.length > 0) pick(playable[Math.floor(Math.random() * playable.length)]);
  }

  return (
    <main className="mx-auto flex max-w-5xl flex-col gap-4 p-6">
      <div className="flex items-center justify-between">
        <h1 className="font-display text-3xl">Choix des boutons</h1>
        <button type="button" onClick={onBack} className="text-ivory/70 hover:text-ivory">
          ← Accueil
        </button>
      </div>

      <div className="grid gap-3 sm:grid-cols-2">
        {([0, 1] as const).map((i) => (
          <button
            key={i}
            type="button"
            onClick={() => setSeat(i)}
            className={`rounded-xl p-4 text-left ${seat === i ? "bg-gold text-ink" : "bg-felt-dark"}`}
          >
            <span className="text-sm opacity-70">Joueur {i + 1}</span>
            <span className="block font-display text-xl">{picks[i]?.name ?? "—"}</span>
            <span className="block font-mono text-sm">{picks[i]?.recipe ?? ""}</span>
          </button>
        ))}
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Rechercher un bouton"
          className="min-w-48 flex-1 rounded-lg bg-ivory px-3 py-2 text-ink"
        />
        {hasImport &&
          SOURCES.map((s) => (
            <button
              key={s.value}
              type="button"
              onClick={() => setSource(s.value)}
              className={`rounded-lg px-3 py-2 ${source === s.value ? "bg-gold text-ink" : "bg-felt-dark"}`}
            >
              {s.label}
            </button>
          ))}
        <button type="button" onClick={pickRandom} className="rounded-lg bg-felt-dark px-3 py-2">
          Au hasard
        </button>
        <button
          type="button"
          disabled={!picks[0] || !picks[1]}
          onClick={() => picks[0] && picks[1] && onStart(picks[0], picks[1])}
          className="rounded-lg bg-gold px-4 py-2 font-semibold text-ink disabled:opacity-40"
        >
          Jouer
        </button>
      </div>

      <ul className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
        {shown.map((e) => {
          const playable = e.playability.type === "Playable";
          return (
            <li key={`${e.source}-${e.name}`}>
              <button
                type="button"
                disabled={!playable}
                onClick={() => pick(e)}
                title={e.playability.type === "Unplayable" ? e.playability.reason : undefined}
                className="w-full rounded-lg bg-felt-dark p-3 text-left enabled:hover:ring-2 enabled:hover:ring-gold disabled:opacity-40"
              >
                <span className="font-display">{e.name}</span>
                {e.set && <span className="ml-2 text-xs text-ivory/60">{e.set}</span>}
                <span className="block font-mono text-sm text-ivory/80">{e.recipe}</span>
                {e.playability.type === "Unplayable" && (
                  <span className="block text-xs text-danger">{e.playability.reason}</span>
                )}
              </button>
            </li>
          );
        })}
      </ul>
    </main>
  );
}
