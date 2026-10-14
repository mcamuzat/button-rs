import type { CatalogEntry } from "../engine/generated/CatalogEntry";
import type { Edition } from "../engine/generated/Edition";
import type { SetCategory } from "../engine/generated/SetCategory";

export type CategoryFilter = "all" | SetCategory;

export interface CatalogFilter {
  query: string;
  category: CategoryFilter;
  /** `null`: every edition of the category. */
  edition: string | null;
}

const fold = (s: string) => s.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase();

/** Search box, category tabs and edition menu of the button picker (display filtering only). */
export function filterCatalog(entries: CatalogEntry[], { query, category, edition }: CatalogFilter): CatalogEntry[] {
  const q = fold(query.trim());
  return entries.filter(
    (e) =>
      (category === "all" || e.category === category) &&
      (edition === null || e.set === edition) &&
      fold(e.name).includes(q),
  );
}

/** The categories present in the catalog, in the engine's order. */
export function categoriesOf(editions: Edition[]): SetCategory[] {
  return [...new Set(editions.map((e) => e.category))];
}

/** Official editions first when the import is there, otherwise the originals. */
export function defaultCategory(editions: Edition[]): CategoryFilter {
  return categoriesOf(editions)[0] ?? "all";
}

export function editionsOf(editions: Edition[], category: CategoryFilter): Edition[] {
  return category === "all" ? editions : editions.filter((e) => e.category === category);
}
