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
  last_login_at?: unknown;
  preferred_variant?: unknown;
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

function providerIds(value: unknown, fallback: string[]): string[] {
  if (!Array.isArray(value)) {
    return fallback;
  }

  return value.filter((entry): entry is string => typeof entry === "string" && entry.trim().length > 0);
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
    preferredVariant: validVariant(document?.preferred_variant) ?? fallbackVariant,
    uid: user.uid,
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
    last_login_at: now,
    preferred_variant: preferredVariant,
    uid: user.uid,
    updated_at: now,
    username: null,
  };
}

export function existingCloudProfileUpdate(user: CloudAuthUser, preferredVariant: GameVariant) {
  const now = serverTimestamp();

  return {
    auth_providers: user.providerIds,
    avatar_url: user.avatarUrl,
    email: user.email,
    last_login_at: now,
    preferred_variant: preferredVariant,
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
    await setDoc(profileRef, existingCloudProfileUpdate(user, preferredVariant), { merge: true });
    return cloudProfileFromDocument(user, preferredVariant, data);
  }

  const profile = newCloudProfileWrite(user, preferredVariant);
  await setDoc(profileRef, profile);
  return cloudProfileFromDocument(user, preferredVariant, profile);
}
