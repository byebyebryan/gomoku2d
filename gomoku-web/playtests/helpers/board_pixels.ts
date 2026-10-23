import { expect, type Locator, type Page } from "@playwright/test";

export type BoardRendererMode = "canvas" | "webgl";
export type Rgb = readonly [number, number, number];

declare global {
  interface Window {
    __boardPixelContexts?: WeakMap<HTMLCanvasElement, string>;
  }
}

/** Observe the context actually used by the displayed canvas, not a feature probe. */
export async function prepareBoardPixels(page: Page, mode: BoardRendererMode): Promise<void> {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.addInitScript((forceCanvas) => {
    const original = HTMLCanvasElement.prototype.getContext;
    window.__boardPixelContexts = new WeakMap();
    HTMLCanvasElement.prototype.getContext = function (this: HTMLCanvasElement, kind: string, ...args: unknown[]) {
      if (forceCanvas && kind.includes("webgl")) return null;
      const context = Reflect.apply(original, this, [kind, ...args]);
      if (context) window.__boardPixelContexts!.set(this, kind);
      return context;
    } as typeof original;
  }, mode === "canvas");
}

export async function expectBoardRenderer(canvas: Locator, mode: BoardRendererMode): Promise<void> {
  await expect(canvas).toBeVisible();
  await expect.poll(() => canvas.evaluate((element) => (
    window.__boardPixelContexts?.get(element as HTMLCanvasElement) ?? "uninitialized"
  ))).toBe(mode === "canvas" ? "2d" : "webgl");
}

/** Decode compositor pixels; works with WebGL's non-preserved drawing buffer. */
export async function cellPixels(canvas: Locator, row: number, col: number, size = 15): Promise<number[]> {
  const screenshot = await canvas.screenshot();
  return canvas.page().evaluate(async ({ image, row, col, size }) => {
    const source = new Image();
    source.src = `data:image/png;base64,${image}`;
    await source.decode();
    const decoded = document.createElement("canvas");
    decoded.width = source.width;
    decoded.height = source.height;
    const context = decoded.getContext("2d")!;
    context.drawImage(source, 0, 0);
    const cell = Math.min(decoded.width, decoded.height) / size;
    const x = (decoded.width - (size - 1) * cell) / 2 + col * cell;
    const y = (decoded.height - (size - 1) * cell) / 2 + row * cell;
    const inset = cell * 0.45;
    return Array.from(context.getImageData(
      Math.ceil(x - inset), Math.ceil(y - inset),
      Math.max(1, Math.floor(inset * 2)), Math.max(1, Math.floor(inset * 2)),
    ).data);
  }, { image: screenshot.toString("base64"), row, col, size });
}

export async function expectCellColour(canvas: Locator, row: number, col: number, colour: Rgb, size = 15): Promise<void> {
  // Allow only one channel step for GPU multiplication/compositor rounding.
  await expect.poll(async () => {
    const pixels = await cellPixels(canvas, row, col, size);
    let matching = 0;
    for (let index = 0; index < pixels.length; index += 4) {
      if (pixels[index + 3] === 255 && colour.every((channel, offset) => Math.abs(pixels[index + offset]! - channel) <= 1)) {
        matching += 1;
      }
    }
    return matching;
  }, { message: `Cell ${row},${col} must contain the expected RGB ${colour.join(",")}` }).toBeGreaterThan(0);
}

/** Every animated frame must keep a Black sprite dark, including texture swaps. */
export async function observeBlackAnimation(canvas: Locator, row: number, col: number, durationMs: number) {
  return canvas.evaluate(async (element, { row, col, durationMs }) => {
    const target = element as HTMLCanvasElement;
    const kind = window.__boardPixelContexts!.get(target);
    const context = kind === "2d" ? target.getContext("2d") : null;
    const gl = kind === "webgl" ? target.getContext("webgl") : null;
    if (!context && !gl) throw new Error("Expected an initialized Canvas or WebGL board");
    const cell = Math.min(target.width, target.height) / 15;
    const side = Math.max(1, Math.floor(cell * 0.9));
    const left = Math.ceil((target.width - 14 * cell) / 2 + col * cell - cell * 0.45);
    const top = Math.ceil((target.height - 14 * cell) / 2 + row * cell - cell * 0.45);
    const hashes = new Set<number>();
    let frames = 0;
    let maximumGrey = 0;
    let blackPixels = 0;
    const start = performance.now();
    await new Promise<void>((resolve) => {
      const sample = () => {
        let data: Uint8Array | Uint8ClampedArray;
        if (context) data = context.getImageData(left, top, side, side).data;
        else {
          data = new Uint8Array(side * side * 4);
          gl!.readPixels(left, target.height - top - side, side, side, gl!.RGBA, gl!.UNSIGNED_BYTE, data);
        }
        let hash = 2166136261;
        for (let index = 0; index < data.length; index += 4) {
          const r = data[index]!, g = data[index + 1]!, b = data[index + 2]!;
          hash = Math.imul(hash ^ r ^ (g << 8) ^ (b << 16), 16777619);
          if (data[index + 3] === 255 && Math.abs(r - g) <= 1 && Math.abs(g - b) <= 1) {
            maximumGrey = Math.max(maximumGrey, r, g, b);
            if (r >= 63 && r <= 65) blackPixels += 1;
          }
        }
        hashes.add(hash);
        frames += 1;
        if (performance.now() - start >= durationMs) resolve();
        else requestAnimationFrame(sample);
      };
      requestAnimationFrame(sample);
    });
    return { frames, distinctFrames: hashes.size, maximumGrey, blackPixels, glError: gl?.getError() ?? 0 };
  }, { row, col, durationMs });
}
