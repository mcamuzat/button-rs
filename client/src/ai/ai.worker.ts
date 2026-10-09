/// <reference lib="webworker" />
import init, { ai_choose_action } from "../wasm/buttonmen";
import type { WorkerRequest, WorkerResponse } from "./brain";

let ready: Promise<unknown> | null = null;

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const { id, stateJson, player, level, seed } = event.data;
  let response: WorkerResponse;
  try {
    ready ??= init();
    await ready;
    const json = ai_choose_action(stateJson, JSON.stringify(player), JSON.stringify(level), seed);
    response = { id, actionJson: json === "null" ? null : json };
  } catch (e) {
    response = { id, error: e instanceof Error ? e.message : String(e) };
  }
  self.postMessage(response);
};
