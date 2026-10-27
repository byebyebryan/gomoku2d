export const PROJECT_SOURCE_URL = "https://github.com/byebyebryan/gomoku2d";

export const DEVELOPER_SECTIONS = [
  {
    to: "/visuals/",
    label: "Visual design",
    icon: "settings",
    description: "The visual language, actual controls and the pixels behind them.",
  },
  {
    to: "/dev/reviews/",
    label: "UI design",
    icon: "replay",
    description: "Inspect screen references, compare layout changes and review the findings.",
  },
  {
    to: "/lab/",
    label: "Bot lab",
    icon: "bot",
    description: "Inspect recorded experiments, search costs and where games turned.",
  },
] as const;
