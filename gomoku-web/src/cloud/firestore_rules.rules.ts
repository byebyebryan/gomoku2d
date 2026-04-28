import { readFile } from "node:fs/promises";

import {
  assertFails,
  assertSucceeds,
  initializeTestEnvironment,
  type RulesTestEnvironment,
} from "@firebase/rules-unit-testing";
import firebase from "firebase/compat/app";
import "firebase/compat/firestore";
import { afterAll, beforeAll, beforeEach, describe, it } from "vitest";

const projectId = "demo-gomoku2d-rules-test";

let testEnv: RulesTestEnvironment;

function timestamp(iso: string): firebase.firestore.Timestamp {
  return firebase.firestore.Timestamp.fromDate(new Date(iso));
}

function serverTimestamp(): firebase.firestore.FieldValue {
  return firebase.firestore.FieldValue.serverTimestamp();
}

function ownerDb(uid = "uid-1"): firebase.firestore.Firestore {
  return testEnv.authenticatedContext(uid).firestore();
}

function profileDocument(uid: string, overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    auth_providers: ["google.com"],
    avatar_url: "https://example.com/avatar.png",
    created_at: timestamp("2020-01-01T00:00:00.000Z"),
    display_name: "Bryan",
    email: "bryan@example.com",
    history_reset_at: null,
    last_login_at: timestamp("2020-01-01T00:00:00.000Z"),
    preferred_variant: "freestyle",
    schema_version: 1,
    uid,
    updated_at: timestamp("2020-01-01T00:00:00.000Z"),
    username: null,
    ...overrides,
  };
}

function profileCreateDocument(uid: string, overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    ...profileDocument(uid),
    created_at: serverTimestamp(),
    last_login_at: serverTimestamp(),
    updated_at: serverTimestamp(),
    ...overrides,
  };
}

function profileUpdateDocument(uid: string, overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    ...profileDocument(uid),
    last_login_at: serverTimestamp(),
    updated_at: serverTimestamp(),
    ...overrides,
  };
}

async function seedProfile(uid: string, overrides: Record<string, unknown> = {}): Promise<void> {
  await testEnv.withSecurityRulesDisabled(async (context) => {
    await context.firestore().doc(`profiles/${uid}`).set(profileDocument(uid, overrides));
  });
}

function botPlayer(): Record<string, unknown> {
  return {
    bot: {
      config: {
        depth: 3,
        kind: "baseline",
      },
      config_version: 1,
      engine: "baseline_search",
      id: "practice_bot",
      version: 1,
    },
    display_name: "Practice Bot",
    kind: "bot",
    local_profile_id: null,
    profile_uid: null,
  };
}

function cloudHumanPlayer(uid: string): Record<string, unknown> {
  return {
    bot: null,
    display_name: "Bryan",
    kind: "human",
    local_profile_id: null,
    profile_uid: uid,
  };
}

function importedHumanPlayer(uid: string): Record<string, unknown> {
  return {
    ...cloudHumanPlayer(uid),
    local_profile_id: "guest-1",
  };
}

function cloudSavedMatchDocument(
  uid: string,
  matchId: string,
  matchSavedAt: firebase.firestore.Timestamp,
  overrides: Record<string, unknown> = {},
): Record<string, unknown> {
  return {
    board_size: 15,
    created_at: serverTimestamp(),
    id: matchId,
    match_kind: "local_vs_bot",
    match_saved_at: matchSavedAt,
    move_cells: [112],
    move_count: 1,
    player_black: cloudHumanPlayer(uid),
    player_white: botPlayer(),
    saved_at: matchSavedAt.toDate().toISOString(),
    schema_version: 1,
    source: "cloud_saved",
    status: "draw",
    trust: "client_uploaded",
    undo_floor: 0,
    variant: "freestyle",
    ...overrides,
  };
}

function guestImportMatchDocument(
  uid: string,
  matchId: string,
  matchSavedAt: firebase.firestore.Timestamp,
  overrides: Record<string, unknown> = {},
): Record<string, unknown> {
  const document: Record<string, unknown> = {
    ...cloudSavedMatchDocument(uid, matchId, matchSavedAt),
    id: matchId,
    imported_at: serverTimestamp(),
    local_match_id: "local-1",
    local_origin_id: "guest:guest-1:local-1",
    player_black: importedHumanPlayer(uid),
    source: "guest_import",
    ...overrides,
  };
  delete document.created_at;
  return document;
}

async function seedMatch(uid: string, matchId: string, document: Record<string, unknown>): Promise<void> {
  await testEnv.withSecurityRulesDisabled(async (context) => {
    await context.firestore().doc(`profiles/${uid}/matches/${matchId}`).set(document);
  });
}

beforeAll(async () => {
  const rules = await readFile(new URL("../../../firestore.rules", import.meta.url), "utf8");
  testEnv = await initializeTestEnvironment({
    projectId,
    firestore: { rules },
  });
});

beforeEach(async () => {
  await testEnv.clearFirestore();
});

afterAll(async () => {
  await testEnv.cleanup();
});

describe("Firestore profile rules", () => {
  it("allows the owner to create and read their profile only", async () => {
    await assertSucceeds(ownerDb().doc("profiles/uid-1").set(profileCreateDocument("uid-1")));
    await assertSucceeds(ownerDb().doc("profiles/uid-1").get());
    await assertFails(ownerDb("uid-2").doc("profiles/uid-1").get());
    await assertFails(testEnv.unauthenticatedContext().firestore().doc("profiles/uid-1").get());
  });

  it("allows reset barriers only when they are generated by the write request", async () => {
    await seedProfile("uid-1");

    await assertSucceeds(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          history_reset_at: serverTimestamp(),
        }),
      ),
    );

    await assertFails(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          history_reset_at: timestamp("2020-01-01T00:00:00.000Z"),
        }),
      ),
    );
  });

  it("does not allow the owner to remove an existing reset barrier", async () => {
    await seedProfile("uid-1", {
      history_reset_at: timestamp("2020-01-01T00:00:00.000Z"),
    });

    const update = profileUpdateDocument("uid-1");
    delete update.history_reset_at;

    await assertFails(ownerDb().doc("profiles/uid-1").set(update));
  });

  it("does not allow a reset write to move an existing barrier backward", async () => {
    await seedProfile("uid-1", {
      history_reset_at: timestamp("2999-01-01T00:00:00.000Z"),
    });

    await assertFails(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          history_reset_at: serverTimestamp(),
        }),
      ),
    );
  });

  it("rejects arbitrary reset barriers during profile creation", async () => {
    await assertFails(
      ownerDb("uid-2").doc("profiles/uid-2").set(
        profileCreateDocument("uid-2", {
          history_reset_at: timestamp("2020-01-01T00:00:00.000Z"),
        }),
      ),
    );
  });
});

describe("Firestore private match rules", () => {
  it("allows owner cloud_saved creates after the profile reset barrier", async () => {
    await seedProfile("uid-1", {
      history_reset_at: timestamp("2020-01-01T00:00:00.000Z"),
    });

    await assertSucceeds(
      ownerDb().doc("profiles/uid-1/matches/match-1").set(
        cloudSavedMatchDocument("uid-1", "match-1", timestamp("2020-01-01T01:00:00.000Z")),
      ),
    );
  });

  it("allows owner guest_import creates after the profile reset barrier", async () => {
    await seedProfile("uid-1", {
      history_reset_at: timestamp("2020-01-01T00:00:00.000Z"),
    });

    await assertSucceeds(
      ownerDb().doc("profiles/uid-1/matches/local-match-1").set(
        guestImportMatchDocument("uid-1", "local-match-1", timestamp("2020-01-01T01:00:00.000Z")),
      ),
    );
  });

  it("rejects stale match creates at or before the profile reset barrier", async () => {
    await seedProfile("uid-1", {
      history_reset_at: timestamp("2020-01-01T00:00:00.000Z"),
    });

    await assertFails(
      ownerDb().doc("profiles/uid-1/matches/match-1").set(
        cloudSavedMatchDocument("uid-1", "match-1", timestamp("2020-01-01T00:00:00.000Z")),
      ),
    );
  });

  it("requires match_saved_at on match creates", async () => {
    await seedProfile("uid-1");

    const missingTimestamp = cloudSavedMatchDocument("uid-1", "match-1", timestamp("2020-01-01T01:00:00.000Z"));
    delete missingTimestamp.match_saved_at;
    await assertFails(ownerDb().doc("profiles/uid-1/matches/match-1").set(missingTimestamp));
  });

  it("rejects match creates when the parent cloud profile does not exist", async () => {
    await assertFails(
      ownerDb().doc("profiles/uid-1/matches/match-1").set(
        cloudSavedMatchDocument("uid-1", "match-1", timestamp("2020-01-01T01:00:00.000Z")),
      ),
    );
  });

  it("rejects cross-user match writes and reads", async () => {
    await seedProfile("uid-1");

    await assertFails(
      ownerDb("uid-2").doc("profiles/uid-1/matches/match-1").set(
        cloudSavedMatchDocument("uid-1", "match-1", timestamp("2020-01-01T01:00:00.000Z")),
      ),
    );
    await assertFails(ownerDb("uid-2").doc("profiles/uid-1/matches/match-1").get());
  });

  it("lets owners delete private client-uploaded matches but not verified matches", async () => {
    await seedProfile("uid-1");
    await seedMatch("uid-1", "client-match", { trust: "client_uploaded" });
    await seedMatch("uid-1", "verified-match", { trust: "server_verified" });

    await assertSucceeds(ownerDb().doc("profiles/uid-1/matches/client-match").delete());
    await assertFails(ownerDb().doc("profiles/uid-1/matches/verified-match").delete());
  });
});
