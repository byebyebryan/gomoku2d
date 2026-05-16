import { describe, expect, it } from "vitest";

import {
  DEFAULT_BOT_CONFIG,
  botConfigSummary,
  botPlayerName,
  labSpecForBot,
  resolveBotConfig,
  sanitizeBotConfig,
} from "./bot_config";

describe("bot config", () => {
  it("defaults to the Normal preset", () => {
    expect(DEFAULT_BOT_CONFIG).toEqual({
      mode: "preset",
      preset: "normal",
      version: 1,
    });
    expect(labSpecForBot(DEFAULT_BOT_CONFIG)).toBe("search-d3+pattern-eval");
    expect(botConfigSummary(DEFAULT_BOT_CONFIG)).toBe("D3 · full · pattern");
    expect(botPlayerName(DEFAULT_BOT_CONFIG)).toBe("Normal Bot");
  });

  it("formats bot configs as player names", () => {
    expect(botPlayerName({ mode: "preset", preset: "easy", version: 1 })).toBe("Easy Bot");
    expect(botPlayerName({ mode: "preset", preset: "hard", version: 1 })).toBe("Hard Bot");
    expect(botPlayerName({
      depth: 5,
      extraPass: "corridor_proof",
      mode: "custom",
      scoring: "pattern",
      version: 1,
      width: 16,
    })).toBe("Custom Bot");
  });

  it("resolves the report-backed presets to structured worker specs", () => {
    expect(resolveBotConfig({ mode: "preset", preset: "easy", version: 1 })).toMatchObject({
      childLimit: null,
      corridorProof: null,
      depth: 1,
      kind: "search",
      patternEval: false,
    });
    expect(resolveBotConfig({ mode: "preset", preset: "hard", version: 1 })).toMatchObject({
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
    const config = sanitizeBotConfig({
      depth: 5,
      extraPass: "corridor_proof",
      labSpec: "search-d999+no-safety",
      mode: "custom",
      scoring: "pattern",
      version: 1,
      width: 16,
    });

    expect(config).toEqual({
      depth: 5,
      extraPass: "corridor_proof",
      mode: "custom",
      scoring: "pattern",
      version: 1,
      width: 16,
    });
    expect(labSpecForBot(config)).toBe(
      "search-d5+tactical-cap-16+pattern-eval+corridor-proof-c16-d8-w4",
    );
    expect(botConfigSummary(config)).toBe("D5 · W16 · pattern · proof");
  });

  it("clamps custom widths that are too expensive for browser play", () => {
    expect(sanitizeBotConfig({
      depth: 5,
      extraPass: "none",
      mode: "custom",
      scoring: "pattern",
      version: 1,
      width: "full",
    })).toMatchObject({
      depth: 5,
      width: 16,
    });
    expect(sanitizeBotConfig({
      depth: 7,
      extraPass: "none",
      mode: "custom",
      scoring: "pattern",
      version: 1,
      width: 16,
    })).toMatchObject({
      depth: 7,
      width: 8,
    });
  });

  it("sanitizes unknown persisted values to Normal", () => {
    expect(sanitizeBotConfig(null)).toEqual(DEFAULT_BOT_CONFIG);
    expect(sanitizeBotConfig({ mode: "preset", preset: "deep", version: 1 })).toEqual(
      DEFAULT_BOT_CONFIG,
    );
    expect(sanitizeBotConfig({
      depth: 9,
      extraPass: "none",
      mode: "custom",
      scoring: "pattern",
      version: 1,
      width: 32,
    })).toEqual(DEFAULT_BOT_CONFIG);
  });
});
