import { useEffect } from "react";
import { useStore } from "zustand";

import { DEFAULT_LOCAL_DISPLAY_NAME, localProfileStore } from "../profile/local_profile_store";

import { cloudAuthStore, type CloudAuthUser } from "./auth_store";
import { cloudHistoryStore } from "./cloud_history_store";
import {
  cloudMatchHistoryHasMatch,
  cloudProfileNeedsSnapshotSync,
  cloudProfileSyncWaitMs,
  type CloudProfile,
} from "./cloud_profile";
import { cloudProfileStore } from "./cloud_profile_store";
import { cloudPromotionStore } from "./cloud_promotion_store";
import { flushCloudProfileSync } from "./cloud_sync";

const SYNC_RETRY_BUFFER_MS = 100;
const EMPTY_PENDING_CLOUD_MATCHES = {};

function shouldAdoptCloudDisplayName(
  localDisplayName: string | undefined,
  cloudDisplayName: string | undefined,
): cloudDisplayName is string {
  return (
    localDisplayName === DEFAULT_LOCAL_DISPLAY_NAME
    && Boolean(cloudDisplayName?.trim())
    && cloudDisplayName !== DEFAULT_LOCAL_DISPLAY_NAME
  );
}

async function flushLocalStateToCloud(user: CloudAuthUser): Promise<CloudProfile | null> {
  const localState = localProfileStore.getState();
  const cloudProfile = await flushCloudProfileSync(user, {
    localMatchHistory: localState.matchHistory,
  });

  if (!cloudProfile) {
    return null;
  }

  cloudHistoryStore.getState().loadFromProfile(user, cloudProfile);
  for (const match of localState.matchHistory.replayMatches) {
    if (!cloudMatchHistoryHasMatch(cloudProfile.matchHistory, match.id)) {
      void cloudHistoryStore.getState().syncMatchForUser(user, match, cloudProfile.resetAt);
    }
  }

  return cloudProfile;
}

export function CloudSessionController() {
  const authStatus = useStore(cloudAuthStore, (state) => state.status);
  const authUser = useStore(cloudAuthStore, (state) => state.user);
  const cloudProfile = useStore(cloudProfileStore, (state) => state.profile);
  const cloudProfileStatus = useStore(cloudProfileStore, (state) => state.status);
  const localMatchHistory = useStore(localProfileStore, (state) => state.matchHistory);
  const localProfile = useStore(localProfileStore, (state) => state.profile);
  const settings = useStore(localProfileStore, (state) => state.settings);
  const pendingCloudMatches = useStore(cloudHistoryStore, (state) => (
    authUser ? state.users[authUser.uid]?.pendingMatches ?? EMPTY_PENDING_CLOUD_MATCHES : EMPTY_PENDING_CLOUD_MATCHES
  ));

  useEffect(() => {
    localProfileStore.getState().ensureLocalProfile();
  }, []);

  useEffect(() => {
    cloudAuthStore.getState().start();

    return () => {
      cloudAuthStore.getState().stop();
    };
  }, []);

  useEffect(() => {
    if (authStatus === "signed_in" && authUser) {
      void cloudProfileStore.getState().loadForUser(authUser, localProfileStore.getState().settings);
      return;
    }

    cloudProfileStore.getState().reset();
    cloudPromotionStore.getState().reset();
  }, [authStatus, authUser]);

  useEffect(() => {
    if (authStatus === "signed_in" && authUser && cloudProfileStatus === "ready") {
      void cloudHistoryStore.getState().loadForUser(authUser, cloudProfile?.resetAt ?? null);
    }
  }, [authStatus, authUser, cloudProfile?.resetAt, cloudProfileStatus]);

  useEffect(() => {
    if (authStatus !== "signed_in" || !authUser || cloudProfileStatus !== "ready") {
      return undefined;
    }

    const retryCloudSync = () => {
      void cloudHistoryStore.getState().loadForUser(authUser, cloudProfile?.resetAt ?? null);
      void flushLocalStateToCloud(authUser).catch(() => {
        // Store actions surface foreground failures through their own state.
      });
      void cloudHistoryStore.getState().syncPendingForUser(authUser, cloudProfile?.resetAt ?? null);
    };

    window.addEventListener("online", retryCloudSync);
    return () => {
      window.removeEventListener("online", retryCloudSync);
    };
  }, [authStatus, authUser, cloudProfile?.resetAt, cloudProfileStatus]);

  useEffect(() => {
    const cloudDisplayName = cloudProfile?.displayName;
    if (
      authStatus === "signed_in"
      && cloudProfileStatus === "ready"
      && shouldAdoptCloudDisplayName(localProfile?.displayName, cloudDisplayName)
    ) {
      localProfileStore.getState().renameDisplayName(cloudDisplayName);
    }
  }, [
    authStatus,
    cloudProfile?.displayName,
    cloudProfileStatus,
    localProfile?.displayName,
  ]);

  useEffect(() => {
    if (
      authStatus !== "signed_in"
      || !authUser
      || cloudProfileStatus !== "ready"
      || !cloudProfile
      || !localProfile
      || shouldAdoptCloudDisplayName(localProfile.displayName, cloudProfile.displayName)
    ) {
      return undefined;
    }

    if (!cloudProfileNeedsSnapshotSync({
      cloudDisplayName: cloudProfile.displayName,
      cloudMatchHistory: cloudProfile.matchHistory,
      cloudSettings: cloudProfile.settings,
      displayName: localProfile.displayName,
      matchHistory: localMatchHistory,
      settings,
    })) {
      return undefined;
    }

    let cancelled = false;
    const flush = () => {
      void flushLocalStateToCloud(authUser).catch(() => {
        // Store actions surface foreground failures through their own state.
      });
    };
    const waitMs = cloudProfileSyncWaitMs(cloudProfile);

    if (waitMs <= 0) {
      flush();
      return undefined;
    }

    const timer = window.setTimeout(() => {
      if (!cancelled) {
        flush();
      }
    }, waitMs + SYNC_RETRY_BUFFER_MS);

    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [
    authStatus,
    authUser,
    cloudProfile,
    cloudProfileStatus,
    localMatchHistory,
    localProfile,
    settings,
  ]);

  useEffect(() => {
    if (
      authStatus !== "signed_in"
      || !authUser
      || cloudProfileStatus !== "ready"
      || !cloudProfile
      || Object.keys(pendingCloudMatches).length === 0
    ) {
      return undefined;
    }

    const syncPending = () => {
      void cloudHistoryStore.getState().syncPendingForUser(authUser, cloudProfile.resetAt);
    };
    const waitMs = cloudProfileSyncWaitMs(cloudProfile);

    if (waitMs <= 0) {
      syncPending();
      return undefined;
    }

    const timer = window.setTimeout(syncPending, waitMs + SYNC_RETRY_BUFFER_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [
    authStatus,
    authUser,
    cloudProfile,
    cloudProfileStatus,
    pendingCloudMatches,
  ]);

  return null;
}
