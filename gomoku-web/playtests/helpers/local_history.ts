import type { Page } from "@playwright/test";

export interface SeedLocalSavedMatchInput {
  displayName: string;
  id: string;
  moves: Array<{ col: number; row: number }>;
  preferredVariant: "freestyle" | "renju";
  savedAt: string;
  status: "black_won" | "white_won" | "draw";
  variant: "freestyle" | "renju";
}

export async function seedLocalSavedMatch(page: Page, input: SeedLocalSavedMatchInput) {
  await page.evaluate((seed) => {
    const storageKey = "gomoku2d.local-profile.v5";
    const emptyCounter = () => ({
      draws: 0,
      losses: 0,
      matches: 0,
      moves: 0,
      wins: 0,
    });
    const stored = JSON.parse(localStorage.getItem(storageKey) ?? "{\"state\":{},\"version\":0}");
    const existingState = stored.state ?? {};
    const baseProfile = existingState.profile ?? {
      avatarUrl: null,
      createdAt: "2026-04-22T18:00:00.000Z",
      displayName: seed.displayName,
      id: "fixture-profile",
      kind: "local",
      updatedAt: seed.savedAt,
      username: null,
    };
    const profile = {
      ...baseProfile,
      displayName: seed.displayName,
      updatedAt: seed.savedAt,
    };
    const practiceBot = {
      config: {
        mode: "preset",
        preset: "normal",
        version: 1,
      },
      config_version: 1,
      engine: "search_bot",
      id: "bot",
      label: "Normal",
      lab_spec: "search-d3+pattern-eval",
      version: 1,
    };
    const moveCells = seed.moves.map((move) => move.row * 15 + move.col);
    const match = {
      board_size: 15,
      id: seed.id,
      match_kind: "local_vs_bot",
      move_cells: moveCells,
      move_count: moveCells.length,
      player_black: {
        bot: null,
        display_name: seed.displayName,
        kind: "human",
        local_profile_id: profile.id,
        profile_uid: null,
      },
      player_white: {
        bot: practiceBot,
        display_name: "Normal Bot",
        kind: "bot",
        local_profile_id: null,
        profile_uid: null,
      },
      saved_at: seed.savedAt,
      schema_version: 2,
      source: "local_history",
      status: seed.status,
      trust: "local_only",
      undo_floor: 0,
      ruleset: seed.variant,
    };

    localStorage.setItem(
      storageKey,
      JSON.stringify({
        state: {
          matchHistory: {
            archivedStats: {
              archived_before: null,
              archived_count: 0,
              by_opponent_type: {
                bot: emptyCounter(),
                human: emptyCounter(),
              },
              by_ruleset: {
                freestyle: emptyCounter(),
                renju: emptyCounter(),
              },
              by_side: {
                black: emptyCounter(),
                white: emptyCounter(),
              },
              schema_version: 1,
              totals: emptyCounter(),
            },
            replayMatches: [match],
            summaryMatches: [],
          },
          profile,
          settings: {
            boardHints: {
              immediate: "win_threat",
              imminent: "threat_counter",
            },
            botConfig: {
              mode: "preset",
              preset: "normal",
              version: 1,
            },
            gameConfig: {
              opening: "standard",
              ruleset: seed.preferredVariant,
            },
            touchControl: "touchpad",
          },
        },
        version: 5,
      }),
    );
  }, input);
}
