import { doc, runTransaction, type Firestore } from "firebase/firestore";

import type { CloudAuthUser } from "./auth_store";
import { cloudProfileFromDocument, type CloudProfile, type CloudProfileDocument } from "./cloud_profile";

function snapshotState(profile: CloudProfile): string {
  // Server timestamps are not known exactly in a just-written client snapshot.
  // Compare every field a normal snapshot can replace, plus the reset barrier.
  return JSON.stringify({
    uid: profile.uid,
    displayName: profile.displayName,
    username: profile.username,
    resetAt: profile.resetAt,
    settings: profile.settings,
    matchHistory: profile.matchHistory,
  }, (_key, value: unknown) => {
    if (value === null || typeof value !== "object" || Array.isArray(value)) return value;
    const record = value as Record<string, unknown>;
    return Object.fromEntries(Object.keys(record).sort().map((key) => [key, record[key]]));
  });
}

export async function writeCloudSnapshot(
  firestore: Firestore,
  user: CloudAuthUser,
  expected: CloudProfile | undefined,
  patch: Record<string, unknown>,
): Promise<void> {
  if (!expected || expected.uid !== user.uid) throw new Error("Load the cloud profile before syncing.");
  const ref = doc(firestore, "profiles", user.uid);
  await runTransaction(firestore, async (transaction) => {
    const current = await transaction.get(ref);
    if (!current.exists() || snapshotState(cloudProfileFromDocument(
      user, expected.settings, current.data() as CloudProfileDocument,
    )) !== snapshotState(expected)) {
      throw new Error("Cloud profile changed on another device. Reload this page before syncing again.");
    }
    transaction.set(ref, patch, { merge: true });
  });
}
