import { emptyLocalMatchHistory } from "../profile/local_profile_store";
import {
  createDefaultProfileSettings,
  type ProfileSettings,
} from "../profile/profile_settings";

export const testLocalProfile = {
  avatarUrl: null,
  createdAt: "2026-05-15T00:00:00.000Z",
  displayName: "Bryan",
  id: "local-1",
  kind: "local" as const,
  updatedAt: "2026-05-15T00:00:00.000Z",
  username: null,
};

export const noOpBotRunner = {
  chooseMove: async () => null,
  configure: () => undefined,
  dispose: () => undefined,
};

export function createLocalProfileTestState(
  settings: ProfileSettings = createDefaultProfileSettings(),
) {
  return {
    matchHistory: emptyLocalMatchHistory(),
    profile: testLocalProfile,
    settings,
  };
}
