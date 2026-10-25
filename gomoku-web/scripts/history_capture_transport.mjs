import { readFileSync, realpathSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { resolve } from "node:path";

const project = "demo-gomoku-history";
const authOrigin = "http://127.0.0.1:29099";
const firestoreOrigin = "http://127.0.0.1:28081";
const proxyOrigin = "http://127.0.0.1:29444";
const apiKey = "history-key";
/** @typedef {{mode:'legacy-proxy'|'native-emulators',ownerRoot:string,session:'guest'|'account'}} HistoryTransport */
/** @param {unknown} raw @returns {HistoryTransport} */
export function validateHistoryTransport(raw) {
  if (!raw || typeof raw !== "object" || Array.isArray(raw))
    throw new Error("Expected explicit owned history transport config");
  const value = /** @type {Record<string,any>} */ (raw);
  if (
    Object.keys(value).some(
      (key) => !["mode", "ownerRoot", "session"].includes(key),
    ) ||
    !["legacy-proxy", "native-emulators"].includes(value.mode) ||
    !["guest", "account"].includes(value.session) ||
    typeof value.ownerRoot !== "string" ||
    !value.ownerRoot.startsWith("/tmp/gomoku2d-history-")
  )
    throw new Error("Unsupported history transport mode/session/root");
  const ownerRoot = realpathSync(value.ownerRoot);
  const marker = JSON.parse(
    readFileSync(resolve(ownerRoot, "owner.json"), "utf8"),
  );
  if (
    marker.owner !== "gomoku-history-implementation" ||
    marker.root !== ownerRoot ||
    marker.repo !== resolve(import.meta.dirname, "../..")
  )
    throw new Error("History transport does not own this source root");
  return { mode: value.mode, ownerRoot, session: value.session };
}
/** @param {unknown} raw */
export function historyTransportLaunchOptions(raw) {
  const config = validateHistoryTransport(raw);
  return config.mode === "legacy-proxy"
    ? { proxy: { server: proxyOrigin, bypass: "127.0.0.1,localhost" } }
    : {};
}
/** @param {unknown} raw */
export function historyTransportContextOptions(raw) {
  return {
    ignoreHTTPSErrors: validateHistoryTransport(raw).mode === "legacy-proxy",
  };
}
/** @param {string} value @param {unknown} raw */
export function isHistoryTransportURL(value, raw) {
  const config = validateHistoryTransport(raw);
  let url;
  try {
    url = new URL(value);
  } catch {
    return false;
  }
  if (
    url.searchParams.has("database") &&
    url.searchParams.get("database") !==
      `projects/${project}/databases/(default)`
  )
    return false;
  if (config.mode === "native-emulators")
    return (
      [authOrigin, firestoreOrigin].includes(url.origin) &&
      (!url.pathname.startsWith("/v1/projects/") ||
        url.pathname.startsWith(`/v1/projects/${project}/`))
    );
  if (url.protocol !== "https:" || (url.port && url.port !== "443"))
    return false;
  if (url.hostname === "history-capture.invalid")
    return url.pathname === "/__history_transport_probe__";
  return url.hostname === "identitytoolkit.googleapis.com"
    ? url.pathname.startsWith("/v1/accounts:")
    : url.hostname === "securetoken.googleapis.com"
      ? url.pathname === "/v1/token"
      : url.hostname === "firestore.googleapis.com" &&
        (url.pathname.startsWith("/google.firestore.v1.Firestore/") ||
          url.pathname.startsWith("/v1/projects/demo-gomoku-history/"));
}

/** This unused hosted popup loader is intentionally unavailable in offline UI
 * captures. Keep the abort and attributed console/resource errors in evidence;
 * this does not qualify the external Google OAuth popup flow.
 * @param {string} value @param {unknown} raw
 */
export function isOptionalHistoryBootstrapURL(value, raw) {
  validateHistoryTransport(raw);
  let url;
  try {
    url = new URL(value);
  } catch {
    return false;
  }
  return (
    url.origin === "https://apis.google.com" &&
    url.pathname === "/js/api.js" &&
    [...url.searchParams.keys()].every((key) => key === "onload")
  );
}
/** @param {string} url @param {RequestInit} [options] */
async function jsonRequest(url, options) {
  if (![authOrigin, firestoreOrigin, proxyOrigin].includes(new URL(url).origin))
    throw new Error(
      "History fixture requests must stay on owned loopback services",
    );
  const response = await fetch(url, {
    ...options,
    signal: AbortSignal.timeout(15000),
  });
  if (!response.ok)
    throw new Error(
      `Owned history service returned ${response.status}: ${(await response.text()).slice(0, 300)}`,
    );
  return response.json();
}
/** @param {any} value @returns {any} */
function encodeValue(value) {
  if (value === null) return { nullValue: null };
  if (typeof value === "string") return { stringValue: value };
  if (typeof value === "boolean") return { booleanValue: value };
  if (typeof value === "number" && Number.isInteger(value))
    return { integerValue: String(value) };
  if (Array.isArray(value))
    return { arrayValue: { values: value.map(encodeValue) } };
  if (value && typeof value === "object") {
    if (typeof value.__timestampValue === "string")
      return { timestampValue: value.__timestampValue };
    return { mapValue: { fields: encodeFields(value) } };
  }
  throw new Error("Unsupported owned Firestore fixture value");
}
/** @param {Record<string,any>} value @returns {Record<string,any>} */
function encodeFields(value) {
  return Object.fromEntries(
    Object.entries(value).map(([key, entry]) => [key, encodeValue(entry)]),
  );
}
/** @param {string} path @param {Record<string,any>} fields */
async function seedDocument(path, fields) {
  await jsonRequest(
    `${firestoreOrigin}/v1/projects/${project}/databases/(default)/documents/${path.split("/").map(encodeURIComponent).join("/")}`,
    {
      method: "PATCH",
      headers: {
        Authorization: "Bearer owner",
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ fields: encodeFields(fields) }),
    },
  );
}
/** @param {string} uid @param {number} [number] */
function portablePayload(uid, number = 1) {
  const wire = {
    v: 1,
    r: "freestyle",
    m: Buffer.from([112, 111, 113, 0, 114, 15, 115, 1, 116]).toString(
      "base64url",
    ),
    b: { k: "h", n: "Reference Player", u: uid },
    w: { k: "h", n: `Opponent ${number}` },
  };
  const fingerprint = createHash("sha256")
    .update("gomoku2d-portable-replay-v1\0" + JSON.stringify(wire))
    .digest("base64url");
  return {
    fingerprint,
    payload: Buffer.from(JSON.stringify({ ...wire, at: 9 })).toString(
      "base64url",
    ),
  };
}
/** @param {string} uid */
async function seedPublicProfile(uid) {
  const updated = { __timestampValue: "2026-10-24T00:00:00.000Z" };
  await seedDocument(`public_profiles/${uid}`, {
    schema_version: 1,
    uid,
    display_name: "Reference Player",
    enabled: true,
    generation: 1,
    revision: 1,
    updated_at: updated,
    shared_stats: {
      matches: 20,
      wins: 12,
      losses: 6,
      draws: 2,
      updated_at: updated,
    },
    catalog_page_counts: [8, 1, 0, 0],
  });
  for (let page = 0; page < 2; page++) {
    const entries = Array.from({ length: page === 0 ? 8 : 1 }, (_, index) => {
      const number = page * 8 + index + 1;
      return {
        ...portablePayload(uid, number),
        title: `Freestyle win ${number}`,
        view_ply: 9,
      };
    });
    await seedDocument(`public_profiles/${uid}/pages/${page}`, {
      schema_version: 1,
      generation: 1,
      revision: 1,
      updated_at: updated,
      entries,
    });
  }
}

/** Prepare real synthetic service data and an ordinary persisted Auth session.
 * No application response or rendered UI is replaced.
 * @param {import('@playwright/test').BrowserContext} context
 * @param {Record<string,any>} source
 * @param {Record<string,any>|string} profile
 * @param {unknown} raw
 * @param {{screen?:string,state?:string}} [scene]
 */
export async function prepareHistoryContext(
  context,
  source,
  profile,
  raw,
  scene,
) {
  const config = validateHistoryTransport(raw);
  if (
    source.buildReceipt?.environment?.firebaseEnvironment
      ?.VITE_FIREBASE_PROJECT_ID !== project ||
    source.buildReceipt?.environment?.firebaseEnvironment
      ?.VITE_FIREBASE_API_KEY !== apiKey
  )
    throw new Error(
      "Historical cloud capture needs the explicitly synthetic demo build",
    );
  if (config.mode === "legacy-proxy") {
    const health = await jsonRequest(
      proxyOrigin + "/__history_transport_health__",
    );
    if (
      health.ownerRoot !== config.ownerRoot ||
      health.project !== project ||
      health.authPort !== 29099 ||
      health.firestorePort !== 28081
    )
      throw new Error(
        "Historical proxy does not belong to this source/capture goal",
      );
    // Verify the actual browser proxy before any Google-style SDK URL is used.
    // This reserved .invalid name has no public service if proxy options were
    // accidentally omitted. Only the owned CONNECT/TLS server can attest it.
    const probe = await context.newPage();
    try {
      const response = await probe.goto(
        "https://history-capture.invalid/__history_transport_probe__",
        { timeout: 10000 },
      );
      const proof = await response?.json();
      if (
        proof?.ownerRoot !== config.ownerRoot ||
        proof?.project !== project ||
        proof?.route !== "loopback-only"
      )
        throw new Error("Browser did not reach the owned history proxy");
    } finally {
      await probe.close();
    }
  }
  const rules = await readFile(
    resolve(source.worktree, "firestore.rules"),
    "utf8",
  );
  await jsonRequest(
    `${firestoreOrigin}/emulator/v1/projects/${project}:securityRules`,
    {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        rules: { files: [{ name: "firestore.rules", content: rules }] },
      }),
    },
  );
  const profileName = typeof profile === "string" ? profile : profile.id;
  const key =
    `${source.tag}-${profileName}-${scene?.screen ?? "scene"}-${scene?.state ?? "default"}`.replace(
      /[^A-Za-z0-9_-]/g,
      "-",
    );
  let uid = `history-public-${source.tag.replace(/[^A-Za-z0-9]/g, "")}`;
  if (config.session === "account") {
    const claims = {
      sub: key,
      email: `${key}@example.invalid`,
      email_verified: true,
      name: "Reference Player",
    };
    const auth = await jsonRequest(
      `${authOrigin}/identitytoolkit.googleapis.com/v1/accounts:signInWithIdp?key=${apiKey}`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          requestUri: "http://127.0.0.1",
          postBody: new URLSearchParams({
            providerId: "google.com",
            id_token: JSON.stringify(claims),
          }).toString(),
          returnSecureToken: true,
        }),
      },
    );
    uid = auth.localId;
    const user = {
      uid,
      email: auth.email,
      emailVerified: true,
      displayName: "Reference Player",
      isAnonymous: false,
      providerData: [
        {
          providerId: "google.com",
          uid: key,
          displayName: "Reference Player",
          email: auth.email,
          phoneNumber: null,
          photoURL: null,
        },
      ],
      stsTokenManager: {
        refreshToken: auth.refreshToken,
        accessToken: auth.idToken,
        expirationTime: Date.now() + Number(auth.expiresIn) * 1000,
      },
      createdAt: String(Date.now()),
      lastLoginAt: String(Date.now()),
      apiKey,
      appName: "[DEFAULT]",
    };
    const origin = new URL(source.baseUrl).origin;
    await context.addInitScript(
      ({ origin, user }) => {
        if (location.origin === origin)
          localStorage.setItem(
            "firebase:authUser:history-key:[DEFAULT]",
            JSON.stringify(user),
          );
      },
      { origin, user },
    );
  }
  const isPublicEra = /^v0\.6\./.test(source.tag);
  if (isPublicEra) await seedPublicProfile(uid);
  const payload = portablePayload(uid).payload;
  return {
    project,
    session: config.session,
    uid: config.session === "account" ? uid : null,
    publicUid: isPublicEra ? uid : null,
    sharedReplayFragment: "#g2r1=" + payload,
    sharedReplayUrl: "/replay/shared/#g2r1=" + payload,
    rulesSha256: createHash("sha256").update(rules).digest("hex"),
    transport: config.mode,
    responseSource:
      "owned Auth/Firestore emulators; tagged security rules; no fabricated SDK responses",
    publicFixture: isPublicEra
      ? "Reference Player; nine published games; dated synthetic 20/12/6/2 statistics"
      : null,
    optionalBootstrap: {
      policy: "blocked",
      resource: "https://apis.google.com/js/api.js",
      reason:
        "Hosted popup bootstrap is not needed for the restored synthetic session or public view. Aborts and attributed resource errors are retained; Google OAuth popup-flow acceptance is not established.",
    },
  };
}
