import { useEffect, useRef, useState } from "react";

import type { SavedMatchV2 } from "../match/saved_match";

import {
  readReplayAnalysisCache,
  writeReplayAnalysisCache,
} from "./replay_analysis_cache";
import type { ReplayAnalysisOptions } from "./replay_analysis_core";
import {
  mergeReplayAnalysisAnnotations,
  type ReplayAnalysisAnnotationsByPly,
} from "./replay_analysis_overlays";
import {
  replayAnalysisErrorResult,
  type ReplayAnalysisStepResult,
} from "./replay_analysis_protocol";
import { ReplayAnalysisRunner } from "./replay_analysis_runner";

export const REPLAY_ANALYSIS_OPTIONS: ReplayAnalysisOptions = { maxDepth: 4, maxScanPlies: 64 };
export const REPLAY_ANALYSIS_STEP_WORK_UNITS = 1;

export interface ReplayAnalysisState {
  analysisAnnotations: ReplayAnalysisAnnotationsByPly;
  analysisStep: ReplayAnalysisStepResult | null;
}

export function useReplayAnalysis(match: SavedMatchV2 | null): ReplayAnalysisState {
  const [analysisAnnotations, setAnalysisAnnotations] = useState<ReplayAnalysisAnnotationsByPly>({});
  const [analysisStep, setAnalysisStep] = useState<ReplayAnalysisStepResult | null>(null);
  const analysisRunnerRef = useRef<ReplayAnalysisRunner | null>(null);

  useEffect(() => {
    return () => {
      analysisRunnerRef.current?.cancel();
      analysisRunnerRef.current?.dispose();
      analysisRunnerRef.current = null;
    };
  }, []);

  useEffect(() => {
    if (!match) {
      analysisRunnerRef.current?.cancel();
      setAnalysisAnnotations({});
      setAnalysisStep(null);
      return undefined;
    }

    setAnalysisAnnotations({});
    setAnalysisStep(null);
    analysisRunnerRef.current?.cancel();

    const cached = readReplayAnalysisCache(match, REPLAY_ANALYSIS_OPTIONS);
    if (cached) {
      setAnalysisAnnotations(cached.annotationsByPly);
      setAnalysisStep(cached.step);
      return undefined;
    }

    let accumulatedAnnotations: ReplayAnalysisAnnotationsByPly = {};
    const mergeStep = (step: ReplayAnalysisStepResult) => {
      accumulatedAnnotations = mergeReplayAnalysisAnnotations(accumulatedAnnotations, step);
      setAnalysisStep(step);
      setAnalysisAnnotations(accumulatedAnnotations);
      return accumulatedAnnotations;
    };

    try {
      let runner = analysisRunnerRef.current;
      if (!runner) {
        runner = new ReplayAnalysisRunner();
        analysisRunnerRef.current = runner;
      }

      runner.analyze(
        match,
        {
          onComplete: (step) => {
            const annotationsByPly = mergeStep(step);
            writeReplayAnalysisCache(match, REPLAY_ANALYSIS_OPTIONS, {
              annotationsByPly,
              step,
            });
          },
          onError: (error) => {
            setAnalysisAnnotations({});
            setAnalysisStep(replayAnalysisErrorResult(error.message));
          },
          onProgress: mergeStep,
        },
        REPLAY_ANALYSIS_OPTIONS,
        REPLAY_ANALYSIS_STEP_WORK_UNITS,
      );
    } catch {
      setAnalysisAnnotations({});
      setAnalysisStep(replayAnalysisErrorResult("Replay analyzer could not start"));
    }

    return () => {
      analysisRunnerRef.current?.cancel();
    };
  }, [match]);

  return { analysisAnnotations, analysisStep };
}
