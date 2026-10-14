import { describe, expect, it } from "vitest";
import type { CatalogEntry } from "../../engine/generated/CatalogEntry";
import type { Edition } from "../../engine/generated/Edition";
import { categoriesOf, defaultCategory, editionsOf, filterCatalog } from "../catalogFilter";

const entry = (name: string, category: CatalogEntry["category"], set: string): CatalogEntry => ({
  name,
  recipe: "(6)",
  set,
  category,
  source: category === "Original" ? "Original" : "Buttonweavers",
  playability: { type: "Playable" },
});

const entries = [
  entry("Avis", "Official", "Soldiers"),
  entry("Ricky", "Official", "Uptown"),
  entry("Écho", "Original", "Originaux"),
  entry("Avoine", "Original", "Originaux"),
];

const edition = (name: string, category: Edition["category"]): Edition => ({ name, category, playable: 1, total: 1 });
const editions = [edition("Soldiers", "Official"), edition("Uptown", "Official"), edition("Originaux", "Original")];

describe("filterCatalog", () => {
  it("filters by category and edition", () => {
    const names = (category: "all" | "Official" | "Original", ed: string | null) =>
      filterCatalog(entries, { query: "", category, edition: ed }).map((e) => e.name);
    expect(names("Original", null)).toEqual(["Écho", "Avoine"]);
    expect(names("Official", "Uptown")).toEqual(["Ricky"]);
    expect(names("all", null)).toHaveLength(4);
  });

  it("searches names without case or accents", () => {
    const search = (query: string) =>
      filterCatalog(entries, { query, category: "all", edition: null }).map((e) => e.name);
    expect(search("AV")).toEqual(["Avis", "Avoine"]);
    expect(search("echo")).toEqual(["Écho"]);
  });
});

describe("editions", () => {
  it("prefers the official editions when they are present", () => {
    expect(categoriesOf(editions)).toEqual(["Official", "Original"]);
    expect(defaultCategory(editions)).toBe("Official");
    expect(defaultCategory([edition("Originaux", "Original")])).toBe("Original");
  });

  it("lists the editions of a category", () => {
    expect(editionsOf(editions, "Official").map((e) => e.name)).toEqual(["Soldiers", "Uptown"]);
    expect(editionsOf(editions, "all")).toHaveLength(3);
  });
});
