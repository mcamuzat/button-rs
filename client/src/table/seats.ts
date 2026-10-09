import type { PlayerId } from "../engine/generated/PlayerId";

export function seatLabel(p: PlayerId): string {
  return p === "One" ? "Joueur 1" : "Joueur 2";
}
