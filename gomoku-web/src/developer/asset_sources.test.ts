import { resolve } from "node:path";
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import manifest from "../../assets/asset_manifest.json";
import * as board from "../board/constants";
import { multiplyTintRgbaPixels } from "../board/board_sprite_resolver";
import { canonicalIcons } from "./icon_catalog";
import { spriteTints } from "./SpriteInspector";

describe("source-backed developer references", () => {
  it("retains every source-defined animation range/rate and sheet bounds", () => {
    const groups = [
      board.STONE_ANIMS,
      board.POINTER_ANIMS,
      board.HOVER_ANIMS,
      board.CAUTION_ANIMS,
      board.HIGHLIGHTER_ANIMS,
      board.MARKER_ANIMS,
      board.TRANSFORM_ANIMS,
    ];
    const animations = groups
      .flatMap((group) => Object.values(group))
      .filter((value) => "start" in value);
    expect(manifest.sprites.animations).toHaveLength(animations.length);
    for (const item of manifest.sprites.animations) {
      const runtime = animations.find((animation) => animation.key === item.name)!;
      expect(runtime).toBeDefined();
      expect([item.start, item.end, item.fps]).toEqual([
        runtime.start,
        runtime.end,
        runtime.frameRate,
      ]);
      expect(item.start).toBeGreaterThanOrEqual(0);
      expect(item.end).toBeLessThan(item.cols * item.rows);
    }
    for (const sheet of manifest.sprites.sheets) {
      const bytes = readFileSync(resolve("assets", sheet.file));
      expect([bytes.readUInt32BE(16), bytes.readUInt32BE(20)]).toEqual([
        sheet.cols * board.FRAME_SIZE,
        sheet.rows * board.FRAME_SIZE,
      ]);
    }
  });
  it("uses runtime colours with the exact alpha-preserving tint operation", () => {
    expect(spriteTints.black).toBe(board.COLOR.STONE_BLACK);
    expect(spriteTints.white).toBe(board.COLOR.STONE_WHITE);
    expect(spriteTints.red).toBe(board.COLOR.FORBIDDEN);
    expect(spriteTints.green).toBe(board.COLOR.WIN_MOVE);
    expect([
      ...multiplyTintRgbaPixels(new Uint8ClampedArray([127, 200, 255, 73]), spriteTints.black),
    ]).toEqual([32, 50, 64, 73]);
  });
  it("derives palette values from CSS and retains unequal legacy icon margins", () => {
    const css = readFileSync(resolve("src/app/global.css"), "utf8");
    for (const group of manifest.guide.palette)
      for (const token of group.tokens) expect(css).toContain(`${token.name}: ${token.value};`);
    const reset = canonicalIcons.find((icon) => icon.name === "reset")!;
    expect(new Set(reset.margins).size).toBeGreaterThan(1);
    expect(canonicalIcons.every((icon) => icon.mask.every((cell) => cell >= 0 && cell < 576))).toBe(
      true,
    );
  });
});
