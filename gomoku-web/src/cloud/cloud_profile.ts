import {
  doc,
  getDoc,
  serverTimestamp,
  setDoc,
  type Firestore,
} from "firebase/firestore";

import type { GameVariant } from "../core/bot_protocol";

import { getFirebaseClients } from "./firebase";
import type { CloudAuthUser } from "./auth_store";

export interface CloudProfile {
  authProviders: string[];
  avatarUrl: string | null;
  displayName: string;
  email: string | null;
  historyResetAt: string | null;
  preferredVariant: GameVariant;
  uid: string;
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
  return {
    authProviders: providerIds(document?.auth_providers, user.providerIds),
    avatarUrl: stringOrNull(document?.avatar_url) ?? user.avatarUrl,
    displayName: stringOrNull(document?.display_name) ?? user.displayName,
    email: stringOrNull(document?.email) ?? user.email,
    historyResetAt: timestampIsoOrNull(document?.history_reset_at),
    preferredVariant: validVariant(document?.preferred_variant) ?? fallbackVariant,
    uid: user.uid,
    username: stringOrNull(document?.username),
  };
}

export const CLOUD_PROFILE_SCHEMA_VERSION = 1;

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
    schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
    uid: user.uid,
    updated_at: now,
  };
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
