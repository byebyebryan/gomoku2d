import { createHash } from "node:crypto";
import { isAbsolute } from "node:path";

export const CAPTURE_PROFILES = Object.freeze({
  desktop: Object.freeze({
    id: "desktop",
    viewport: Object.freeze({ width: 1280, height: 900 }),
    deviceScaleFactor: 1,
    isMobile: false,
    hasTouch: false,
    userAgentKind: "desktop Chromium default",
  }),
  portrait: Object.freeze({
    id: "portrait",
    viewport: Object.freeze({ width: 412, height: 800 }),
    deviceScaleFactor: 1,
    isMobile: true,
    hasTouch: true,
    userAgentKind: "generic Android mobile; not a measured OnePlus profile",
  }),
});

const TAGS = [
  ["v0.2.1", "add1953c3228849a8d37c424d0deec971c1fc937"],
  ["v0.2.2", "a0a87e2adaf31f50c5e45f245c9765b09eac6fbc"],
  ["v0.2.3", "95b9707a6c01d77fa50d181bdd1ee4542047a2a2"],
  ["v0.2.4", "d9a3476d11dcb24e1c3fa67374a27b867a781da1"],
  ["v0.3.0", "5e44832d872cc1030be6707b97dcd9f9b067d83c"],
  ["v0.3.1", "b9fcc3df81cc2ccea24140a3556db40c1929c5f7"],
  ["v0.3.2", "3a8dbb05d94f8811e995ae7621329a390a313954"],
  ["v0.3.3", "73c1698667ddb85085feedf42c7f2cc2a9de7073"],
  ["v0.4.0", "0fc4a45b79c5056cefa5e7485599c0eef5e6295a"],
  ["v0.4.1", "aaa6ac548fc2f3fc8c1a3815abd262473823be6f"],
  ["v0.4.2", "5e132248d9602ab0ce3abbdf86da8f92a3fa0e26"],
  ["v0.4.3", "dba0fb4213520d30f3640729cccaad0ceb5ccc58"],
  ["v0.4.4", "9744aa2576860fbc3af14148504cb2144ef69b75"],
  ["v0.4.5", "b97dcff422350ebe78bb0d2cf5061ce52a408ed0"],
  ["v0.4.6", "2ec50c02f29deb02c14295d07929e9f7e835fd2b"],
  ["v0.4.7", "0b93fed092975c172959343a45decc9c8c89fe2e"],
  ["v0.4.8", "ea188acaf3e98b0b0b0952c05993391bc8a8a1ef"],
  ["v0.5.0", "678a69ac51f2b1f4fa968bf9fe03b69831b847c9"],
  ["v0.5.1", "bf861f596e7a5bf03b588953f6552b9a3c4c7827"],
  ["v0.5.2", "71793b0f21adf408b5b714514937d979ecffc944"],
  ["v0.5.3", "8fa3035f2e2e7445907c1d74903d27b1fa072c1a"],
  ["v0.5.4", "b09ac7ff56f1452bccdf63334ce38415b92fdbd8"],
  ["v0.5.5", "73658d0a546ba558c746854d38cdac7ec2465802"],
  ["v0.6.1", "b1a67d32feee5b57275e3d04c684e27bcd1a571f"],
  ["v0.6.2", "37a122d2bd8e899bcfc581055e685c0327f43cb4"],
];

/** @param {string} tag */
function schemaForTag(tag) {
  if (tag <= "v0.2.4") return "guest-v1";
  if (tag === "v0.3.0") return "guest-v1";
  if (tag <= "v0.3.2") return "guest-v2";
  if (tag <= "v0.4.4") return "local-v3";
  return "local-v5";
}

/** @param {string} tag */
function reportInputsForTag(tag) {
  if (tag < "v0.4.0") return [];
  if (tag <= "v0.4.1") {
    return ["gomoku-bot-lab/reports/index.html", "gomoku-bot-lab/reports/latest.json"];
  }
  if (tag <= "v0.5.0") {
    return [
      "gomoku-bot-lab/reports/index.html",
      "gomoku-bot-lab/reports/latest.json",
      "gomoku-bot-lab/analysis-reports/index.html",
      "gomoku-bot-lab/analysis-reports/latest.json",
    ];
  }
  if (tag <= "v0.5.2") {
    return [
      "gomoku-bot-lab/reports/report.json",
      "gomoku-bot-lab/analysis-reports/report.json",
    ];
  }
  return ["reports/lab/bot-report.json", "reports/lab/analysis-report.json"];
}

/** @type {Record<string, any>} */
export const TAG_ADAPTERS = Object.freeze(Object.fromEntries(TAGS.map(([tag, commit]) => [tag, Object.freeze({
  tag,
  commit,
  adapter: schemaForTag(tag),
  profileSchema: schemaForTag(tag),
  hasReplayBranch: tag >= "v0.2.2",
  hasReplayAnalysis: tag >= "v0.4.6",
  hasSettings: tag >= "v0.4.5",
  hasLab: tag >= "v0.5.1",
  hasRulesAndGuide: tag >= "v0.5.2",
  hasPolicyRoutes: tag >= "v0.5.3",
  hasPublicSharing: tag >= "v0.6.1",
  reportInputs: Object.freeze(reportInputsForTag(tag)),
})])));

const RESERVED_PORTS = new Set([
  3000, 3001, 4000, 5000, 5001, 5173, 8001, 8013, 8080, 8085, 9000, 9099, 9199,
  18081, 19099, 46862,
]);
const HASH = /^[a-f0-9]{64}$/;
const GIT_TREE_OID = /^[a-f0-9]{40}$/;
const SAFE_ID = /^[a-zA-Z0-9][a-zA-Z0-9_.-]*$/;
const AUTH_STATES = new Set([
  "account", "sync", "reset", "share", "publish", "public-settings",
]);
/** @type {Record<string, Set<string>>} */
const SCREEN_STATES = Object.freeze({
  home: new Set(["default"]),
  match: new Set(["default"]),
  profile: new Set(["default", "empty", "account", "sync", "reset", "saved-replays", "public-settings"]),
  replay: new Set(["default", "share", "publish"]),
  settings: new Set(["default", "custom"]),
  "bot-report": new Set(["default", "search", "pairwise"]),
  "analysis-report": new Set(["default", "proof-open"]),
  visuals: new Set(["default", "sprites", "icons", "fonts"]),
  lab: new Set(["default", "search", "analysis"]),
  rules: new Set(["default"]),
  guide: new Set(["default", "examples"]),
  privacy: new Set(["default"]),
  terms: new Set(["default"]),
  "public-player": new Set(["default"]),
  "shared-replay": new Set(["default", "from-profile"]),
});

/** @param {unknown} value @returns {value is Record<string, any>} */
function isRecord(value) {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** @param {unknown} value @param {string} label */
function requireText(value, label) {
  if (typeof value !== "string" || value.trim() === "") throw new Error(label + " must be a non-empty string.");
  return value.trim();
}

/** @param {unknown} value @param {string} label */
function requireHash(value, label) {
  const normalized = requireText(value, label).toLowerCase();
  if (!HASH.test(normalized)) throw new Error(label + " must be a 64-character SHA-256 hash.");
  return normalized;
}

/** @param {unknown} value @param {string} label */
function requireTreeOid(value, label) {
  const normalized = requireText(value, label).toLowerCase();
  if (!GIT_TREE_OID.test(normalized)) throw new Error(label + " must be a 40-character Git tree OID.");
  return normalized;
}

/** @param {unknown} receipt @param {string} tag */
function requireReceipt(receipt, tag) {
  if (!isRecord(receipt)) throw new Error("source.buildReceipt must be the parsed receipt object.");
  if (receipt.schemaVersion !== 1 || receipt.tag !== tag) throw new Error(tag + " build receipt schema/tag mismatch.");
  for (const field of ["recordedAt", "sourceCommit", "sourceTree", "applicationTree", "packageLockSha256", "wasmSha256", "distSha256", "indexSha256"]) {
    requireText(receipt[field], "buildReceipt." + field);
  }
  requireTreeOid(receipt.sourceTree, "buildReceipt.sourceTree");
  requireTreeOid(receipt.applicationTree, "buildReceipt.applicationTree");
  requireHash(receipt.packageLockSha256, "buildReceipt.packageLockSha256");
  requireHash(receipt.wasmSha256, "buildReceipt.wasmSha256");
  requireHash(receipt.distSha256, "buildReceipt.distSha256");
  requireHash(receipt.indexSha256, "buildReceipt.indexSha256");
  if (!isRecord(receipt.cargoLock) || !["tracked", "generated-external"].includes(receipt.cargoLock.kind)) {
    throw new Error("buildReceipt.cargoLock.kind must be tracked or generated-external.");
  }
  requireHash(receipt.cargoLock.sha256, "buildReceipt.cargoLock.sha256");
  if (!isRecord(receipt.environment)) throw new Error("buildReceipt.environment must be an object.");
  for (const field of ["node", "npm", "rustc", "cargo", "wasmPack", "firebaseMode"]) {
    requireText(receipt.environment[field], "buildReceipt.environment." + field);
  }
  if (!Array.isArray(receipt.files) || receipt.files.length === 0) {
    throw new Error("buildReceipt.files must contain the dist file inventory.");
  }
  const fileNames = new Set();
  for (const [index, file] of receipt.files.entries()) {
    if (!isRecord(file)) throw new Error("buildReceipt.files[" + index + "] must be an object.");
    const path = requireText(file.path, "buildReceipt.files[" + index + "].path");
    requireHash(file.sha256, "buildReceipt.files[" + index + "].sha256");
    if (path.startsWith("/") || path.includes("\\") || path.split("/").some((part) => part === ".." || part === "")) {
      throw new Error("Build receipt contains an unsafe dist path: " + path);
    }
    if (fileNames.has(path)) throw new Error("Build receipt has a duplicate dist path: " + path);
    fileNames.add(path);
  }
  if (!Array.isArray(receipt.reportInputs)) throw new Error("buildReceipt.reportInputs must be an array.");
  return receipt;
}

/** @param {any} adapter */
function availableScreens(adapter) {
  const screens = new Set(["home", "match", "profile", "replay"]);
  if (adapter.tag >= "v0.2.4") screens.add("visuals");
  if (adapter.tag >= "v0.3.0") {
    screens.add("privacy");
    screens.add("terms");
  }
  if (adapter.tag >= "v0.4.0" && adapter.tag <= "v0.5.0") screens.add("bot-report");
  if (adapter.tag >= "v0.4.2" && adapter.tag <= "v0.5.0") screens.add("analysis-report");
  if (adapter.hasSettings) screens.add("settings");
  if (adapter.hasLab) screens.add("lab");
  if (adapter.hasRulesAndGuide) {
    screens.add("rules");
    screens.add("guide");
  }
  if (adapter.hasPublicSharing) {
    screens.add("public-player");
    screens.add("shared-replay");
  }
  return screens;
}

/** @param {any} scene */
function sceneNeedsTransport(scene) {
  return AUTH_STATES.has(scene.state) || scene.screen === "public-player" || scene.screen === "shared-replay";
}

/** @param {any} adapter @param {Array<any>} scenes */
function expectedReportInputs(adapter, scenes) {
  const screenSet = new Set(scenes.map((scene) => scene.screen));
  if (!["bot-report", "analysis-report", "lab"].some((screen) => screenSet.has(screen))) return [];
  if (screenSet.has("lab")) return [...adapter.reportInputs];
  return adapter.reportInputs.filter((/** @type {string} */ path) => {
    const isAnalysis = /analysis-report/.test(path);
    return (isAnalysis && screenSet.has("analysis-report"))
      || (!isAnalysis && screenSet.has("bot-report"));
  });
}

/** Validate source, scene, state, profile and receipt fields before filesystem work. */
/** @param {any} input @returns {any} */
export function validateHistoryCaptureConfig(input) {
  if (!isRecord(input) || input.schemaVersion !== 1) throw new Error("Config schemaVersion must be 1.");
  if (!isRecord(input.source)) throw new Error("Config source must be one prepared source object.");
  const tag = requireText(input.source.tag, "source.tag");
  const adapter = TAG_ADAPTERS[tag];
  if (!adapter) throw new Error("Unsupported history source tag: " + tag);
  const expectedCommit = requireText(input.source.expectedCommit, "source.expectedCommit").toLowerCase();
  if (!/^[a-f0-9]{40}$/.test(expectedCommit)) throw new Error("source.expectedCommit must be a full 40-character commit.");
  if (expectedCommit !== adapter.commit) throw new Error(tag + " config commit does not match the audited tag commit.");
  if (input.source.adapter !== adapter.adapter) throw new Error(tag + " requires adapter " + adapter.adapter + ".");
  const worktree = requireText(input.source.worktree, "source.worktree");
  if (!isAbsolute(worktree)) throw new Error("source.worktree must be an absolute path.");
  const baseUrl = requireText(input.source.baseUrl, "source.baseUrl");
  let parsedUrl;
  try { parsedUrl = new URL(baseUrl); } catch { throw new Error("source.baseUrl must be a valid URL."); }
  if (parsedUrl.protocol !== "http:" || !["127.0.0.1", "localhost", "[::1]"].includes(parsedUrl.hostname)
    || parsedUrl.username || parsedUrl.password || parsedUrl.pathname !== "/" || parsedUrl.search || parsedUrl.hash
    || !parsedUrl.port) {
    throw new Error("source.baseUrl must be a root HTTP loopback URL with an explicit port.");
  }
  const port = Number(parsedUrl.port);
  if (!Number.isInteger(port) || port < 1024 || port > 65535 || RESERVED_PORTS.has(port)) {
    throw new Error("source.baseUrl uses a reserved or invalid port: " + parsedUrl.port);
  }
  const buildReceipt = requireReceipt(input.source.buildReceipt, tag);
  if (buildReceipt.sourceCommit !== expectedCommit) throw new Error(tag + " build receipt source commit mismatch.");
  const expectedLockKind = ["v0.2.1", "v0.2.2", "v0.2.3"].includes(tag) ? "generated-external" : "tracked";
  if (buildReceipt.cargoLock.kind !== expectedLockKind) {
    throw new Error(tag + " build receipt Cargo.lock kind must be " + expectedLockKind + ".");
  }

  const profiles = input.profiles ?? ["desktop", "portrait"];
  if (!Array.isArray(profiles) || profiles.length === 0 || profiles.some((id) => !Object.hasOwn(CAPTURE_PROFILES, id))) {
    throw new Error("profiles must be a non-empty list of desktop and/or portrait.");
  }
  if (new Set(profiles).size !== profiles.length) throw new Error("profiles must not contain duplicates.");
  if (!Array.isArray(input.scenes) || input.scenes.length === 0) throw new Error("scenes must contain at least one screen/state.");
  const available = availableScreens(adapter);
  const sceneKeys = new Set();
  const scenes = input.scenes.map((/** @type {any} */ raw, /** @type {number} */ index) => {
    if (!isRecord(raw)) throw new Error("scenes[" + index + "] must be an object.");
    const screen = requireText(raw.screen, "scenes[" + index + "].screen");
    const state = raw.state === undefined ? "default" : requireText(raw.state, "scenes[" + index + "].state");
    if (!SAFE_ID.test(screen) || !SAFE_ID.test(state)) throw new Error("Scene screen/state IDs must be simple identifiers.");
    if (!available.has(screen)) throw new Error(tag + " does not contain the " + screen + " screen.");
    const allowedStates = SCREEN_STATES[screen];
    if (!allowedStates?.has(state)) throw new Error("Unsupported state " + screen + "/" + state + ".");
    if (tag === "v0.4.0" && screen === "bot-report" && ["search", "pairwise"].includes(state)) {
      throw new Error("v0.4.0 Bot Report is a static ranking page without Search or Pairwise controls.");
    }
    if (screen === "profile" && state === "empty" && !["v0.5.3", "v0.5.4"].includes(tag)) {
      throw new Error(tag + " does not contain the tagged Profile empty-history state.");
    }
    if (["share", "publish"].includes(state) && (screen !== "replay" || !adapter.hasPublicSharing)) {
      throw new Error(tag + " has no tagged Replay " + state + " state.");
    }
    if (state === "public-settings" && (screen !== "profile" || !adapter.hasPublicSharing)) {
      throw new Error(tag + " has no tagged Profile public-settings state.");
    }
    if (state === "saved-replays" && (screen !== "profile" || !adapter.hasPublicSharing)) {
      throw new Error(tag + " has no tagged Profile saved-replays state.");
    }
    if (screen === "shared-replay" && state === "from-profile" && tag < "v0.6.2") {
      throw new Error(tag + " predates the Shared Replay return-to-profile state.");
    }
    if (screen === "analysis-report" && state === "proof-open" && tag > "v0.5.0") {
      throw new Error(tag + " no longer publishes the standalone Analysis Report page.");
    }
    if (screen === "lab" && state !== "default" && !adapter.hasLab) {
      throw new Error(tag + " predates the React Lab interaction states.");
    }
    if (state === "account" && tag < "v0.3.0") throw new Error(tag + " predates cloud account state.");
    if (state === "sync" && tag < "v0.3.1") throw new Error(tag + " predates cloud sync state.");
    if (state === "reset" && tag < "v0.3.2") throw new Error(tag + " predates Profile reset confirmation.");
    if (screen === "settings" && !adapter.hasSettings) throw new Error(tag + " predates Settings.");
    if (screen === "lab" && !adapter.hasLab) throw new Error(tag + " predates the React Lab.");
    const key = screen + ":" + state;
    if (sceneKeys.has(key)) throw new Error("Duplicate scene: " + key);
    sceneKeys.add(key);
    return Object.freeze({ screen, state, requiresTransport: sceneNeedsTransport({ screen, state }) });
  });
  const needsTransport = scenes.some((scene) => scene.requiresTransport);
  const transport = input.transport;
  if (needsTransport && !isRecord(transport)) {
    throw new Error("Requested account/public scene needs an explicit synthetic transport config; production and real accounts are not supported.");
  }
  if (transport !== undefined && !isRecord(transport)) {
    throw new Error("transport must be an object.");
  }
  if ((buildReceipt.environment.firebaseMode === "legacy-proxy" || buildReceipt.environment.firebaseMode === "modern-emulators")
    && transport === undefined) {
    throw new Error("Build receipt uses Firebase transport; provide the approved synthetic-demo transport config.");
  }
  if (!new Set(["none", "legacy-proxy", "modern-emulators"]).has(buildReceipt.environment.firebaseMode)) {
    throw new Error("buildReceipt.environment.firebaseMode must be none, legacy-proxy, or modern-emulators.");
  }
  if (transport !== undefined) {
    if (buildReceipt.environment.firebaseMode === "none") {
      throw new Error("A no-Firebase tagged build cannot use synthetic Auth/Firestore transport.");
    }
    if (!new Set(["legacy-proxy", "native-emulators"]).has(transport.mode)) {
      throw new Error("transport.mode must be legacy-proxy or native-emulators.");
    }
    if (!isAbsolute(requireText(transport.ownerRoot, "transport.ownerRoot"))) {
      throw new Error("transport.ownerRoot must be an absolute path inside the documented synthetic demo root.");
    }
    if (!new Set(["guest", "account"]).has(transport.session)) {
      throw new Error("transport.session must be guest or account.");
    }
    const expectedTransportMode = buildReceipt.environment.firebaseMode === "legacy-proxy"
      ? "legacy-proxy"
      : buildReceipt.environment.firebaseMode === "modern-emulators" ? "native-emulators" : null;
    if (expectedTransportMode && transport.mode !== expectedTransportMode) {
      throw new Error("Transport mode must match the tagged Firebase build mode " + buildReceipt.environment.firebaseMode + ".");
    }
    if (transport.session === "guest" && scenes.some((scene) => ["account", "sync", "reset", "share", "publish", "public-settings"].includes(scene.state))) {
      throw new Error("Account transition scenes require transport.session account.");
    }
    if (transport.session === "account" && scenes.some((scene) => scene.screen === "public-player" || scene.screen === "shared-replay")) {
      throw new Error("Public Player and Shared Replay captures require transport.session guest.");
    }
  }
  if (needsTransport && transport === undefined) {
    throw new Error("Requested account/public scene needs the primary-owned synthetic transport config.");
  }
  const fixture = input.fixture === undefined ? "missed_defense_closed_four" : requireText(input.fixture, "fixture");
  if (fixture !== "missed_defense_closed_four" && fixture !== "pilot-short-win") {
    throw new Error("Unsupported deterministic local fixture: " + fixture);
  }
  const reportInputs = expectedReportInputs(adapter, scenes);
  if (reportInputs.length > 0) {
    const recorded = new Map(buildReceipt.reportInputs.map((/** @type {any} */ item, /** @type {number} */ index) => {
      if (!isRecord(item)) throw new Error("buildReceipt.reportInputs[" + index + "] must be an object.");
      return [requireText(item.path, "buildReceipt.reportInputs[" + index + "].path"),
        requireHash(item.sha256, "buildReceipt.reportInputs[" + index + "].sha256")];
    }));
    for (const path of reportInputs) {
      if (!recorded.has(path)) throw new Error(tag + " report receipt is missing tagged input " + path + ".");
    }
  }
  return Object.freeze({
    schemaVersion: 1,
    source: Object.freeze({
      tag,
      expectedCommit,
      worktree,
      adapter: adapter.adapter,
      baseUrl: parsedUrl.href,
      origin: parsedUrl.origin,
      port,
      buildReceipt,
    }),
    profiles: Object.freeze([...profiles]),
    fixture,
    scenes: Object.freeze(scenes),
    transport: transport === undefined ? undefined : Object.freeze({
      mode: transport.mode,
      ownerRoot: transport.ownerRoot,
      session: transport.session,
    }),
    expectedReportInputs: Object.freeze(reportInputs),
    tagAdapter: adapter,
  });
}

/** @param {string} tag @param {string} screen @param {string} [matchId] @param {string} [state] */
export function routeForScene(tag, screen, matchId = "history-fixture-match", state = "default") {
  const adapter = TAG_ADAPTERS[tag];
  if (!adapter) throw new Error("Unsupported history source tag: " + tag);
  if (screen === "home") return "/";
  if (screen === "match") return "/match/local";
  if (screen === "profile") return "/profile";
  if (screen === "replay") return tag < "v0.3.3" ? "/replays/local/" + matchId : "/replay/" + matchId;
  if (screen === "settings") return "/settings";
  if (screen === "bot-report") return "/bot-report/";
  if (screen === "analysis-report") return "/analysis-report/";
  if (screen === "visuals") {
    if (tag >= "v0.5.2") return "/visuals/";
    if (tag >= "v0.5.1") return "/assets/";
    if (state === "sprites" || state === "icons" || state === "fonts") return "/assets/" + state + "/preview.html";
    return "/assets/preview.html";
  }
  if (screen === "lab") return tag >= "v0.5.2" ? "/lab/" : "/lab-report/";
  if (screen === "rules") return "/rules/";
  if (screen === "guide") return "/guide/";
  if (screen === "privacy") return "/privacy/";
  if (screen === "terms") return "/terms/";
  if (screen === "public-player") return "/player";
  if (screen === "shared-replay") return "/replay/shared";
  throw new Error("No route adapter for screen " + screen + ".");
}

/** @type {Record<string, any>} */
export const FIXTURES = Object.freeze({
  "missed_defense_closed_four": Object.freeze({
    id: "history-fixture-match",
    profileId: "history-fixture-profile",
    displayName: "Reference Player",
    savedAt: "2026-10-24T00:00:00.000Z",
    variant: "freestyle",
    mode: "bot",
    status: "black_won",
    undoFloor: 0,
    moveNames: Object.freeze(["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"]),
    moveCells: Object.freeze([112, 111, 113, 0, 114, 15, 115, 1, 116]),
  }),
  "pilot-short-win": Object.freeze({
    id: "history-fixture-match",
    profileId: "history-fixture-profile",
    displayName: "Reference Player",
    savedAt: "2026-10-24T00:00:00.000Z",
    variant: "freestyle",
    mode: "bot",
    status: "black_won",
    undoFloor: 0,
    moveNames: Object.freeze(["H8", "A1", "I8", "A2", "J8", "A3", "K8", "B1", "L8"]),
    moveCells: Object.freeze([112, 0, 113, 15, 114, 30, 115, 1, 116]),
  }),
});

/** @param {any} fixture @param {string} botName @param {string} profileId */
function savedMatchV1(fixture, botName, profileId) {
  return {
    board_size: 15,
    id: fixture.id,
    match_kind: "local_vs_bot",
    move_cells: [...fixture.moveCells],
    move_count: fixture.moveCells.length,
    player_black: {
      bot: null,
      display_name: fixture.displayName,
      kind: "human",
      local_profile_id: profileId,
      profile_uid: null,
    },
    player_white: {
      bot: {
        config: { depth: 3, kind: "baseline" },
        config_version: 1,
        engine: "baseline_search",
        id: "practice_bot",
        version: 1,
      },
      display_name: botName,
      kind: "bot",
      local_profile_id: null,
      profile_uid: null,
    },
    saved_at: fixture.savedAt,
    schema_version: 1,
    source: "local_history",
    status: fixture.status,
    trust: "local_only",
    undo_floor: fixture.undoFloor,
    variant: fixture.variant,
  };
}

/** @param {any} fixture @param {string} profileId @param {string} botName */
function savedMatchV2(fixture, profileId, botName) {
  const match = savedMatchV1(fixture, botName, profileId);
  delete match.variant;
  return {
    ...match,
    player_white: {
      bot: {
        config: { mode: "preset", preset: "normal", version: 1 },
        config_version: 1,
        engine: "search_bot",
        id: "bot",
        label: "Normal",
        lab_spec: "search-d3+pattern-eval",
        version: 1,
      },
      display_name: botName,
      kind: "bot",
      local_profile_id: null,
      profile_uid: null,
    },
    schema_version: 2,
    ruleset: fixture.variant,
  };
}

function emptyCounter() {
  return { draws: 0, losses: 0, matches: 0, moves: 0, wins: 0 };
}

function emptyArchivedStats() {
  return {
    archived_before: null,
    archived_count: 0,
    by_opponent_type: { bot: emptyCounter(), human: emptyCounter() },
    by_ruleset: { freestyle: emptyCounter(), renju: emptyCounter() },
    by_side: { black: emptyCounter(), white: emptyCounter() },
    schema_version: 1,
    totals: emptyCounter(),
  };
}

/** Build the actual localStorage payload expected by a specific tagged store. */
/** @param {string} tag @param {string} [fixtureId] @param {{emptyHistory?:boolean}} [options] */
export function localStorageSeed(tag, fixtureId = "missed_defense_closed_four", options = {}) {
  const adapter = TAG_ADAPTERS[tag];
  if (!adapter) throw new Error("Unsupported history source tag: " + tag);
  const emptyHistory = options.emptyHistory === true;
  if (emptyHistory && !["v0.5.3", "v0.5.4"].includes(tag)) {
    throw new Error(tag + " does not support the tagged empty local-history fixture.");
  }
  const fixture = FIXTURES[fixtureId];
  if (!fixture) throw new Error("Unsupported deterministic local fixture: " + fixtureId);
  const botName = tag >= "v0.4.5" ? "Normal Bot" : tag < "v0.2.4" ? "Classic Bot" : "Practice Bot";
  const profile = {
    avatarUrl: null,
    createdAt: fixture.savedAt,
    displayName: fixture.displayName,
    id: fixture.profileId,
    kind: adapter.profileSchema.startsWith("local-") ? "local" : "guest",
    updatedAt: fixture.savedAt,
    username: null,
  };
  const winningCells = [
    { row: 7, col: 7 }, { row: 7, col: 8 }, { row: 7, col: 9 },
    { row: 7, col: 10 }, { row: 7, col: 11 },
  ];
  if (adapter.profileSchema === "guest-v1") {
    const moves = fixture.moveCells.map((/** @type {number} */ cell, /** @type {number} */ index) => ({
      col: cell % 15,
      row: Math.floor(cell / 15),
      moveNumber: index + 1,
      player: index % 2 + 1,
    }));
    return {
      key: "gomoku2d.guest-profile.v1",
      value: {
        state: {
          history: [{
            guestStone: "black",
            id: fixture.id,
            mode: "bot",
            moves,
            players: [
              { kind: "human", name: fixture.displayName, stone: "black" },
              { kind: "bot", name: botName, stone: "white" },
            ],
            savedAt: fixture.savedAt,
            status: fixture.status,
            undoFloor: fixture.undoFloor,
            variant: fixture.variant,
            winningCells,
          }],
          profile,
          settings: { preferredVariant: fixture.variant },
        },
        version: 0,
      },
    };
  }
  if (adapter.profileSchema === "guest-v2") {
    return {
      key: "gomoku2d.guest-profile.v2",
      value: {
        state: {
          history: [savedMatchV1(fixture, botName, profile.id)],
          profile,
          settings: { preferredVariant: fixture.variant },
        },
        version: 0,
      },
    };
  }
  const match = adapter.profileSchema === "local-v3"
    ? savedMatchV1(fixture, botName, profile.id)
    : savedMatchV2(fixture, profile.id, botName);
  const matchHistory = {
    archivedStats: emptyArchivedStats(),
    replayMatches: emptyHistory ? [] : [match],
    summaryMatches: [],
  };
  const settings = adapter.profileSchema === "local-v3"
    ? { preferredVariant: fixture.variant }
    : {
      boardHints: { immediate: "win_threat", imminent: "threat_counter" },
      botConfig: { mode: "preset", preset: "normal", version: 1 },
      gameConfig: { opening: "standard", ruleset: fixture.variant },
      touchControl: "touchpad",
    };
  const version = adapter.profileSchema === "local-v3" ? 3 : 5;
  /** @type {Record<string, any>} */
  const seed = {
    key: "gomoku2d.local-profile.v" + version,
    value: { state: { matchHistory, profile, settings }, version },
  };
  if (adapter.hasPublicSharing) {
    seed.additional = [savedReplayLibrarySeed(fixture, botName)];
  }
  return seed;
}

/** @param {any} fixture @param {string} botName */
function savedReplayLibrarySeed(fixture, botName) {
  const wire = {
    v: 1,
    r: fixture.variant,
    m: Buffer.from(fixture.moveCells).toString("base64url"),
    b: { k: "h", n: fixture.displayName },
    w: {
      k: "b",
      n: botName,
      e: "search_bot",
      v: 1,
      cv: 1,
      c: { mode: "preset", preset: "normal", version: 1 },
    },
  };
  const canonical = JSON.stringify(wire);
  const fingerprint = createHash("sha256")
    .update("gomoku2d-portable-replay-v1\0" + canonical)
    .digest("base64url");
  return {
    key: "gomoku2d.replay-library.v1",
    value: {
      schema_version: 1,
      owners: {
        guest: {
          dirty: false,
          entries: [{
            fingerprint,
            payload: Buffer.from(canonical).toString("base64url"),
            kind: "saved",
            saved_at: fixture.savedAt,
          }],
        },
      },
    },
  };
}

/** @param {string} tag @param {string} profile @param {string} screen @param {string} state */
export function capturePathFor(tag, profile, screen, state) {
  const components = [tag, profile, screen + "--" + state];
  if (components.some((part) => !SAFE_ID.test(part))) throw new Error("Unsafe capture filename component.");
  return components.join("/") + ".png";
}
