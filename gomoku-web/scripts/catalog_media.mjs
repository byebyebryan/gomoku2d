import { createHash } from "node:crypto";
import { readFile, realpath } from "node:fs/promises";
import { resolve, sep } from "node:path";

/**
 * Resolve and verify catalog PNG media against its raw-byte digest and IHDR dimensions.
 * @param {string} repositoryRoot
 * @param {{ sourcePath: string, sha256: string, width: number, height: number }} media
 * @param {{ outsideRepository: string, identityMismatch: string }} errors
 */
export async function readRepositoryPngIdentity(repositoryRoot, media, errors) {
  const absolute = await realpath(resolve(repositoryRoot, media.sourcePath));
  if (!absolute.startsWith(repositoryRoot + sep)) {
    throw new Error(errors.outsideRepository);
  }

  const bytes = await readFile(absolute);
  const actualSha256 = createHash("sha256").update(bytes).digest("hex");
  if (
    actualSha256 !== media.sha256 ||
    bytes.length < 24 ||
    bytes.subarray(0, 8).toString("hex") !== "89504e470d0a1a0a" ||
    bytes.readUInt32BE(16) !== media.width ||
    bytes.readUInt32BE(20) !== media.height
  ) {
    throw new Error(errors.identityMismatch);
  }

  return absolute;
}
