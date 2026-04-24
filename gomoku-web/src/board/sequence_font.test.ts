import { describe, expect, it, vi } from "vitest";

import { loadBoardFonts } from "./sequence_font";

describe("loadBoardFonts", () => {
  it("loads the result sequence number font before canvas text is rendered", async () => {
    const fonts = {
      load: vi.fn().mockResolvedValue([]),
    };

    await loadBoardFonts(fonts);

    expect(fonts.load).toHaveBeenCalledWith("24px PixelOperator8Bold");
  });

  it("skips loading when the browser font loading API is unavailable", async () => {
    await expect(loadBoardFonts(undefined)).resolves.toBeUndefined();
  });
});
