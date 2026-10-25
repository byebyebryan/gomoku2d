import { readFile, realpath } from "node:fs/promises";
import { createHash } from "node:crypto";
import { dirname, resolve, sep } from "node:path";
const { isHistoryCatalog } = await import(
  new URL("../src/developer/history_record.ts", import.meta.url).href
);
export const historyWebRoot = resolve(import.meta.dirname, "..");
export const historyRepoRoot = resolve(historyWebRoot, "..");
/** @typedef {import('../src/developer/history_record.ts').HistoryCatalog} HistoryCatalog */
/** @param {string} path */
async function repositoryFile(path) {
  const absolute = await realpath(resolve(historyRepoRoot, path));
  if (!absolute.startsWith(historyRepoRoot + sep))
    throw new Error("History evidence leaves repository: " + path);
  return { absolute, bytes: await readFile(absolute) };
}
/** @param {Buffer} bytes */
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

/** Read the allowlist and independently verify the source/manifest/image graph.
 * @returns {Promise<HistoryCatalog>}
 */
export async function readHistoryCatalog() {
  const value = JSON.parse(
    await readFile(
      resolve(historyWebRoot, "src/developer/history.json"),
      "utf8",
    ),
  );
  if (!isHistoryCatalog(value)) throw new Error("Invalid UI history catalog");
  const catalog = /** @type {HistoryCatalog} */ (value);
  const releases = new Map(catalog.releases.map((r) => [r.version, r]));
  const manifests = new Map();
  for (const snapshot of catalog.snapshots) {
    let manifest = manifests.get(snapshot.manifest.sourcePath);
    if (!manifest) {
      const source = await repositoryFile(snapshot.manifest.sourcePath);
      if (hash(source.bytes) !== snapshot.manifest.sha256)
        throw new Error(
          "History manifest hash mismatch: " + snapshot.manifest.sourcePath,
        );
      manifest = {
        path: source.absolute,
        value: JSON.parse(source.bytes.toString("utf8")),
        sha256: snapshot.manifest.sha256,
      };
      if (
        manifest.value.status !== "complete" ||
        !Array.isArray(manifest.value.errors) ||
        manifest.value.errors.length !== 0 ||
        !Array.isArray(manifest.value.sources) ||
        !Array.isArray(manifest.value.captures)
      )
        throw new Error(
          "History manifest is not a complete capture: " +
            snapshot.manifest.sourcePath,
        );
      manifests.set(snapshot.manifest.sourcePath, manifest);
    }
    if (manifest.sha256 !== snapshot.manifest.sha256)
      throw new Error("Conflicting history manifest identities");
    const release = releases.get(snapshot.version);
    if (
      !release ||
      !manifest.value.sources.some(
        /** @param {any} source */ (source) =>
          source.tag === `v${release.version}` &&
          source.commit === release.commit,
      )
    )
      throw new Error(
        "History capture source does not match its release: " + snapshot.id,
      );
    for (const [layout, media] of Object.entries(snapshot.media)) {
      if (!media) continue;
      const source = await repositoryFile(media.sourcePath);
      if (
        hash(source.bytes) !== media.sha256 ||
        source.bytes.length < 24 ||
        source.bytes.subarray(0, 8).toString("hex") !== "89504e470d0a1a0a" ||
        source.bytes.readUInt32BE(16) !== media.width ||
        source.bytes.readUInt32BE(20) !== media.height
      )
        throw new Error("History PNG identity mismatch: " + media.sourcePath);
      const entry = manifest.value.captures.find(
        /** @param {any} capture */ (capture) =>
          (capture.source === `v${snapshot.version}` ||
            (capture.source === undefined &&
              manifest.value.sources.length === 1)) &&
          capture.screen === snapshot.screen &&
          (capture.state ?? "default") === snapshot.state &&
          (typeof capture.profile === "string"
            ? capture.profile
            : capture.profile?.id) === layout &&
          typeof capture.file === "string" &&
          resolve(dirname(manifest.path), capture.file) === source.absolute,
      );
      if (
        !entry ||
        entry.status !== "captured" ||
        ![
          entry.errors?.console,
          entry.errors?.page,
          entry.network?.forbidden,
          entry.network?.failed,
          entry.network?.consoleErrors,
          entry.network?.pageErrors,
        ].every((errors) => Array.isArray(errors) && errors.length === 0) ||
        entry.sha256 !== media.sha256 ||
        entry.width !== media.width ||
        entry.height !== media.height
      )
        throw new Error(
          "History PNG is not qualified by its capture manifest: " +
            media.sourcePath,
        );
    }
  }
  return catalog;
}
