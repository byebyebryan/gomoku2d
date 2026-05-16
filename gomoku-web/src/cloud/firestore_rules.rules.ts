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
const matchHistoryLimit = 128;
const summaryMatchesLimit = 1024;

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
