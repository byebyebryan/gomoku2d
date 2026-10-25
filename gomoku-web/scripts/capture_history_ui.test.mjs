import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, mkdir, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  cssViewportDimensions,
  isAllowedRequest,
  labControlForState,
  isOptionalBootstrapURL,
  isExpectedFirestoreChannelCancellation,
  isProfileResetConfirmation,
  isSettledReplayAnalysisStatus,
  parseArguments,
  PROFILE_NAME_INPUT_SELECTOR,
  PROFILE_NAME_LABEL_PATTERN,
  SETTINGS_CUSTOM_BUTTON_NAME,
  redactText,
  validateBotSearchHeaders,
  validateFontPreviewEvidence,
  validateFramingRect,
  validateGuideExamplesEvidence,
  validateLabAnalysisEvidence,
  validateManifestForWrite,
  validateReactVisualEvidence,
  validateReturnToPublicProfileControl,
  validateSharedReplayContext,
  validateOutputPath,
  visualsControlForState,
} from "./capture_history_ui.mjs";
import {
  CAPTURE_PROFILES,
  FIXTURES,
  TAG_ADAPTERS,
  localStorageSeed,
  routeForScene,
  validateHistoryCaptureConfig,
} from "./history_capture_adapters.mjs";

const HASH = "a".repeat(64);
const TREE = "b".repeat(40);

/** @param {string} tag @param {string} [firebaseMode] */
function receiptFor(tag, firebaseMode = "none") {
  const adapter = TAG_ADAPTERS[tag];
  return {
    schemaVersion: 1,
    tag,
    recordedAt: "2026-10-25T00:00:00.000Z",
    sourceCommit: adapter.commit,
    sourceTree: TREE,
    applicationTree: TREE,
    packageLockSha256: HASH,
    cargoLock: {
      kind: ["v0.2.1", "v0.2.2", "v0.2.3"].includes(tag) ? "generated-external" : "tracked",
      sha256: HASH,
    },
    wasmSha256: HASH,
    distSha256: HASH,
    indexSha256: HASH,
    environment: {
      node: "v26.12.1",
      npm: "12.2.1",
      rustc: "1.99.1",
      cargo: "1.99.1",
      wasmPack: "0.14.0",
      firebaseMode,
    },
    files: [{ path: "index.html", sha256: HASH }],
    reportInputs: [],
  };
}

/** @param {string} [tag] @param {Record<string, any>} [options] @returns {Record<string, any>} */
function configFor(tag = "v0.2.4", options = {}) {
  const adapter = TAG_ADAPTERS[tag];
  const firebaseMode = options.firebaseMode ?? "none";
  return {
    schemaVersion: 1,
    source: {
      tag,
      expectedCommit: adapter.commit,
      worktree: "/tmp/tag-source-" + tag,
      baseUrl: options.baseUrl ?? "http://127.0.0.1:46924/",
      adapter: adapter.adapter,
      buildReceipt: receiptFor(tag, firebaseMode),
    },
    profiles: ["desktop"],
    fixture: "missed_defense_closed_four",
    scenes: [{ screen: "home" }],
    ...(options.transport ? { transport: options.transport } : {}),
    ...(options.scenes ? { scenes: options.scenes } : {}),
  };
}

test("registry covers the audited tags and both fixed capture profiles", () => {
  assert.equal(Object.keys(TAG_ADAPTERS).length, 25);
  assert.deepEqual(CAPTURE_PROFILES.desktop.viewport, { width: 1280, height: 900 });
  assert.deepEqual(CAPTURE_PROFILES.portrait.viewport, { width: 412, height: 800 });
  assert.match(CAPTURE_PROFILES.portrait.userAgentKind, /not a measured OnePlus/);
});

test("configured CSS profile uses the visible visual viewport and retains layout fallback", () => {
  assert.deepEqual(cssViewportDimensions({
    visualViewportWidth: 412,
    visualViewportHeight: 800,
    innerWidth: 536,
    innerHeight: 1041,
  }), { width: 412, height: 800, basis: "visualViewport" });
  assert.deepEqual(cssViewportDimensions({
    visualViewportWidth: null,
    visualViewportHeight: null,
    innerWidth: 1280,
    innerHeight: 900,
  }), { width: 1280, height: 900, basis: "innerWidth fallback" });
  assert.throws(() => cssViewportDimensions({ innerWidth: 0, innerHeight: 0 }), /usable CSS viewport/);
});

test("capture framing requires an in-viewport anchor and validates requested top alignment", () => {
  const viewport = { width: 412, height: 800 };
  assert.deepEqual(validateFramingRect({ left: 8, top: 12, right: 404, bottom: 160, width: 396, height: 148 }, viewport, "Custom card", true), {
    visible: true,
    fullyVisible: true,
    topAligned: true,
  });
  assert.deepEqual(validateFramingRect({ left: 8, top: 96, right: 404, bottom: 180, width: 396, height: 84 }, viewport, "Advanced Controls"), {
    visible: true,
    fullyVisible: true,
    topAligned: false,
  });
  assert.throws(() => validateFramingRect({ left: 8, top: 804, right: 404, bottom: 900, width: 396, height: 96 }, viewport, "Proof frame"), /outside the visible visual viewport/);
  assert.throws(() => validateFramingRect({ left: 8, top: 40, right: 404, bottom: 188, width: 396, height: 148 }, viewport, "Custom card", true), /did not align near the visual viewport top/);
  assert.throws(() => validateFramingRect({ left: 8, top: 12, right: 420, bottom: 160, width: 412, height: 148 }, viewport, "Custom card", false, true), /clipped by the visible visual viewport/);
});

test("Guide Examples readiness requires both tagged replay rows and real shared-replay links", () => {
  assert.deepEqual(validateGuideExamplesEvidence({
    heading: "Example replays",
    entries: [
      { title: "Opening study", href: "/replay/shared/#g2r1=opening" },
      { title: "Forced corridor", href: "/replay/shared/#g2r1=corridor" },
    ],
  }), {
    heading: "Example replays",
    rowCount: 2,
    linkCount: 2,
    entries: [
      { title: "Opening study", href: "/replay/shared/#g2r1=opening" },
      { title: "Forced corridor", href: "/replay/shared/#g2r1=corridor" },
    ],
  });
  assert.throws(() => validateGuideExamplesEvidence({ heading: "Guide", entries: [] }), /section heading/);
  assert.throws(() => validateGuideExamplesEvidence({ heading: "Example replays", entries: [{ title: "Only one", href: "/replay/shared/#g2r1=x" }] }), /two tagged replay example rows/);
  assert.throws(() => validateGuideExamplesEvidence({
    heading: "Example replays",
    entries: [
      { title: "Opening study", href: "https://example.invalid/" },
      { title: "Forced corridor", href: "/replay/shared/#g2r1=corridor" },
    ],
  }), /real shared-replay link/);
});

test("source config requires its exact audited tag commit and adapter", () => {
  const valid = configFor();
  assert.equal(validateHistoryCaptureConfig(valid).source.tag, "v0.2.4");

  const wrongCommit = configFor();
  wrongCommit.source.expectedCommit = "b".repeat(40);
  assert.throws(() => validateHistoryCaptureConfig(wrongCommit), /audited tag commit/);

  const wrongAdapter = configFor();
  wrongAdapter.source.adapter = "guest-v2";
  assert.throws(() => validateHistoryCaptureConfig(wrongAdapter), /requires adapter/);

  const wrongTreeWidth = configFor();
  wrongTreeWidth.source.buildReceipt.sourceTree = HASH;
  assert.throws(() => validateHistoryCaptureConfig(wrongTreeWidth), /40-character Git tree OID/);

  const wrongApplicationTreeWidth = configFor();
  wrongApplicationTreeWidth.source.buildReceipt.applicationTree = HASH;
  assert.throws(() => validateHistoryCaptureConfig(wrongApplicationTreeWidth), /40-character Git tree OID/);
});

test("source config rejects unsafe previews, reserved ports, and unavailable screens or states", () => {
  assert.throws(() => validateHistoryCaptureConfig(configFor("v0.2.4", { baseUrl: "https://127.0.0.1:46924/" })), /root HTTP loopback/);
  assert.throws(() => validateHistoryCaptureConfig(configFor("v0.2.4", { baseUrl: "http://127.0.0.1:8001/" })), /reserved or invalid port/);
  assert.throws(() => validateHistoryCaptureConfig(configFor("v0.2.1", { scenes: [{ screen: "settings" }] })), /does not contain the settings screen/);
  assert.throws(() => validateHistoryCaptureConfig(configFor("v0.2.4", { scenes: [{ screen: "profile", state: "account" }] })), /predates cloud account state/);
  assert.throws(() => validateHistoryCaptureConfig(configFor("v0.4.0", { scenes: [{ screen: "bot-report", state: "search" }] })), /without Search or Pairwise controls/);
  assert.throws(() => validateHistoryCaptureConfig(configFor("v0.4.0", { scenes: [{ screen: "bot-report", state: "pairwise" }] })), /without Search or Pairwise controls/);
  assert.throws(() => validateHistoryCaptureConfig(configFor("v0.5.2", { scenes: [{ screen: "profile", state: "empty" }] })), /does not contain the tagged Profile empty-history state/);
  assert.equal(validateHistoryCaptureConfig(configFor("v0.5.3", { scenes: [{ screen: "profile", state: "empty" }] })).scenes[0].state, "empty");
  assert.equal(validateHistoryCaptureConfig(configFor("v0.5.4", { scenes: [{ screen: "profile", state: "empty" }] })).scenes[0].state, "empty");
});

test("Firebase builds require matching owned synthetic transport mode and session", () => {
  const raw = configFor("v0.3.0", { firebaseMode: "legacy-proxy" });
  assert.throws(() => validateHistoryCaptureConfig(raw), /provide the approved synthetic-demo transport config/);

  raw.transport = { mode: "legacy-proxy", ownerRoot: "/tmp/gomoku2d-history-unit", session: "guest" };
  assert.equal(validateHistoryCaptureConfig(raw).transport.mode, "legacy-proxy");
  raw.transport.mode = "native-emulators";
  assert.throws(() => validateHistoryCaptureConfig(raw), /must match the tagged Firebase build mode/);

  const noFirebase = configFor("v0.2.4", {
    transport: { mode: "legacy-proxy", ownerRoot: "/tmp/gomoku2d-history-unit", session: "guest" },
  });
  assert.throws(() => validateHistoryCaptureConfig(noFirebase), /no-Firebase tagged build cannot use/);

  const accountScene = configFor("v0.3.0", {
    firebaseMode: "legacy-proxy",
    scenes: [{ screen: "profile", state: "account" }],
    transport: { mode: "legacy-proxy", ownerRoot: "/tmp/gomoku2d-history-unit", session: "account" },
  });
  assert.equal(validateHistoryCaptureConfig(accountScene).scenes[0].requiresTransport, true);
});

test("report inputs are source-receipted before report scenes are accepted", () => {
  const config = configFor("v0.4.3", {
    scenes: [
      { screen: "bot-report", state: "search" },
      { screen: "analysis-report", state: "default" },
      { screen: "analysis-report", state: "proof-open" },
    ],
  });
  assert.throws(() => validateHistoryCaptureConfig(config), /report receipt is missing tagged input/);
  config.source.buildReceipt.reportInputs = [
    { path: "gomoku-bot-lab/reports/index.html", sha256: HASH },
    { path: "gomoku-bot-lab/reports/latest.json", sha256: HASH },
    { path: "gomoku-bot-lab/analysis-reports/index.html", sha256: HASH },
    { path: "gomoku-bot-lab/analysis-reports/latest.json", sha256: HASH },
  ];
  assert.equal(validateHistoryCaptureConfig(config).expectedReportInputs.length, 4);
});

test("tag storage adapters preserve their actual guest and local schemas", () => {
  const early = localStorageSeed("v0.2.1");
  assert.equal(early.key, "gomoku2d.guest-profile.v1");
  assert.equal(early.value.state.history[0].moves.length, 9);
  assert.equal(early.value.state.profile.displayName, FIXTURES.missed_defense_closed_four.displayName);
  assert.equal(early.value.state.history[0].players[1].name, "Classic Bot");

  const pilot = localStorageSeed("v0.2.4");
  assert.equal(pilot.value.state.history[0].players[1].name, "Practice Bot");

  const promoted = localStorageSeed("v0.3.1");
  assert.equal(promoted.key, "gomoku2d.guest-profile.v2");
  assert.equal(promoted.value.state.history[0].schema_version, 1);

  const local = localStorageSeed("v0.4.5");
  const match = local.value.state.matchHistory.replayMatches[0];
  assert.equal(local.key, "gomoku2d.local-profile.v5");
  assert.equal(match.schema_version, 2);
  assert.equal(match.ruleset, "freestyle");
  assert.equal(Object.hasOwn(match, "variant"), false);
  assert.equal(match.player_white.display_name, "Normal Bot");

  const emptyLocal = localStorageSeed("v0.5.4", "missed_defense_closed_four", { emptyHistory: true });
  assert.equal(emptyLocal.key, "gomoku2d.local-profile.v5");
  assert.equal(emptyLocal.value.state.profile.displayName, FIXTURES.missed_defense_closed_four.displayName);
  assert.deepEqual(emptyLocal.value.state.matchHistory.replayMatches, []);
  assert.deepEqual(emptyLocal.value.state.matchHistory.summaryMatches, []);
  assert.throws(() => localStorageSeed("v0.5.5", "missed_defense_closed_four", { emptyHistory: true }), /does not support the tagged empty local-history fixture/);

  const recent = localStorageSeed("v0.6.2");
  const library = recent.additional[0];
  assert.equal(library.key, "gomoku2d.replay-library.v1");
  assert.equal(library.value.schema_version, 1);
  const [entry] = library.value.owners.guest.entries;
  assert.match(entry.fingerprint, /^[A-Za-z0-9_-]{43}$/);
  assert.equal(entry.kind, "saved");
  const wire = Buffer.from(entry.payload, "base64url").toString("utf8");
  assert.equal(JSON.parse(wire).m, Buffer.from(FIXTURES.missed_defense_closed_four.moveCells).toString("base64url"));
  assert.equal(entry.fingerprint, createHash("sha256").update("gomoku2d-portable-replay-v1\0" + wire).digest("base64url"));
});

test("tag routing preserves the replay path rename and historical Visuals pages", () => {
  assert.equal(routeForScene("v0.3.2", "replay"), "/replays/local/history-fixture-match");
  assert.equal(routeForScene("v0.3.3", "replay"), "/replay/history-fixture-match");
  assert.equal(routeForScene("v0.2.4", "visuals", undefined, "sprites"), "/assets/sprites/preview.html");
  assert.equal(routeForScene("v0.5.2", "visuals", undefined, "default"), "/visuals/");
  assert.equal(validateHistoryCaptureConfig(configFor("v0.6.2", {
    firebaseMode: "modern-emulators",
    transport: { mode: "native-emulators", ownerRoot: "/tmp/gomoku2d-history-unit", session: "account" },
    scenes: [{ screen: "profile", state: "saved-replays" }, { screen: "replay", state: "share" }, { screen: "profile", state: "public-settings" }],
  })).scenes.length, 3);
  assert.equal(validateHistoryCaptureConfig(configFor("v0.6.2", {
    firebaseMode: "modern-emulators",
    transport: { mode: "native-emulators", ownerRoot: "/tmp/gomoku2d-history-unit", session: "guest" },
    scenes: [{ screen: "shared-replay", state: "from-profile" }],
  })).scenes[0].state, "from-profile");
  assert.equal(validateHistoryCaptureConfig(configFor("v0.5.1", {
    scenes: [{ screen: "visuals", state: "fonts" }],
  })).scenes[0].state, "fonts");
  assert.throws(() => validateHistoryCaptureConfig(configFor("v0.6.2", {
    firebaseMode: "modern-emulators",
    transport: { mode: "native-emulators", ownerRoot: "/tmp/gomoku2d-history-unit", session: "account" },
    scenes: [{ screen: "profile", state: "share" }],
  })), /Unsupported state profile\/share/);
  assert.throws(() => validateHistoryCaptureConfig(configFor("v0.6.2", {
    firebaseMode: "modern-emulators",
    transport: { mode: "native-emulators", ownerRoot: "/tmp/gomoku2d-history-unit", session: "guest" },
    scenes: [{ screen: "replay", state: "share" }],
  })), /Account transition scenes require transport.session account/);
  assert.throws(() => validateHistoryCaptureConfig(configFor("v0.6.1", {
    firebaseMode: "modern-emulators",
    transport: { mode: "native-emulators", ownerRoot: "/tmp/gomoku2d-history-unit", session: "guest" },
    scenes: [{ screen: "shared-replay", state: "from-profile" }],
  })), /predates the Shared Replay return-to-profile state/);
  assert.throws(() => validateHistoryCaptureConfig(configFor("v0.6.2", {
    firebaseMode: "modern-emulators",
    transport: { mode: "native-emulators", ownerRoot: "/tmp/gomoku2d-history-unit", session: "account" },
    scenes: [{ screen: "shared-replay", state: "default" }],
  })), /require transport.session guest/);
});

test("network policy allows only tagged loopback reads or explicitly approved transport URLs", () => {
  const origin = "http://127.0.0.1:46924";
  assert.equal(isAllowedRequest(origin + "/assets/app.js", "GET", origin), true);
  assert.equal(isAllowedRequest(origin + "/api/profile", "GET", origin), false);
  assert.equal(isAllowedRequest(origin + "/assets/app.js", "POST", origin), false);
  assert.equal(isAllowedRequest("https://example.invalid/", "GET", origin), false);
  assert.equal(isAllowedRequest("https://identitytoolkit.googleapis.com/v1/accounts:lookup", "POST", origin), false);
  assert.equal(isAllowedRequest("https://identitytoolkit.googleapis.com/v1/accounts:lookup", "POST", origin, true), true);
});

test("only the two approved native Firestore channel GETs can classify a post-200 ERR_ABORTED as expected", () => {
  const accepted = {
    url: "http://127.0.0.1:28081/google.firestore.v1.Firestore/Write/channel?database=projects%2Fdemo-gomoku-history%2Fdatabases%2F(default)",
    method: "GET",
    resourceType: "fetch",
    failure: "net::ERR_ABORTED",
    responseStatus: 200,
  };
  assert.equal(isExpectedFirestoreChannelCancellation(accepted, "native-emulators", true), true);
  const acceptedListen = {
    ...accepted,
    url: "http://127.0.0.1:28081/google.firestore.v1.Firestore/Listen/channel?database=projects%2Fdemo-gomoku-history%2Fdatabases%2F(default)",
  };
  assert.equal(isExpectedFirestoreChannelCancellation(acceptedListen, "native-emulators", true), true);

  const rejected = [
    { ...accepted, url: "http://127.0.0.1:28082/google.firestore.v1.Firestore/Write/channel" },
    { ...accepted, url: "https://firestore.googleapis.com/google.firestore.v1.Firestore/Write/channel" },
    { ...accepted, url: "http://127.0.0.1:28081/google.firestore.v1.Firestore/BatchWrite/channel" },
    { ...acceptedListen, url: "http://127.0.0.1:28081/google.firestore.v1.Firestore/Write/channel/extra" },
    { ...accepted, method: "POST" },
    { ...accepted, resourceType: "xhr" },
    { ...accepted, failure: "net::ERR_FAILED" },
    { ...accepted, responseStatus: null },
    { ...accepted, responseStatus: 204 },
  ];
  for (const event of rejected) assert.equal(isExpectedFirestoreChannelCancellation(event, "native-emulators", true), false);
  for (const event of [accepted, acceptedListen]) {
    assert.equal(isExpectedFirestoreChannelCancellation(event, "legacy-proxy", true), false);
    assert.equal(isExpectedFirestoreChannelCancellation(event, "native-emulators", false), false);
  }
});

test("optional bootstrap classification delegates exact URL policy to the transport helper", () => {
  const expected = "https://apis.google.com/js/api.js?onload=initPopup";
  /** @type {Array<[string, any]>} */
  const seen = [];
  const helper = {
    /** @param {string} url @param {any} normalized */
    isOptionalHistoryBootstrapURL(url, normalized) {
      seen.push([url, normalized]);
      return url === expected && normalized.mode === "legacy-proxy";
    },
  };
  const normalized = { mode: "legacy-proxy" };
  assert.equal(isOptionalBootstrapURL(expected, helper, normalized), true);
  assert.equal(isOptionalBootstrapURL("https://apis.google.com/js/other.js", helper, normalized), false);
  assert.equal(isOptionalBootstrapURL(expected, undefined, normalized), false);
  assert.deepEqual(seen, [[expected, normalized], ["https://apis.google.com/js/other.js", normalized]]);
});

test("Bot Report Search readiness follows the exact tagged header revisions", () => {
  const v042 = ["Avg nodes", "Eval", "Cand gen", "Breadth", "Legal", "TT hit/cut"];
  const v043 = [...v042.slice(0, 5), "Portal", "Exit", "TT hit/cut"];
  const v044 = ["Nodes", "Move gen", "Ordering", "Scoring", "Threat detection", "Proof", "Other", "TT", "Pool"];
  assert.deepEqual(validateBotSearchHeaders("v0.4.2", v042), v042);
  assert.deepEqual(validateBotSearchHeaders("v0.4.3", v043), v043);
  assert.deepEqual(validateBotSearchHeaders("v0.4.4", v044), v044);
  assert.throws(() => validateBotSearchHeaders("v0.4.4", v043), /missing Nodes, Move gen/);
});

test("Profile reset readiness accepts the tagged copy without activating confirmation", () => {
  assert.equal(isProfileResetConfirmation("This clears cloud history, resets cloud profile values, and clears this device's local cache."), true);
  assert.equal(isProfileResetConfirmation("Reset cloud profile and clear both cloud and local match history?"), true);
  assert.equal(isProfileResetConfirmation("Reset profile data here and in the cloud? This clears settings, games, and local replay analyses."), true);
  assert.equal(isProfileResetConfirmation("Profile controls are ready."), false);
});

test("Profile identity readiness supports the tagged Name and Display name inputs", () => {
  assert.match(PROFILE_NAME_INPUT_SELECTOR, /placeholder="Name"/);
  assert.match(PROFILE_NAME_INPUT_SELECTOR, /placeholder="Display name"/);
  assert.equal(PROFILE_NAME_LABEL_PATTERN.test("Name"), true);
  assert.equal(PROFILE_NAME_LABEL_PATTERN.test("Display name"), true);
  assert.equal(PROFILE_NAME_LABEL_PATTERN.test("Default rule"), false);
});

test("Settings Custom readiness matches the visible label with its tagged detail text", () => {
  assert.match("Custom Tune the search knobs", SETTINGS_CUSTOM_BUTTON_NAME);
  assert.doesNotMatch("Customary preset", SETTINGS_CUSTOM_BUTTON_NAME);
});

test("Replay analysis readiness accepts settled hidden-layout text and rejects transient labels", () => {
  assert.equal(isSettledReplayAnalysisStatus("Black won"), true);
  assert.equal(isSettledReplayAnalysisStatus("2-ply forced corridor"), true);
  assert.equal(isSettledReplayAnalysisStatus("Analyzing replay"), false);
  assert.equal(isSettledReplayAnalysisStatus("   "), false);
});

test("React Visuals states use tagged controls and semantic specimen evidence", () => {
  assert.deepEqual(visualsControlForState("v0.5.1", "fonts"), { role: "button", label: "Guide" });
  assert.deepEqual(visualsControlForState("v0.5.1", "sprites"), { role: "button", label: "Sprites" });
  assert.deepEqual(visualsControlForState("v0.5.2", "fonts"), { role: "tab", label: "Style" });
  assert.deepEqual(visualsControlForState("v0.6.2", "icons"), { role: "tab", label: "Icons" });
  assert.equal(visualsControlForState("v0.5.0", "sprites"), null);

  assert.equal(validateReactVisualEvidence("sprites", {
    heading: "SPRITES", supportingHeading: "SOURCE SHEETS", specimenCount: 3, pageTab: "sprites",
  }).specimenCount, 3);
  assert.throws(() => validateReactVisualEvidence("icons", {
    heading: "ICONS", supportingHeading: "EXPORTED SVGS", specimenCount: 0,
  }), /no rendered tagged specimens/);
  assert.deepEqual(validateReactVisualEvidence("fonts", {
    heading: "TYPOGRAPHY",
    specimenCount: 2,
    sampleTexts: ["Player name", "WINS"],
    pageTab: "guide typography section",
  }, { faces: [{ family: "PixelOperator", loaded: true, loadedFaceCount: 1 }] }).specimenCards, 2);
  assert.throws(() => validateReactVisualEvidence("fonts", {
    heading: "Typography", specimenCount: 1, sampleTexts: ["A"],
  }, { faces: [] }), /no loaded tagged PixelOperator font face/);
});

test("Lab control role follows the tagged Report tab revision", () => {
  assert.deepEqual(labControlForState("v0.5.1", "analysis"), { role: "button", label: "Analysis" });
  assert.deepEqual(labControlForState("v0.5.2", "search"), { role: "tab", label: "Search" });
  assert.deepEqual(labControlForState("v0.6.2", "analysis"), { role: "tab", label: "Analysis" });
  assert.equal(labControlForState("v0.5.0", "analysis"), null);
});

test("Lab Analysis readiness requires loaded results, notes, provenance, and report rows", () => {
  assert.deepEqual(validateLabAnalysisEvidence({
    headings: ["RESULTS", "LAB NOTES", "PROVENANCE"],
    resultRows: 2,
  }), {
    id: "analysis",
    routeState: "tab=analysis",
    resultRows: 2,
    headings: ["RESULTS", "LAB NOTES", "PROVENANCE"],
  });
  assert.throws(() => validateLabAnalysisEvidence({
    headings: ["RESULTS", "LAB NOTES", "PROVENANCE"],
    resultRows: 0,
  }), /no rendered analysis report rows/);
  assert.throws(() => validateLabAnalysisEvidence({
    headings: ["RESULTS", "PROVENANCE"],
    resultRows: 1,
  }), /missing its tagged Lab Notes section/);
});

test("Shared Replay readiness accepts the tagged compact-layout hidden context and records visibility", () => {
  assert.deepEqual(validateSharedReplayContext({
    attached: true, text: "Shared replay", visible: false,
  }), {
    label: "Shared replay",
    visible: false,
    visibilityNote: "attached but hidden by the tagged compact portrait layout",
  });
  assert.equal(validateSharedReplayContext({
    attached: true, text: "Shared replay", visible: true,
  }).visible, true);
  assert.throws(() => validateSharedReplayContext({
    attached: false, text: "Shared replay", visible: false,
  }), /missing its tagged Shared replay context label/);
});

test("Shared Replay from-profile readiness requires the tagged button control", () => {
  assert.deepEqual(validateReturnToPublicProfileControl({
    role: "button",
    label: "Return to public profile",
    visible: true,
  }), { role: "button", label: "Return to public profile", visible: true });
  assert.throws(() => validateReturnToPublicProfileControl({
    role: "link",
    label: "Return to public profile",
    visible: true,
  }), /visible tagged Return to public profile button/);
  assert.throws(() => validateReturnToPublicProfileControl({
    role: "button",
    label: "Return to public profile",
    visible: false,
  }), /visible tagged Return to public profile button/);
});

test("font preview evidence requires rendered specimens and loaded actual font faces", () => {
  const assetEvidence = {
    fontCardCount: 15,
    expectedFontFaceCount: 15,
    sampleText: "Practice Bot vs Local Player",
  };
  const fontReadiness = {
    faces: Array.from({ length: 15 }, (_, index) => ({
      family: "PixelOperator" + index,
      loaded: true,
      loadedFaceCount: 1,
    })),
  };
  const observed = validateFontPreviewEvidence(assetEvidence, fontReadiness);
  assert.equal(observed.specimenCards, 15);
  assert.equal(observed.loadedFontFaceCount, 15);
  assert.match(observed.presentation, /actual font faces/);
  assert.throws(() => validateFontPreviewEvidence({ ...assetEvidence, fontCardCount: 0 }, fontReadiness), /no rendered font specimen cards/);
  assert.throws(() => validateFontPreviewEvidence(assetEvidence, { faces: [] }), /loaded 0 of 15/);
});

test("error redaction hides free-form token values and URL query strings", () => {
  const output = redactText("token=secret-value at https://example.invalid/callback?access_token=secret");
  assert.doesNotMatch(output, /secret-value|access_token=secret/);
  assert.match(output, /token=\[redacted\]/);
  assert.match(output, /\?\[redacted\]/);
});

test("manifest validator requires safe unique screen/state/profile PNG entries", () => {
  const capture = {
    profile: { id: "desktop" },
    screen: "home",
    state: "default",
    status: "captured",
    file: "v0.2.4/desktop/home--default.png",
    sha256: HASH,
    width: 1280,
    height: 900,
  };
  const manifest = { schemaVersion: 1, sources: [{}], captures: [capture] };
  assert.equal(validateManifestForWrite(manifest), true);
  assert.throws(() => validateManifestForWrite({ ...manifest, captures: [capture, { ...capture }] }), /duplicate capture/);
  assert.throws(() => validateManifestForWrite({ ...manifest, captures: [{ ...capture, file: "../escape.png" }] }), /unsafe screenshot filename/);
  assert.throws(() => validateManifestForWrite({ ...manifest, captures: [{ ...capture, sha256: "bad" }] }), /SHA-256 hash/);
});

test("CLI parses required explicit config/output and rejects duplicate switches", () => {
  assert.deepEqual(parseArguments(["--config", "one.json", "--output", "/tmp/capture"]), {
    validateOnly: false,
    help: false,
    configPath: "one.json",
    outputPath: "/tmp/capture",
  });
  assert.throws(() => parseArguments(["--config", "one.json", "--config", "two.json", "--output", "/tmp/capture"]), /supplied only once/);
  assert.throws(() => parseArguments(["--config", "one.json"]), /Both --config and --output are required/);
});

test("output guards reject overwrite, source traversal, and symlinked parents", async (t) => {
  const root = await mkdtemp(join(tmpdir(), "history-capture-test-"));
  t.after(async () => rm(root, { recursive: true, force: true }));
  const protectedRoot = join(root, "source");
  await mkdir(protectedRoot);

  assert.equal(await validateOutputPath(join(root, "new-run"), [protectedRoot]), join(root, "new-run"));
  await assert.rejects(() => validateOutputPath(join(protectedRoot, "captures"), [protectedRoot]), /disjoint from protected/);

  const existing = join(root, "existing");
  await mkdir(existing);
  await assert.rejects(() => validateOutputPath(existing, []), /already exists/);

  const link = join(root, "source-link");
  await symlink(protectedRoot, link, "dir");
  await assert.rejects(() => validateOutputPath(join(link, "new-run"), []), /symlink component/);
  await writeFile(join(root, "marker"), "temporary test fixture\n");
});
