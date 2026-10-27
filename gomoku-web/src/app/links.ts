export const PROJECT_SOURCE_URL = "https://github.com/byebyebryan/gomoku2d";
export const DEV_SITE_LABEL = "Dev site";
export const UI_DESIGN_ENTRY_PATH = "/dev/reviews/core-screens-062/";

export const DEVELOPER_SECTIONS = [
  {
    to: "/visuals/",
    match: "/visuals/*",
    label: "Visual design",
    icon: "settings",
    description: "The visual language, actual controls and the pixels behind them.",
  },
  {
    to: UI_DESIGN_ENTRY_PATH,
    match: "/dev/reviews/*",
    label: "UI design",
    icon: "replay",
    description: "Inspect screen references, compare layout changes and review the findings.",
  },
  {
    to: "/lab/",
    match: "/lab/*",
    label: "Bot lab",
    icon: "bot",
    description: "Inspect recorded experiments, search costs and where games turned.",
  },
] as const;
