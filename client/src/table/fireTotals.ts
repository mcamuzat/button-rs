import type { Action } from "../engine/generated/Action";
import type { DieId } from "../engine/generated/DieId";

type Amounts = Record<DieId, number>;

const entries = (amounts: Amounts): [DieId, number][] =>
  Object.entries(amounts)
    .map(([id, n]) => [Number(id), n] as [DieId, number])
    .filter(([, n]) => n > 0);

/** Turns the amounts typed in the Fire panel into an action; the engine validates it. */
export function fireAction(fire: Amounts, boosts: Amounts): Action | null {
  const f = entries(fire);
  const b = entries(boosts);
  const total = (list: [DieId, number][]) => list.reduce((sum, [, n]) => sum + n, 0);
  if (total(f) === 0 || total(f) !== total(b)) return null;
  return { type: "FireTurnDown", fire: f, boosts: b };
}
