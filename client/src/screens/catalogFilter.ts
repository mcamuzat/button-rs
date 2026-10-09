import type { ButtonSource } from "../engine/generated/ButtonSource";
import type { CatalogEntry } from "../engine/generated/CatalogEntry";

export type SourceFilter = "all" | ButtonSource;

const fold = (s: string) => s.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase();

/** Search box and source tabs of the button picker (display filtering only). */
export function filterCatalog(entries: CatalogEntry[], query: string, source: SourceFilter): CatalogEntry[] {
  const q = fold(query.trim());
  return entries.filter((e) => (source === "all" || e.source === source) && fold(e.name).includes(q));
}
