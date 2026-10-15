import type {
  ReplayAnalysisFailure,
  ReplayAnalysisStepResult,
  ReplayAnalysisSummary,
  ReplayFrameAnnotations,
} from "./replay_analysis_protocol";

type RecordValue = Record<string, unknown>;

function object(value: unknown): asserts value is RecordValue {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error("Expected an analysis object");
}

function integer(value: unknown, max = Number.MAX_SAFE_INTEGER): asserts value is number {
  if (!Number.isSafeInteger(value) || (value as number) < 0 || (value as number) > max) {
    throw new Error("Expected an in-range analysis integer");
  }
}

function text(value: unknown): asserts value is string {
  if (typeof value !== "string") throw new Error("Expected analysis text");
}

function choice(value: unknown, choices: readonly string[]): void {
  if (typeof value !== "string" || !choices.includes(value)) throw new Error("Unknown analysis value");
}

function side(value: unknown): void { choice(value, ["Black", "White"]); }
function move(value: unknown): void {
  object(value);
  integer(value.row, 14);
  integer(value.col, 14);
}

function list(value: unknown, validate: (item: unknown) => void): void {
  if (!Array.isArray(value)) throw new Error("Expected an analysis array");
  value.forEach(validate);
}

function optional(value: unknown, validate: (item: unknown) => void): void {
  if (value != null) validate(value);
}

function interval(value: unknown): void {
  object(value);
  integer(value.start_ply, 225);
  integer(value.end_ply, 225);
  if (value.end_ply < value.start_ply) throw new Error("Reversed analysis interval");
}

const HIGHLIGHT_ROLES = ["immediate_win", "immediate_threat", "imminent_threat", "counter_threat", "corridor_entry"];
const MARKER_ROLES = ["confirmed_escape", "possible_escape", "forced_loss", "immediate_loss", "forbidden", "unknown"];
const REPLY_ROLES = ["actual", "immediate_defense", "imminent_defense", "offensive_counter"];

function hints(value: unknown, roles: readonly string[]): void {
  list(value, (hint) => {
    object(hint);
    move(hint.mv);
    text(hint.notation);
    choice(hint.role, roles);
    side(hint.side);
  });
}

export function validateReplayFrameAnnotations(value: unknown): ReplayFrameAnnotations {
  object(value);
  integer(value.ply, 225);
  side(value.side_to_move);
  hints(value.highlights, HIGHLIGHT_ROLES);
  hints(value.markers, MARKER_ROLES);
  optional(value.evidence, (items) => hints(items, HIGHLIGHT_ROLES));
  return value as unknown as ReplayFrameAnnotations;
}

export function validateReplayAnalysisFailure(value: unknown): ReplayAnalysisFailure {
  object(value);
  choice(value.confidence, ["confirmed", "possible", "unclear"]);
  choice(value.mode, ["missed_immediate_win", "missed_immediate_response", "missed_imminent_response", "missed_escape", "missed_lethal_prevention", "unclear"]);
  side(value.side);
  optional(value.prefix_ply, (v) => integer(v, 225));
  optional(value.prevented_onset_ply, (v) => integer(v, 225));
  optional(value.actual_move, move);
  optional(value.actual_notation, text);
  list(value.missed_candidates, (candidate) => {
    object(candidate);
    move(candidate.mv);
    text(candidate.notation);
    choice(candidate.outcome, ["confirmed_escape", "possible_escape", "prevents_lethal_onset", "prevents_corridor_entry"]);
    list(candidate.roles, (v) => choice(v, REPLY_ROLES));
  });
  return value as unknown as ReplayAnalysisFailure;
}

export function validateReplayAnalysisSummary(value: unknown): ReplayAnalysisSummary {
  object(value);
  // The step envelope is version 1; its native GameAnalysis summary has an
  // independent version (currently 21). Validate the consumed projection.
  optional(value.schema_version, integer);
  optional(value.failure, validateReplayAnalysisFailure);
  optional(value.final_forced_interval, interval);
  optional(value.setup_corridor, interval);
  optional(value.lethal_onset, (onset) => {
    object(onset);
    integer(onset.prefix_ply, 225);
    optional(onset.attacker, side);
    optional(onset.defender, side);
    optional(onset.kind, (v) => choice(v, ["terminal_coverage", "one_step_coverage"]));
    optional(onset.covering_replies, (v) => list(v, move));
    optional(onset.terminal_targets, (v) => list(v, move));
    optional(onset.shape, (shape) => {
      object(shape);
      text(shape.label);
      list(shape.mechanisms, (v) => choice(v, ["multi_route", "forbidden_cover"]));
      list(shape.components, (component) => {
        object(component);
        move(component.mv);
        choice(component.tier, ["four", "three"]);
      });
    });
    optional(onset.one_step_replies, (v) => list(v, (reply) => {
      object(reply);
      move(reply.reply);
      list(reply.lethal_entries, (entry) => {
        object(entry);
        move(entry.mv);
        list(entry.terminal_targets, move);
      });
    }));
  });
  return value as unknown as ReplayAnalysisSummary;
}

export function validateReplayAnalysisStep(value: unknown): ReplayAnalysisStepResult {
  object(value);
  if (value.schema_version !== 1) throw new Error("Unsupported analysis step version");
  choice(value.status, ["running", "resolved", "unclear", "unsupported", "error"]);
  if (typeof value.done !== "boolean") throw new Error("Expected analysis completion flag");
  if (value.current_ply !== null) integer(value.current_ply, 225);
  if (value.error !== null) text(value.error);
  list(value.annotations, validateReplayFrameAnnotations);
  object(value.counters);
  for (const field of ["branch_roots", "prefixes_analyzed", "proof_nodes"]) integer(value.counters[field]);
  if (value.analysis !== null) validateReplayAnalysisSummary(value.analysis);
  return value as unknown as ReplayAnalysisStepResult;
}
