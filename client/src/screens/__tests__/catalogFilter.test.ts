import { describe, expect, it } from "vitest";
import type { CatalogEntry } from "../../engine/generated/CatalogEntry";
import { filterCatalog } from "../catalogFilter";

const entry = (name: string, source: CatalogEntry["source"]): CatalogEntry => ({
  name,
  recipe: "(6)",
  set: null,
  source,
  playability: { type: "Playable" },
});

const entries = [entry("Écho", "Original"), entry("Avis", "Buttonweavers"), entry("Avoine", "Original")];

describe("filterCatalog", () => {
  it("filters by source", () => {
    expect(filterCatalog(entries, "", "Original").map((e) => e.name)).toEqual(["Écho", "Avoine"]);
    expect(filterCatalog(entries, "", "all")).toHaveLength(3);
  });

  it("searches names without case or accents", () => {
    expect(filterCatalog(entries, "AV", "all").map((e) => e.name)).toEqual(["Avis", "Avoine"]);
    expect(filterCatalog(entries, "echo", "all").map((e) => e.name)).toEqual(["Écho"]);
  });
});
