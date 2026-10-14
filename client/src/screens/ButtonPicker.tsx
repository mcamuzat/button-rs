import { useMemo, useState } from "react";
import { levelLabel } from "../adapters/AiAdapter";
import type { Catalog } from "../engine/generated/Catalog";
import type { CatalogEntry } from "../engine/generated/CatalogEntry";
import type { Level } from "../engine/generated/Level";
import type { SetCategory } from "../engine/generated/SetCategory";
import { type CategoryFilter, categoriesOf, defaultCategory, editionsOf, filterCatalog } from "./catalogFilter";

interface Props {
  catalog: Catalog;
  /** Seat 2 is played by the AI. */
  vsAi: boolean;
  onStart: (one: CatalogEntry, two: CatalogEntry, level: Level) => void;
  onBack: () => void;
}

const CATEGORY_LABELS: Record<SetCategory, string> = {
  Official: "Officielles",
  Original: "Originaux",
  Online: "Button Men Online",
  Unofficial: "Fans",
  Fanatics: "Fanatics",
  Special: "Spéciaux",
  Other: "Autres",
};

const LEVELS: Level[] = ["Easy", "Normal", "Hard"];

export function ButtonPicker({ catalog, vsAi, onStart, onBack }: Props) {
  const [level, setLevel] = useState<Level>("Normal");
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<CategoryFilter>(() => defaultCategory(catalog.editions));
  const [edition, setEdition] = useState<string | null>(null);
  const [picks, setPicks] = useState<[CatalogEntry | null, CatalogEntry | null]>([null, null]);
  const [seat, setSeat] = useState<0 | 1>(0);
  const categories = categoriesOf(catalog.editions);
  const editions = editionsOf(catalog.editions, category);
  const shown = useMemo(
    () => filterCatalog(catalog.entries, { query, category, edition }),
    [catalog, query, category, edition],
  );

  function chooseCategory(next: CategoryFilter) {
    setCategory(next);
    setEdition(null);
  }

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
        <button type="button" onClick={onBack} className="font-extrabold text-muted hover:text-ink">
          ← Accueil
        </button>
      </div>

      <div className="grid gap-3 sm:grid-cols-2">
        {([0, 1] as const).map((i) => (
          <button
            key={i}
            type="button"
            onClick={() => setSeat(i)}
            className={`print-card p-4 text-left ${seat === i ? "border-accent shadow-[4px_4px_0_var(--color-accent)]" : ""}`}
          >
            <span className="text-sm opacity-70">{vsAi ? (i === 0 ? "Vous" : "IA") : `Joueur ${i + 1}`}</span>
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
          className="field min-w-48 flex-1 px-3 py-2"
        />
        {vsAi &&
          LEVELS.map((l) => (
            <button
              key={l}
              type="button"
              onClick={() => setLevel(l)}
              className={level === l ? "btn-primary" : "btn-secondary"}
            >
              {levelLabel(l)}
            </button>
          ))}
        <button type="button" onClick={pickRandom} className="btn-secondary">
          Au hasard
        </button>
        <button
          type="button"
          disabled={!picks[0] || !picks[1]}
          onClick={() => picks[0] && picks[1] && onStart(picks[0], picks[1], level)}
          className="btn-primary"
        >
          Jouer
        </button>
      </div>

      {categories.length > 1 && (
        <div className="flex flex-wrap items-center gap-2">
          {[...categories, "all" as const].map((c) => (
            <button
              key={c}
              type="button"
              onClick={() => chooseCategory(c)}
              className={category === c ? "btn-primary" : "btn-secondary"}
            >
              {c === "all" ? "Toutes" : CATEGORY_LABELS[c]}
            </button>
          ))}
          {editions.length > 1 && (
            <select
              value={edition ?? ""}
              onChange={(e) => setEdition(e.target.value === "" ? null : e.target.value)}
              className="field px-3 py-2"
              aria-label="Édition"
            >
              <option value="">Toutes les éditions ({editions.length})</option>
              {editions.map((ed) => (
                <option key={`${ed.category}-${ed.name}`} value={ed.name}>
                  {ed.name} ({ed.playable}/{ed.total})
                </option>
              ))}
            </select>
          )}
        </div>
      )}

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
                className="w-full border-2 border-ink bg-card p-3 text-left transition enabled:hover:-translate-y-0.5 enabled:hover:shadow-[3px_3px_0_var(--color-accent)] disabled:opacity-40"
              >
                <span className="font-display">{e.name}</span>
                {e.set && <span className="ml-2 text-xs text-muted">{e.set}</span>}
                <span className="block font-mono text-sm text-muted">{e.recipe}</span>
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
