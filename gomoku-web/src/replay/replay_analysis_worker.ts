/// <reference lib="webworker" />

import { WasmReplayAnalyzer } from "../core/wasm_bridge";

import type { ReplayAnalysisStepResult, ReplayAnalysisWorkerRequest, ReplayAnalysisWorkerResponse } from "./replay_analysis_protocol";

const workerScope = self as DedicatedWorkerGlobalScope;

type ActiveAnalysis = {
  analyzer: WasmReplayAnalyzer;
  requestId: number;
  stepWorkUnits: number;
};

let activeAnalysis: ActiveAnalysis | null = null;

function postMessage(message: ReplayAnalysisWorkerResponse): void {
  workerScope.postMessage(message);
}

function freeAnalyzer(analyzer: WasmReplayAnalyzer): void {
  analyzer.dispose();
  analyzer.free();
}

function cancelActiveAnalysis(postCancellation: boolean): void {
  const active = activeAnalysis;
  if (!active) return;

  activeAnalysis = null;
  freeAnalyzer(active.analyzer);
  if (postCancellation) {
    postMessage({ requestId: active.requestId, type: "cancelled" });
  }
}

function parseStepResult(json: string): ReplayAnalysisStepResult {
  return JSON.parse(json) as ReplayAnalysisStepResult;
}

function runStep(active: ActiveAnalysis): void {
  if (activeAnalysis !== active) return;

  try {
    const result = parseStepResult(active.analyzer.step(active.stepWorkUnits));
    if (activeAnalysis !== active) return;

    postMessage({
      requestId: active.requestId,
      result,
      type: result.done ? "complete" : "progress",
    });

    if (result.done) {
      cancelActiveAnalysis(false);
      return;
    }

    workerScope.setTimeout(() => runStep(active), 0);
  } catch (error) {
    cancelActiveAnalysis(false);
    postMessage({
      message: error instanceof Error ? error.message : String(error),
      requestId: active.requestId,
      type: "error",
    });
  }
}

function handleAnalyze(message: Extract<ReplayAnalysisWorkerRequest, { type: "analyze" }>): void {
  cancelActiveAnalysis(true);

  try {
    const analyzer = WasmReplayAnalyzer.createFromReplayJson(message.replayJson, message.optionsJson);
    const active: ActiveAnalysis = {
      analyzer,
      requestId: message.requestId,
      stepWorkUnits: Math.max(1, message.stepWorkUnits),
    };
    activeAnalysis = active;
    workerScope.setTimeout(() => runStep(active), 0);
  } catch (error) {
    postMessage({
      message: error instanceof Error ? error.message : String(error),
      requestId: message.requestId,
      type: "error",
    });
  }
}

workerScope.addEventListener("message", (event: MessageEvent<ReplayAnalysisWorkerRequest>) => {
  const message = event.data;

  switch (message.type) {
    case "analyze":
      handleAnalyze(message);
      break;
    case "cancel":
      if (activeAnalysis?.requestId === message.requestId) {
        cancelActiveAnalysis(true);
      }
      break;
  }
});

postMessage({ type: "ready" });
