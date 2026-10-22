// An owned demo-only emulator and preview. No existing servers or credentials
// are adopted. A distinct bundle leaves the ordinary production dist intact.
import { spawn } from "node:child_process";
import { createServer } from "node:net";
import { mkdir, open } from "node:fs/promises";
import { resolve } from "node:path";
import { homedir } from "node:os";

const root = resolve(import.meta.dirname, "..");
process.chdir(root);
const ports = [8013, 19099, 18081, 19151, 14401, 14501];
const children = new Set();
let stopping = false;
function cleanup() {
  if (stopping) return;
  stopping = true;
  for (const child of children) {
    try { if (child.pid) process.kill(-child.pid, "SIGTERM"); } catch { /* already exited */ }
  }
}
process.on("SIGINT", () => { cleanup(); process.exitCode = 130; });
process.on("SIGTERM", () => { cleanup(); process.exitCode = 143; });
process.on("exit", cleanup);
const deadline = setTimeout(() => { console.error("Cloud acceptance exceeded 15 minutes"); cleanup(); process.exitCode = 1; }, 900_000);
deadline.unref();

/** @param {string[]} args @param {object} [options] */
function start(args, options = {}) {
  return startCommand(process.execPath, args, options);
}
/** @param {string} command @param {string[]} args @param {object} [options] */
function startCommand(command, args, options = {}) {
  const child = spawn(command, args, { cwd: root, detached: true, stdio: "inherit", ...options });
  children.add(child);
  return child;
}
/** @param {ReturnType<typeof start>} child */
function completed(child) {
  return new Promise((resolvePromise, reject) => {
    child.once("error", reject);
    child.once("exit", (code, signal) => code === 0 ? resolvePromise(undefined)
      : reject(new Error(`Cloud acceptance child failed (${code ?? signal})`)));
  });
}
/** @param {string} url @param {ReturnType<typeof start>} child */
async function ready(url, child) {
  const end = Date.now() + 90_000;
  while (Date.now() < end && !stopping && child.exitCode === null) {
    try { if ((await fetch(url, { signal: AbortSignal.timeout(1000) })).ok) return; } catch { /* booting */ }
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 200));
  }
  throw new Error(`Timed out starting ${url}`);
}
try {
  for (const port of ports) {
    await new Promise((resolvePromise, reject) => {
      const server = createServer(); server.once("error", reject);
      server.listen(port, "127.0.0.1", () => server.close(() => resolvePromise(undefined)));
    });
  }
  const env = { ...process.env,
    VITE_FIREBASE_API_KEY: "fake-key", VITE_FIREBASE_APP_ID: "fake-app",
    VITE_FIREBASE_AUTH_DOMAIN: "127.0.0.1", VITE_FIREBASE_MESSAGING_SENDER_ID: "123",
    VITE_FIREBASE_PROJECT_ID: "demo-gomoku2d-acceptance", VITE_FIREBASE_STORAGE_BUCKET: "unused",
    VITE_FIREBASE_EMULATORS: "1", VITE_FIREBASE_AUTH_EMULATOR_PORT: "19099",
    VITE_FIREBASE_FIRESTORE_EMULATOR_PORT: "18081", GOMOKU_BASE_PATH: "/",
  };
  await completed(start(["node_modules/vite/bin/vite.js", "build", "--outDir", "dist-cloud", "--logLevel", "error"], { env }));
  await mkdir("test-results/cloud-logs", { recursive: true });
  const log = await open("test-results/cloud-logs/emulators.log", "w");
  const emulator = start(["node_modules/firebase-tools/lib/bin/firebase.js", "emulators:start",
    "--project", "demo-gomoku2d-acceptance", "--only", "auth,firestore", "--config", "../firebase.acceptance.json"],
  { stdio: ["ignore", log.fd, log.fd] });
  await log.close();
  await ready("http://127.0.0.1:19099/", emulator);
  await ready("http://127.0.0.1:18081/", emulator);
  const preview = start(["node_modules/vite/bin/vite.js", "preview", "--outDir", "dist-cloud",
    "--host", "127.0.0.1", "--port", "8013", "--strictPort"]);
  await ready("http://127.0.0.1:8013/", preview);
  const inContainer = process.argv.includes("--container");
  const args = process.argv.slice(2).filter((arg) => arg !== "--container");
  const playwright = ["node_modules/@playwright/test/cli.js", "test", "--config", "playwright.cloud.config.ts",
    ...(args.some((arg) => arg.startsWith("--browser")) ? [] : ["--browser=all"]), ...args];
  if (inContainer) {
    // The host owns Java/emulators. Ubuntu supplies browser libraries; use the
    // host's installed browser cache so it matches the lockfile's Playwright.
    const browserCache = process.env.PLAYWRIGHT_BROWSERS_PATH ?? resolve(homedir(), ".cache/ms-playwright");
    await completed(startCommand("docker", ["run", "--rm", "--init", "--network", "host", "--ipc", "host",
      "--user", `${process.getuid?.() ?? 1000}:${process.getgid?.() ?? 1000}`, "-e", "CI=1",
      "-e", `GOMOKU_WRITE_EVIDENCE=${process.env.GOMOKU_WRITE_EVIDENCE === "1" ? "1" : "0"}`,
      "-v", `${resolve(root, "..")}:/workspace`, "-v", `${browserCache}:/ms-playwright:ro`,
      "-w", "/workspace/gomoku-web", "mcr.microsoft.com/playwright:v1.59.1-noble", "node", ...playwright]));
  } else await completed(start(playwright));
} catch (error) {
  console.error(error);
  process.exitCode = 1;
} finally { clearTimeout(deadline); cleanup(); }
