import { readFile } from "node:fs/promises";

import {
  assertFails,
  assertSucceeds,
  initializeTestEnvironment,
  type RulesTestEnvironment,
} from "@firebase/rules-unit-testing";
import type {
  FieldValue,
  FirebaseFirestore,
  Timestamp,
} from "@firebase/firestore-types";
import firebase from "firebase/compat/app";
import "firebase/compat/firestore";
import { afterAll, beforeAll, beforeEach, describe, expect, it } from "vitest";
import type { Firestore } from "firebase/firestore";

const projectId = "demo-gomoku2d-rules-test";
const matchHistoryLimit = 128;
const summaryMatchesLimit = 1024;

let testEnv: RulesTestEnvironment;

const firebaseFirestore = (firebase as unknown as {
  firestore: {
    FieldValue: { serverTimestamp: () => FieldValue };
    Timestamp: { fromDate: (date: Date) => Timestamp };
  };
}).firestore;

function timestamp(iso: string): Timestamp {
  return firebaseFirestore.Timestamp.fromDate(new Date(iso));
}

function serverTimestamp(): FieldValue {
  return firebaseFirestore.FieldValue.serverTimestamp();
}

function ownerDb(uid = "uid-1"): FirebaseFirestore {
  return testEnv.authenticatedContext(uid).firestore() as unknown as FirebaseFirestore;
}

function emptyStatsCounter(): Record<string, number> {
  return {
    draws: 0,
    losses: 0,
    matches: 0,
    moves: 0,
    wins: 0,
  };
}

function emptyArchivedStats(): Record<string, unknown> {
  return {
    archived_before: null,
    archived_count: 0,
    by_opponent_type: {
      bot: emptyStatsCounter(),
      human: emptyStatsCounter(),
    },
    by_ruleset: {
      freestyle: emptyStatsCounter(),
      renju: emptyStatsCounter(),
    },
    by_side: {
      black: emptyStatsCounter(),
      white: emptyStatsCounter(),
    },
    schema_version: 1,
    totals: emptyStatsCounter(),
  };
}

function emptyMatchHistory(): Record<string, unknown> {
  return {
    archived_stats: emptyArchivedStats(),
    replay_matches: [],
    summary_matches: [],
  };
}

function defaultBot(): Record<string, unknown> {
  return {
    mode: "preset",
    preset: "normal",
    version: 1,
  };
}

function defaultSettings(ruleset = "freestyle"): Record<string, unknown> {
  return {
    board_hints: {
      evidence: "on",
      immediate: "win_threat",
      imminent: "threat_counter",
    },
    bot_config: defaultBot(),
    game_config: {
      opening: "standard",
      ruleset,
    },
    touch_control: "touchpad",
  };
}

function profileDocument(uid: string, overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    auth: {
      providers: [
        {
          avatar_url: "https://example.com/avatar.png",
          display_name: "Bryan",
          provider: "google.com",
        },
      ],
    },
    created_at: timestamp("2020-01-01T00:00:00.000Z"),
    display_name: "Bryan",
    match_history: emptyMatchHistory(),
    reset_at: null,
    schema_version: 5,
    settings: defaultSettings(),
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
    updated_at: serverTimestamp(),
    ...overrides,
  };
}

function profileUpdateDocument(uid: string, overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    ...profileDocument(uid),
    updated_at: serverTimestamp(),
    ...overrides,
  };
}

async function seedProfile(uid: string, overrides: Record<string, unknown> = {}): Promise<void> {
  await testEnv.withSecurityRulesDisabled(async (context) => {
    await context.firestore().doc(`profiles/${uid}`).set(profileDocument(uid, overrides));
  });
}

async function seedMatch(uid: string, matchId: string, document: Record<string, unknown>): Promise<void> {
  await testEnv.withSecurityRulesDisabled(async (context) => {
    await context.firestore().doc(`profiles/${uid}/matches/${matchId}`).set(document);
  });
}

beforeAll(async () => {
  const rules = await readFile(new URL("../../firestore.rules", import.meta.url), "utf8");
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
  it.each(["history", "promotion"])("guards production %s writes against another device's settings/reset", async (path) => {
    // Runtime imports keep this NodeNext tool file independent of the app's
    // bundler resolution; the app sources are checked by its own tsconfig.
    const profileModule = "../src/cloud/cloud_profile";
    const historyModule = "../src/cloud/cloud_history";
    const promotionModule = "../src/cloud/cloud_promotion";
    const { cloudProfileFromDocument } = await import(profileModule);
    const { saveCloudHistorySnapshot } = await import(historyModule);
    const { promoteLocalProfileToCloud } = await import(promotionModule);
    const user = { uid: "uid-1", avatarUrl: null, displayName: "Bryan", email: null, providerIds: ["google.com"] };
    for (const change of ["settings", "reset"]) {
      await seedProfile(user.uid);
      const original = (await ownerDb().doc(`profiles/${user.uid}`).get()).data();
      const stale = cloudProfileFromDocument(user, undefined, original);
      await seedProfile(user.uid, change === "settings"
        ? { settings: defaultSettings("renju") }
        : { reset_at: timestamp("2020-01-02T00:00:00.000Z") });
      const options = { firestore: ownerDb() as unknown as Firestore };
      const operation = path === "history" ? saveCloudHistorySnapshot(user, {
        cloudProfile: stale, displayName: stale.displayName, matches: [], settings: stale.settings,
    }, options) : promoteLocalProfileToCloud({
      sourceProfile: stale, cloudDisplayName: stale.displayName, cloudMatchHistory: stale.matchHistory,
      cloudSettings: stale.settings, localMatchHistory: stale.matchHistory,
      localProfile: { id: "local-test", displayName: "New name" }, settings: stale.settings, user,
    }, options);
    await expect(operation).rejects.toThrow(/changed.*device/i);
    const retained = (await ownerDb().doc(`profiles/${user.uid}`).get()).data();
    if (change === "settings") expect(retained?.settings.game_config.ruleset).toBe("renju");
    else expect(retained?.reset_at.toDate().toISOString()).toBe("2020-01-02T00:00:00.000Z");
    }
  });

  it.each(["history", "promotion"])("allows a current production %s snapshot through its transaction and rules", async (path) => {
    const profileModule = "../src/cloud/cloud_profile";
    const historyModule = "../src/cloud/cloud_history";
    const promotionModule = "../src/cloud/cloud_promotion";
    const { cloudProfileFromDocument } = await import(profileModule);
    const { saveCloudHistorySnapshot } = await import(historyModule);
    const { promoteLocalProfileToCloud } = await import(promotionModule);
    const user = { uid: "uid-1", avatarUrl: null, displayName: "Bryan", email: null, providerIds: ["google.com"] };
    await seedProfile(user.uid);
    const current = cloudProfileFromDocument(user, undefined, (await ownerDb().doc(`profiles/${user.uid}`).get()).data());
    const options = { firestore: ownerDb() as unknown as Firestore };
    if (path === "history") await saveCloudHistorySnapshot(user, {
      cloudProfile: current, displayName: "New name", matches: [], settings: current.settings,
    }, options);
    else await promoteLocalProfileToCloud({
      sourceProfile: current, cloudDisplayName: current.displayName, cloudMatchHistory: current.matchHistory,
      cloudSettings: current.settings, localMatchHistory: current.matchHistory,
      localProfile: { id: "local-test", displayName: "New name" }, settings: current.settings, user,
    }, options);
    expect((await ownerDb().doc(`profiles/${user.uid}`).get()).data()?.display_name).toBe("New name");
  });
  it("allows the owner to create and read their profile only", async () => {
    await assertSucceeds(ownerDb().doc("profiles/uid-1").set(profileCreateDocument("uid-1")));
    await assertSucceeds(ownerDb().doc("profiles/uid-1").get());
    await assertFails(ownerDb("uid-2").doc("profiles/uid-1").get());
    await assertFails(testEnv.unauthenticatedContext().firestore().doc("profiles/uid-1").get());
  });

  it("rejects legacy auth provider fields in profile schema v5 writes", async () => {
    await assertFails(
      ownerDb().doc("profiles/uid-1").set(
        profileCreateDocument("uid-1", {
          auth: {
            providers: [
              {
                avatar_url: "https://example.com/avatar.png",
                display_name: "Bryan",
                provider: "google",
                provider_id: "google.com",
              },
            ],
          },
        }),
      ),
    );
  });

  it("allows reset barriers only when they are generated by the write request", async () => {
    await seedProfile("uid-1");

    await assertSucceeds(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          reset_at: serverTimestamp(),
          match_history: emptyMatchHistory(),
        }),
      ),
    );

    await assertFails(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          reset_at: timestamp("2020-01-01T00:00:00.000Z"),
        }),
      ),
    );
  });

  it("allows reset barriers to bypass the normal profile update cooldown", async () => {
    await seedProfile("uid-1", {
      reset_at: null,
      updated_at: timestamp("2999-01-01T00:00:00.000Z"),
    });

    await assertSucceeds(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          reset_at: serverTimestamp(),
          match_history: emptyMatchHistory(),
        }),
      ),
    );
  });

  it("rejects rapid profile updates inside the server-side cooldown", async () => {
    await seedProfile("uid-1", {
      updated_at: timestamp("2999-01-01T00:00:00.000Z"),
    });

    await assertFails(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          display_name: "Rapid Edit",
        }),
      ),
    );
  });

  it("allows profile updates after the server-side cooldown", async () => {
    await seedProfile("uid-1", {
      updated_at: timestamp("2020-01-01T00:00:00.000Z"),
    });

    await assertSucceeds(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          display_name: "Settled Edit",
        }),
      ),
    );
  });

  it("allows one initial profile sync immediately after profile creation", async () => {
    await seedProfile("uid-1", {
      created_at: timestamp("2999-01-01T00:00:00.000Z"),
      updated_at: timestamp("2999-01-01T00:00:00.000Z"),
    });

    await assertSucceeds(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          created_at: timestamp("2999-01-01T00:00:00.000Z"),
          display_name: "Promoted Local Name",
          match_history: {
            ...emptyMatchHistory(),
            replay_matches: [{ id: "match-1" }],
            summary_matches: [{ id: "match-1" }],
          },
          settings: {
            ...defaultSettings("renju"),
          },
        }),
      ),
    );
  });

  it("rejects profile writes with loose/raw bot config", async () => {
    await assertFails(
      ownerDb().doc("profiles/uid-1").set(
        profileCreateDocument("uid-1", {
          settings: {
            ...defaultSettings(),
            game_config: {
              opening: "standard",
              ruleset: "freestyle",
            },
            bot_config: {
              lab_spec: "search-d999+no-safety",
              mode: "preset",
              preset: "normal",
              version: 1,
            },
          },
        }),
      ),
    );
  });

  it("allows profile writes with strict custom bot config", async () => {
    await assertSucceeds(
      ownerDb().doc("profiles/uid-1").set(
        profileCreateDocument("uid-1", {
          settings: {
            ...defaultSettings(),
            game_config: {
              opening: "standard",
              ruleset: "freestyle",
            },
            bot_config: {
              depth: 7,
              extra_pass: "corridor_proof",
              mode: "custom",
              scoring: "pattern",
              version: 1,
              width: 8,
            },
          },
        }),
      ),
    );
  });

  it("does not allow the initial profile sync bypass after a later profile update", async () => {
    await seedProfile("uid-1", {
      created_at: timestamp("2020-01-01T00:00:00.000Z"),
      updated_at: timestamp("2999-01-01T00:00:00.000Z"),
    });

    await assertFails(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          display_name: "Rapid Followup",
        }),
      ),
    );
  });

  it("does not allow the owner to remove an existing reset barrier", async () => {
    await seedProfile("uid-1", {
      reset_at: timestamp("2020-01-01T00:00:00.000Z"),
    });

    const update = profileUpdateDocument("uid-1");
    delete update.reset_at;

    await assertFails(ownerDb().doc("profiles/uid-1").set(update));
  });

  it("does not allow a reset write to move an existing barrier backward", async () => {
    await seedProfile("uid-1", {
      reset_at: timestamp("2999-01-01T00:00:00.000Z"),
    });

    await assertFails(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          reset_at: serverTimestamp(),
          match_history: emptyMatchHistory(),
        }),
      ),
    );
  });

  it("rejects arbitrary reset barriers during profile creation", async () => {
    await assertFails(
      ownerDb("uid-2").doc("profiles/uid-2").set(
        profileCreateDocument("uid-2", {
          reset_at: timestamp("2020-01-01T00:00:00.000Z"),
        }),
      ),
    );
  });

  it("allows embedded match history in a settled profile snapshot update", async () => {
    await seedProfile("uid-1", {
      updated_at: timestamp("2020-01-01T00:00:00.000Z"),
    });

    await assertSucceeds(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          match_history: {
            ...emptyMatchHistory(),
            replay_matches: [{ id: "match-1" }],
          },
        }),
      ),
    );
  });

  it("rejects embedded match history snapshots over the cap", async () => {
    await seedProfile("uid-1", {
      updated_at: timestamp("2020-01-01T00:00:00.000Z"),
    });

    await assertFails(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          match_history: {
            ...emptyMatchHistory(),
            replay_matches: Array.from({ length: matchHistoryLimit + 1 }, (_, index) => ({ id: `match-${index}` })),
          },
        }),
      ),
    );
  });

  it("rejects embedded summary match snapshots over the cap", async () => {
    await seedProfile("uid-1", {
      updated_at: timestamp("2020-01-01T00:00:00.000Z"),
    });

    await assertFails(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          match_history: {
            ...emptyMatchHistory(),
            summary_matches: Array.from({ length: summaryMatchesLimit + 1 }, (_, index) => ({ id: `match-${index}` })),
          },
        }),
      ),
    );
  });

  it("rejects malformed archived stats counters", async () => {
    await seedProfile("uid-1", {
      updated_at: timestamp("2020-01-01T00:00:00.000Z"),
    });

    await assertFails(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          match_history: {
            ...emptyMatchHistory(),
            archived_stats: {
              ...emptyArchivedStats(),
              totals: {
                matches: 1,
              },
            },
          },
        }),
      ),
    );
  });

  it("rejects malformed board hint settings", async () => {
    await assertFails(
      ownerDb().doc("profiles/uid-1").set(
        profileCreateDocument("uid-1", {
          settings: {
            ...defaultSettings(),
            board_hints: {
              evidence: "on",
              immediate: "maybe",
              imminent: "threat_counter",
            },
          },
        }),
      ),
    );
  });

  it("rejects malformed touch control settings", async () => {
    await assertFails(
      ownerDb().doc("profiles/uid-1").set(
        profileCreateDocument("uid-1", {
          settings: {
            ...defaultSettings(),
            touch_control: "mouse",
          },
        }),
      ),
    );
  });

  it("rejects the legacy recent_matches field in profile schema v5 writes", async () => {
    await seedProfile("uid-1");

    await assertFails(
      ownerDb().doc("profiles/uid-1").set(
        profileUpdateDocument("uid-1", {
          recent_matches: {
            matches: [{ id: "match-1" }],
            schema_version: 1,
            updated_at: serverTimestamp(),
          },
        }),
      ),
    );
  });

  it("allows only the owner to delete their profile document", async () => {
    await seedProfile("uid-1");

    await assertFails(ownerDb("uid-2").doc("profiles/uid-1").delete());
    await assertFails(testEnv.unauthenticatedContext().firestore().doc("profiles/uid-1").delete());
    await assertSucceeds(ownerDb().doc("profiles/uid-1").delete());
  });
});

describe("Firestore private match subcollection rules", () => {
  it("rejects private match subcollection creates after the embedded-history pivot", async () => {
    await seedProfile("uid-1");

    await assertFails(
      ownerDb().doc("profiles/uid-1/matches/match-1").set(
        {
          id: "match-1",
          source: "cloud_saved",
          trust: "client_uploaded",
        },
      ),
    );
  });

  it("rejects private match subcollection reads and deletes", async () => {
    await seedProfile("uid-1");
    await seedMatch("uid-1", "client-match", { trust: "client_uploaded" });

    await assertFails(ownerDb().doc("profiles/uid-1/matches/client-match").get());
    await assertFails(ownerDb().doc("profiles/uid-1/matches/client-match").delete());
  });
});
