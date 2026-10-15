import { useEffect, useState } from "react";
import { AiAdapter } from "./adapters/AiAdapter";
import type { GameAdapter } from "./adapters/GameAdapter";
import { LocalAdapter } from "./adapters/LocalAdapter";
import type { Catalog } from "./engine/generated/Catalog";
import type { CatalogEntry } from "./engine/generated/CatalogEntry";
import { WorkerBrain } from "./ai/brain";
import type { DieSkill } from "./engine/generated/DieSkill";
import type { Level } from "./engine/generated/Level";
import type { SkillInfo } from "./engine/generated/SkillInfo";
import type { SwingRange } from "./engine/generated/SwingRange";
import { createMatch, loadCatalog, loadSkills, loadSwingRanges } from "./engine/wasm";
import { ButtonPicker } from "./screens/ButtonPicker";
import { Home } from "./screens/Home";
import { Rules } from "./screens/Rules";
import { Table } from "./screens/Table";
import { SkillsContext } from "./table/SkillsContext";

type Screen =
  | { kind: "home" }
  | { kind: "rules" }
  | { kind: "pick"; vsAi: boolean }
  | { kind: "table"; adapter: GameAdapter };

export function App() {
  const [screen, setScreen] = useState<Screen>({ kind: "home" });
  const [catalog, setCatalog] = useState<Catalog | null>(null);
  const [skills, setSkills] = useState<Map<DieSkill, SkillInfo>>(new Map());
  const [swingRanges, setSwingRanges] = useState<SwingRange[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([loadCatalog(), loadSkills(), loadSwingRanges()])
      .then(([c, s, r]) => {
        setCatalog(c);
        setSkills(s);
        setSwingRanges(r);
      })
      .catch((e: unknown) => setError(String(e)));
  }, []);

  function start(one: CatalogEntry, two: CatalogEntry, level: Level, vsAi: boolean) {
    const [seed] = crypto.getRandomValues(new Uint32Array(1));
    const engine = createMatch({
      buttons: [
        { name: one.name, recipe: one.recipe },
        { name: two.name, recipe: two.recipe },
      ],
      seed,
      max_wins: 3,
    });
    const adapter = vsAi ? new AiAdapter(engine, new WorkerBrain(), level) : new LocalAdapter(engine);
    setScreen({ kind: "table", adapter });
  }

  function leave() {
    if (screen.kind === "table") screen.adapter.dispose();
    setScreen({ kind: "home" });
  }

  if (error) return <p className="p-8 text-danger">Le moteur n'a pas pu démarrer : {error}</p>;
  if (!catalog) return <p className="p-8">Chargement…</p>;

  return (
    <SkillsContext.Provider value={skills}>
      {screen.kind === "home" && (
        <Home
          onHotseat={() => setScreen({ kind: "pick", vsAi: false })}
          onAi={() => setScreen({ kind: "pick", vsAi: true })}
          onRules={() => setScreen({ kind: "rules" })}
        />
      )}
      {screen.kind === "rules" && <Rules swingRanges={swingRanges} onBack={() => setScreen({ kind: "home" })} />}
      {screen.kind === "pick" && (
        <ButtonPicker
          catalog={catalog}
          vsAi={screen.vsAi}
          onStart={(one, two, level) => start(one, two, level, screen.vsAi)}
          onBack={() => setScreen({ kind: "home" })}
        />
      )}
      {screen.kind === "table" && <Table adapter={screen.adapter} onLeave={leave} />}
    </SkillsContext.Provider>
  );
}
