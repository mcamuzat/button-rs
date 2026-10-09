import { useSyncExternalStore } from "react";
import type { GameAdapter, Snapshot } from "./GameAdapter";

export function useGame(adapter: GameAdapter): Snapshot {
  return useSyncExternalStore(adapter.subscribe, adapter.getSnapshot);
}
