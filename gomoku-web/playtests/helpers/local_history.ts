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
    const storageKey = "gomoku2d.guest-profile.v2";
    const stored = JSON.parse(localStorage.getItem(storageKey) ?? "{\"state\":{},\"version\":0}");
    const existingState = stored.state ?? {};
    const baseProfile = existingState.profile ?? {
      avatarUrl: null,
      createdAt: "2026-04-22T18:00:00.000Z",
      displayName: seed.displayName,
      id: "fixture-profile",
      kind: "guest",
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
        depth: 3,
        kind: "baseline",
      },
      config_version: 1,
      engine: "baseline_search",
      id: "practice_bot",
      version: 1,
    };
    const moveCells = seed.moves.map((move) => move.row * 15 + move.col);

    localStorage.setItem(
      storageKey,
      JSON.stringify({
        state: {
          history: [
            {
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
                display_name: "Practice Bot",
                kind: "bot",
                local_profile_id: null,
                profile_uid: null,
              },
              saved_at: seed.savedAt,
              schema_version: 1,
              source: "local_history",
              status: seed.status,
              trust: "local_only",
              undo_floor: 0,
              variant: seed.variant,
            },
          ],
          profile,
          settings: {
            preferredVariant: seed.preferredVariant,
          },
        },
        version: 0,
      }),
    );
  }, input);
}
