import { useEffect, useState } from "react";
import type { GameAdapter } from "./adapters/GameAdapter";
import { LocalAdapter } from "./adapters/LocalAdapter";
import type { CatalogEntry } from "./engine/generated/CatalogEntry";
import type { DieSkill } from "./engine/generated/DieSkill";
import type { SkillInfo } from "./engine/generated/SkillInfo";
import { createMatch, loadCatalog, loadSkills } from "./engine/wasm";
import { ButtonPicker } from "./screens/ButtonPicker";
import { Home } from "./screens/Home";
import { Table } from "./screens/Table";
import { SkillsContext } from "./table/SkillsContext";

type Screen = { kind: "home" } | { kind: "pick" } | { kind: "table"; adapter: GameAdapter };

export function App() {
  const [screen, setScreen] = useState<Screen>({ kind: "home" });
  const [catalog, setCatalog] = useState<CatalogEntry[] | null>(null);
  const [skills, setSkills] = useState<Map<DieSkill, SkillInfo>>(new Map());
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([loadCatalog(), loadSkills()])
      .then(([c, s]) => {
        setCatalog(c);
        setSkills(s);
      })
      .catch((e: unknown) => setError(String(e)));
  }, []);

  function start(one: CatalogEntry, two: CatalogEntry) {
    const [seed] = crypto.getRandomValues(new Uint32Array(1));
    const engine = createMatch({
      buttons: [
        { name: one.name, recipe: one.recipe },
        { name: two.name, recipe: two.recipe },
      ],
      seed,
      max_wins: 3,
    });
    setScreen({ kind: "table", adapter: new LocalAdapter(engine) });
  }

  function leave() {
    if (screen.kind === "table") screen.adapter.dispose();
    setScreen({ kind: "home" });
  }

  if (error) return <p className="p-8 text-danger">Le moteur n'a pas pu démarrer : {error}</p>;
  if (!catalog) return <p className="p-8">Chargement…</p>;

  return (
    <SkillsContext.Provider value={skills}>
      {screen.kind === "home" && <Home onHotseat={() => setScreen({ kind: "pick" })} />}
      {screen.kind === "pick" && (
        <ButtonPicker catalog={catalog} onStart={start} onBack={() => setScreen({ kind: "home" })} />
      )}
      {screen.kind === "table" && <Table adapter={screen.adapter} onLeave={leave} />}
    </SkillsContext.Provider>
  );
}
