import { describe, expect, it } from "vitest";

import {
  DEFAULT_PRACTICE_BOT_CONFIG,
  labSpecForPracticeBot,
  practiceBotConfigSummary,
  practiceBotPlayerName,
  resolvePracticeBotConfig,
  sanitizePracticeBotConfig,
} from "./practice_bot_config";

describe("practice bot config", () => {
  it("defaults to the Normal preset", () => {
    expect(DEFAULT_PRACTICE_BOT_CONFIG).toEqual({
      mode: "preset",
      preset: "normal",
      version: 1,
    });
    expect(labSpecForPracticeBot(DEFAULT_PRACTICE_BOT_CONFIG)).toBe("search-d3+pattern-eval");
    expect(practiceBotConfigSummary(DEFAULT_PRACTICE_BOT_CONFIG)).toBe("D3 · full · pattern");
    expect(practiceBotPlayerName(DEFAULT_PRACTICE_BOT_CONFIG)).toBe("Normal Bot");
  });

  it("formats practice bot configs as player names", () => {
    expect(practiceBotPlayerName({ mode: "preset", preset: "easy", version: 1 })).toBe("Easy Bot");
    expect(practiceBotPlayerName({ mode: "preset", preset: "hard", version: 1 })).toBe("Hard Bot");
    expect(practiceBotPlayerName({
      corridorProof: true,
      depth: 5,
      mode: "custom",
      patternScoring: true,
      version: 1,
      width: 16,
    })).toBe("Custom Bot");
  });

  it("resolves the report-backed presets to structured worker specs", () => {
    expect(resolvePracticeBotConfig({ mode: "preset", preset: "easy", version: 1 })).toMatchObject({
      childLimit: null,
      corridorProof: null,
      depth: 1,
      kind: "search",
      patternEval: false,
    });
    expect(resolvePracticeBotConfig({ mode: "preset", preset: "hard", version: 1 })).toMatchObject({
      childLimit: 8,
      corridorProof: {
        candidateLimit: 16,
        depth: 8,
        width: 4,
      },
      depth: 7,
      kind: "search",
      patternEval: true,
    });
  });

  it("keeps custom configs constrained and derives lab specs from product state", () => {
    const config = sanitizePracticeBotConfig({
      corridorProof: true,
      depth: 5,
      labSpec: "search-d999+no-safety",
      mode: "custom",
      patternScoring: true,
      version: 1,
      width: 16,
    });

    expect(config).toEqual({
      corridorProof: true,
      depth: 5,
      mode: "custom",
      patternScoring: true,
      version: 1,
      width: 16,
    });
    expect(labSpecForPracticeBot(config)).toBe(
      "search-d5+tactical-cap-16+pattern-eval+corridor-proof-c16-d8-w4",
    );
    expect(practiceBotConfigSummary(config)).toBe("D5 · W16 · pattern · proof");
  });

  it("clamps custom widths that are too expensive for browser play", () => {
    expect(sanitizePracticeBotConfig({
      corridorProof: false,
      depth: 5,
      mode: "custom",
      patternScoring: true,
      version: 1,
      width: "none",
    })).toMatchObject({
      depth: 5,
      width: 16,
    });
    expect(sanitizePracticeBotConfig({
      corridorProof: false,
      depth: 7,
      mode: "custom",
      patternScoring: true,
      version: 1,
      width: 16,
    })).toMatchObject({
      depth: 7,
      width: 8,
    });
  });

  it("sanitizes unknown persisted values to Normal", () => {
    expect(sanitizePracticeBotConfig(null)).toEqual(DEFAULT_PRACTICE_BOT_CONFIG);
    expect(sanitizePracticeBotConfig({ mode: "preset", preset: "deep", version: 1 })).toEqual(
      DEFAULT_PRACTICE_BOT_CONFIG,
    );
    expect(sanitizePracticeBotConfig({
      corridorProof: false,
      depth: 9,
      mode: "custom",
      patternScoring: true,
      version: 1,
      width: 32,
    })).toEqual(DEFAULT_PRACTICE_BOT_CONFIG);
  });
});
