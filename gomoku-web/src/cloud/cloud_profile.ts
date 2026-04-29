import {
  doc,
  getDoc,
  serverTimestamp,
  setDoc,
  type Firestore,
} from "firebase/firestore";

import type { GameVariant } from "../core/bot_protocol";
import { isSavedMatchV1, savedMatchIsAfterReset, type SavedMatchV1 } from "../match/saved_match";

import { getFirebaseClients } from "./firebase";
import type { CloudAuthUser } from "./auth_store";
import { createCloudSavedMatch } from "./cloud_match";

export const CLOUD_PROFILE_SCHEMA_VERSION = 2;
export const CLOUD_RECENT_MATCHES_SCHEMA_VERSION = 1;
export const CLOUD_RECENT_MATCHES_LIMIT = 24;
export const CLOUD_PROFILE_SYNC_INTERVAL_MS = 15 * 60 * 1000;

export interface CloudRecentMatches {
  matches: SavedMatchV1[];
  schemaVersion: typeof CLOUD_RECENT_MATCHES_SCHEMA_VERSION;
  updatedAt: string | null;
}

export interface CloudProfile {
  authProviders: string[];
  avatarUrl: string | null;
  createdAt: string | null;
  displayName: string;
  email: string | null;
  historyResetAt: string | null;
  preferredVariant: GameVariant;
  recentMatches: CloudRecentMatches;
  uid: string;
  updatedAt: string | null;
  username: string | null;
}

export interface CloudProfileDocument {
  auth_providers?: unknown;
  avatar_url?: unknown;
  created_at?: unknown;
  display_name?: unknown;
  email?: unknown;
  history_reset_at?: unknown;
  last_login_at?: unknown;
  preferred_variant?: unknown;
  recent_matches?: unknown;
  schema_version?: unknown;
  uid?: unknown;
  updated_at?: unknown;
  username?: unknown;
}

export interface EnsureCloudProfileOptions {
  firestore?: Firestore;
}

function validVariant(value: unknown): GameVariant | null {
  return value === "freestyle" || value === "renju" ? value : null;
}

function stringOrNull(value: unknown): string | null {
  return typeof value === "string" && value.trim() ? value : null;
}

function timestampIsoOrNull(value: unknown): string | null {
  if (value instanceof Date && Number.isFinite(value.getTime())) {
    return value.toISOString();
  }

  const candidate = value as { nanoseconds?: unknown; seconds?: unknown; toDate?: unknown } | null;
  if (candidate && typeof candidate === "object") {
    if (typeof candidate.toDate === "function") {
      const date = candidate.toDate() as Date;
      return Number.isFinite(date.getTime()) ? date.toISOString() : null;
    }

    if (typeof candidate.seconds === "number") {
      const nanoseconds = typeof candidate.nanoseconds === "number" ? candidate.nanoseconds : 0;
      return new Date((candidate.seconds * 1000) + Math.floor(nanoseconds / 1_000_000)).toISOString();
    }
  }

  return null;
}

function providerIds(value: unknown, fallback: string[]): string[] {
  if (!Array.isArray(value)) {
    return fallback;
  }

  return value.filter((entry): entry is string => typeof entry === "string" && entry.trim().length > 0);
}

function sortRecentMatches(matches: SavedMatchV1[]): SavedMatchV1[] {
  return [...matches].sort((left, right) => right.saved_at.localeCompare(left.saved_at));
}

function recentMatchesFromDocument(
  value: unknown,
  historyResetAt: string | null,
): CloudRecentMatches {
  const candidate = value as {
    matches?: unknown;
    schema_version?: unknown;
    updated_at?: unknown;
  } | null;
  const matches = Array.isArray(candidate?.matches)
    ? candidate.matches
      .filter(isSavedMatchV1)
      .filter((match) => savedMatchIsAfterReset(match, historyResetAt))
    : [];

  return {
    matches: sortRecentMatches(matches).slice(0, CLOUD_RECENT_MATCHES_LIMIT),
    schemaVersion: CLOUD_RECENT_MATCHES_SCHEMA_VERSION,
    updatedAt: timestampIsoOrNull(candidate?.updated_at),
  };
}

export function mergeCloudRecentMatches(
  user: Pick<CloudAuthUser, "uid">,
  matches: SavedMatchV1[],
  historyResetAt: string | null | undefined = null,
): SavedMatchV1[] {
  const byId = new Map<string, SavedMatchV1>();

  for (const match of matches) {
    if (!savedMatchIsAfterReset(match, historyResetAt)) {
      continue;
    }

    const cloudMatch = match.source === "local_history"
      ? createCloudSavedMatch(user, match)
      : match;
    const existing = byId.get(cloudMatch.id);

    if (!existing || cloudMatch.saved_at >= existing.saved_at) {
      byId.set(cloudMatch.id, cloudMatch);
    }
  }

  return sortRecentMatches(Array.from(byId.values())).slice(0, CLOUD_RECENT_MATCHES_LIMIT);
}

function recentMatchesDocument(matches: SavedMatchV1[], updatedAt: unknown = serverTimestamp()) {
  return {
    matches,
    schema_version: CLOUD_RECENT_MATCHES_SCHEMA_VERSION,
    updated_at: updatedAt,
  };
}

function recentMatchesEqual(left: SavedMatchV1[], right: SavedMatchV1[]): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function hasOwnField(document: CloudProfileDocument, field: keyof CloudProfileDocument): boolean {
  return Object.prototype.hasOwnProperty.call(document, field);
}

function providerIdsMatch(value: unknown, expected: string[]): boolean {
  return (
    Array.isArray(value)
    && value.length === expected.length
    && value.every((entry, index) => entry === expected[index])
  );
}

function nullableFieldMatches(
  document: CloudProfileDocument,
  field: keyof Pick<CloudProfileDocument, "avatar_url" | "email">,
  expected: string | null,
): boolean {
  return hasOwnField(document, field) && document[field] === expected;
}

export function cloudProfileFromDocument(
  user: CloudAuthUser,
  fallbackVariant: GameVariant,
  document: CloudProfileDocument | null,
): CloudProfile {
  const historyResetAt = timestampIsoOrNull(document?.history_reset_at);

  return {
    authProviders: providerIds(document?.auth_providers, user.providerIds),
    avatarUrl: stringOrNull(document?.avatar_url) ?? user.avatarUrl,
    createdAt: timestampIsoOrNull(document?.created_at),
    displayName: stringOrNull(document?.display_name) ?? user.displayName,
    email: stringOrNull(document?.email) ?? user.email,
    historyResetAt,
    preferredVariant: validVariant(document?.preferred_variant) ?? fallbackVariant,
    recentMatches: recentMatchesFromDocument(document?.recent_matches, historyResetAt),
    uid: user.uid,
    updatedAt: timestampIsoOrNull(document?.updated_at),
    username: stringOrNull(document?.username),
  };
}

export function newCloudProfileWrite(user: CloudAuthUser, preferredVariant: GameVariant) {
  const now = serverTimestamp();

  return {
    auth_providers: user.providerIds,
    avatar_url: user.avatarUrl,
    created_at: now,
    display_name: user.displayName,
    email: user.email,
    history_reset_at: null,
    last_login_at: now,
    preferred_variant: preferredVariant,
    recent_matches: recentMatchesDocument([], null),
    schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
    uid: user.uid,
    updated_at: now,
    username: null,
  };
}

export function existingCloudProfileUpdate(
  user: CloudAuthUser,
): Record<string, unknown>;
export function existingCloudProfileUpdate(
  user: CloudAuthUser,
  document: CloudProfileDocument,
): Record<string, unknown> | null;
export function existingCloudProfileUpdate(
  user: CloudAuthUser,
  document?: CloudProfileDocument,
): Record<string, unknown> | null {
  const patch: Record<string, unknown> = {};

  if (!document || !providerIdsMatch(document.auth_providers, user.providerIds)) {
    patch.auth_providers = user.providerIds;
  }

  if (!document || !nullableFieldMatches(document, "avatar_url", user.avatarUrl)) {
    patch.avatar_url = user.avatarUrl;
  }

  if (!document || !nullableFieldMatches(document, "email", user.email)) {
    patch.email = user.email;
  }

  if (!document || document.schema_version !== CLOUD_PROFILE_SCHEMA_VERSION) {
    patch.schema_version = CLOUD_PROFILE_SCHEMA_VERSION;
  }

  if (!document || !document.recent_matches) {
    patch.recent_matches = recentMatchesDocument([]);
  }

  if (!document || document.uid !== user.uid) {
    patch.uid = user.uid;
  }

  if (document && Object.keys(patch).length === 0) {
    return null;
  }

  const now = serverTimestamp();

  return {
    ...patch,
    last_login_at: now,
    updated_at: now,
  };
}

export function resetCloudProfileUpdate(user: CloudAuthUser, preferredVariant: GameVariant) {
  const now = serverTimestamp();

  return {
    auth_providers: user.providerIds,
    avatar_url: user.avatarUrl,
    display_name: user.displayName,
    email: user.email,
    history_reset_at: now,
    last_login_at: now,
    preferred_variant: preferredVariant,
    recent_matches: recentMatchesDocument([], now),
    schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
    uid: user.uid,
    updated_at: now,
  };
}

export function cloudProfileSnapshotUpdate(input: {
  displayName: string;
  preferredVariant: GameVariant;
  recentMatches: SavedMatchV1[];
  user: CloudAuthUser;
}): Record<string, unknown> {
  const now = serverTimestamp();

  return {
    auth_providers: input.user.providerIds,
    avatar_url: input.user.avatarUrl,
    display_name: input.displayName,
    email: input.user.email,
    last_login_at: now,
    preferred_variant: input.preferredVariant,
    recent_matches: recentMatchesDocument(input.recentMatches, now),
    schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
    uid: input.user.uid,
    updated_at: now,
  };
}

export function cloudProfileSyncDue(
  profile: Pick<CloudProfile, "createdAt" | "recentMatches" | "updatedAt">,
  nowMs = Date.now(),
): boolean {
  if (!profile.updatedAt) {
    return true;
  }

  if (profile.createdAt === profile.updatedAt && profile.recentMatches.matches.length === 0) {
    return true;
  }

  const updatedAtMs = Date.parse(profile.updatedAt);
  return !Number.isFinite(updatedAtMs) || nowMs >= updatedAtMs + CLOUD_PROFILE_SYNC_INTERVAL_MS;
}

export function cloudProfileNeedsSnapshotSync(input: {
  cloudDisplayName: string;
  cloudPreferredVariant: GameVariant;
  cloudRecentMatches: SavedMatchV1[];
  displayName: string;
  preferredVariant: GameVariant;
  recentMatches: SavedMatchV1[];
}): boolean {
  return input.cloudDisplayName !== input.displayName
    || input.cloudPreferredVariant !== input.preferredVariant
    || !recentMatchesEqual(input.cloudRecentMatches, input.recentMatches);
}

export async function ensureCloudProfile(
  user: CloudAuthUser,
  preferredVariant: GameVariant,
  options: EnsureCloudProfileOptions = {},
): Promise<CloudProfile> {
  const firestore = options.firestore ?? getFirebaseClients()?.firestore;
  if (!firestore) {
    throw new Error("Cloud profile is not configured for this build.");
  }

  const profileRef = doc(firestore, "profiles", user.uid);
  const snapshot = await getDoc(profileRef);

  if (snapshot.exists()) {
    const data = snapshot.data() as CloudProfileDocument;
    const update = existingCloudProfileUpdate(user, data);
    if (!update) {
      return cloudProfileFromDocument(user, preferredVariant, data);
    }

    await setDoc(profileRef, update, { merge: true });
    return cloudProfileFromDocument(user, preferredVariant, { ...data, ...update });
  }

  const profile = newCloudProfileWrite(user, preferredVariant);
  await setDoc(profileRef, profile);
  return cloudProfileFromDocument(user, preferredVariant, profile);
}

export async function resetCloudProfile(
  user: CloudAuthUser,
  preferredVariant: GameVariant,
  options: EnsureCloudProfileOptions = {},
): Promise<CloudProfile> {
  const firestore = options.firestore ?? getFirebaseClients()?.firestore;
  if (!firestore) {
    throw new Error("Cloud profile reset is not configured for this build.");
  }

  const profileRef = doc(firestore, "profiles", user.uid);
  const snapshot = await getDoc(profileRef);
  if (!snapshot.exists()) {
    const profile = {
      ...newCloudProfileWrite(user, preferredVariant),
      history_reset_at: serverTimestamp(),
    };
    await setDoc(profileRef, profile);
    const refreshed = await getDoc(profileRef);
    return cloudProfileFromDocument(
      user,
      preferredVariant,
      refreshed.exists() ? (refreshed.data() as CloudProfileDocument) : profile,
    );
  }

  const update = resetCloudProfileUpdate(user, preferredVariant);
  await setDoc(profileRef, update, { merge: true });
  const refreshed = await getDoc(profileRef);
  const fallback = {
    ...(snapshot.data() as CloudProfileDocument),
    ...update,
  };

  return cloudProfileFromDocument(
    user,
    preferredVariant,
    refreshed.exists() ? (refreshed.data() as CloudProfileDocument) : fallback,
  );
}
