import type { SavedMatchV2 } from "../match/saved_match";

import { replayAnalysisOptionsJson, type ReplayAnalysisOptions } from "./replay_analysis_core";
import type { ReplayAnalysisAnnotationsByPly } from "./replay_analysis_overlays";
import type { ReplayAnalysisStepResult } from "./replay_analysis_protocol";

const CACHE_STORAGE_KEY = "gomoku2d:replay-analysis-cache:v1";
const CACHE_VERSION = 1;
const CACHE_MAX_ENTRIES = 20;
const CACHE_MAX_BYTES = 2_000_000;

export interface ReplayAnalysisCachedResult {
  annotationsByPly: ReplayAnalysisAnnotationsByPly;
  step: ReplayAnalysisStepResult;
}

interface ReplayAnalysisCacheRecord {
  key: string;
  lastUsedAtMs: number;
  result: ReplayAnalysisCachedResult;
  savedAtMs: number;
}

interface ReplayAnalysisCacheDocument {
  records: ReplayAnalysisCacheRecord[];
  version: typeof CACHE_VERSION;
}

type ReplayAnalysisCacheStorage = Pick<Storage, "getItem" | "removeItem" | "setItem">;

function storageOrNull(storage?: ReplayAnalysisCacheStorage): ReplayAnalysisCacheStorage | null {
  if (storage) {
    return storage;
  }

  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object";
}

function isReplayAnalysisCacheRecord(value: unknown): value is ReplayAnalysisCacheRecord {
  return isRecord(value)
    && typeof value.key === "string"
    && typeof value.lastUsedAtMs === "number"
    && typeof value.savedAtMs === "number"
    && isRecord(value.result)
    && isRecord(value.result.step)
    && isRecord(value.result.annotationsByPly);
}

function emptyCacheDocument(): ReplayAnalysisCacheDocument {
  return { records: [], version: CACHE_VERSION };
}

function loadCacheDocument(storage: ReplayAnalysisCacheStorage): ReplayAnalysisCacheDocument {
  try {
    const raw = storage.getItem(CACHE_STORAGE_KEY);
    if (!raw) {
      return emptyCacheDocument();
    }

    const parsed = JSON.parse(raw) as unknown;
    if (!isRecord(parsed) || parsed.version !== CACHE_VERSION || !Array.isArray(parsed.records)) {
      throw new Error("invalid replay analysis cache document");
    }

    return {
      records: parsed.records.filter(isReplayAnalysisCacheRecord),
      version: CACHE_VERSION,
    };
  } catch {
    try {
      storage.removeItem(CACHE_STORAGE_KEY);
    } catch {
      // Storage access can be blocked entirely; cache reads must remain best-effort.
    }
    return emptyCacheDocument();
  }
}

function cachePayloadSize(document: ReplayAnalysisCacheDocument): number {
  return new Blob([JSON.stringify(document)]).size;
}

function trimCacheDocument(document: ReplayAnalysisCacheDocument): ReplayAnalysisCacheDocument {
  const records = [...document.records]
    .sort((a, b) => b.lastUsedAtMs - a.lastUsedAtMs)
    .slice(0, CACHE_MAX_ENTRIES);
  const trimmed: ReplayAnalysisCacheDocument = { records, version: CACHE_VERSION };

  while (trimmed.records.length > 0 && cachePayloadSize(trimmed) > CACHE_MAX_BYTES) {
    trimmed.records.pop();
  }

  return trimmed;
}

function saveCacheDocument(storage: ReplayAnalysisCacheStorage, document: ReplayAnalysisCacheDocument): void {
  let trimmed = trimCacheDocument(document);

  while (trimmed.records.length > 0) {
    try {
      storage.setItem(CACHE_STORAGE_KEY, JSON.stringify(trimmed));
      return;
    } catch {
      trimmed = { records: trimmed.records.slice(0, -1), version: CACHE_VERSION };
    }
  }

  try {
    storage.removeItem(CACHE_STORAGE_KEY);
  } catch {
    // localStorage can be unavailable or quota-constrained; cache writes are best-effort.
  }
}

export function replayAnalysisCacheKey(match: SavedMatchV2, options: ReplayAnalysisOptions): string {
  return JSON.stringify({
    match: {
      move_cells: match.move_cells,
      move_count: match.move_count,
      ruleset: match.ruleset,
      schema_version: match.schema_version,
      status: match.status,
    },
    options: JSON.parse(replayAnalysisOptionsJson(options)) as unknown,
    version: CACHE_VERSION,
  });
}

export function clearReplayAnalysisCache(storage?: ReplayAnalysisCacheStorage): void {
  const targetStorage = storageOrNull(storage);
  if (!targetStorage) {
    return;
  }

  try {
    targetStorage.removeItem(CACHE_STORAGE_KEY);
  } catch {
    // localStorage can be unavailable; clearing a best-effort cache must not block profile reset.
  }
}

export function readReplayAnalysisCache(
  match: SavedMatchV2,
  options: ReplayAnalysisOptions,
  storage?: ReplayAnalysisCacheStorage,
): ReplayAnalysisCachedResult | null {
  const targetStorage = storageOrNull(storage);
  if (!targetStorage) {
    return null;
  }

  const key = replayAnalysisCacheKey(match, options);
  const document = loadCacheDocument(targetStorage);
  const record = document.records.find((candidate) => candidate.key === key);
  if (!record) {
    return null;
  }

  record.lastUsedAtMs = Date.now();
  saveCacheDocument(targetStorage, document);
  return record.result;
}

export function writeReplayAnalysisCache(
  match: SavedMatchV2,
  options: ReplayAnalysisOptions,
  result: ReplayAnalysisCachedResult,
  storage?: ReplayAnalysisCacheStorage,
): void {
  if (!result.step.done || result.step.status === "error") {
    return;
  }

  const targetStorage = storageOrNull(storage);
  if (!targetStorage) {
    return;
  }

  const key = replayAnalysisCacheKey(match, options);
  const now = Date.now();
  const document = loadCacheDocument(targetStorage);
  const records = document.records.filter((record) => record.key !== key);
  records.unshift({
    key,
    lastUsedAtMs: now,
    result,
    savedAtMs: now,
  });

  saveCacheDocument(targetStorage, { records, version: CACHE_VERSION });
}
