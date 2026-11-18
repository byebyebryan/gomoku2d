import { multiplyTintRgbaPixels } from "../board/board_sprite_resolver";
import { BOARD_SIZE, COLOR, FRAME_SIZE } from "../board/constants";

export interface ReplayThumbnailPosition {
  moves: readonly number[];
  opening: number;
}

const stoneUrl = new URL("../../assets/sprites/stone.png", import.meta.url).href;
let stoneFrames: Promise<readonly HTMLCanvasElement[]> | undefined;

function prepareStone(image: HTMLImageElement, color: number): HTMLCanvasElement {
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = FRAME_SIZE;
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Canvas drawing is unavailable.");
  context.drawImage(image, 0, 0, FRAME_SIZE, FRAME_SIZE, 0, 0, FRAME_SIZE, FRAME_SIZE);
  const pixels = context.getImageData(0, 0, FRAME_SIZE, FRAME_SIZE);
  pixels.data.set(multiplyTintRgbaPixels(pixels.data, color));
  context.putImageData(pixels, 0, 0);
  return canvas;
}

export function loadThumbnailStones(): Promise<readonly HTMLCanvasElement[]> {
  stoneFrames ??= new Promise((resolve, reject) => {
    const image = new Image();
    image.onload = () => {
      try {
        resolve([prepareStone(image, COLOR.STONE_BLACK), prepareStone(image, COLOR.STONE_WHITE)]);
      } catch (error) {
        reject(error);
      }
    };
    image.onerror = () => reject(new Error("The replay thumbnail asset could not load."));
    image.src = stoneUrl;
  });
  return stoneFrames;
}

// Position comes from the existing portable replay validator. The thumbnail
// only projects its move prefix, with no analysis or gameplay rules of its own.
export function drawReplayThumbnail(
  canvas: HTMLCanvasElement,
  position: ReplayThumbnailPosition,
  cssWidth: number,
  pixelRatio: number,
  stones: readonly HTMLCanvasElement[],
): void {
  const step = Math.max(1, Math.round(cssWidth * Math.min(3, Math.max(1, pixelRatio)) / BOARD_SIZE));
  const size = BOARD_SIZE * step;
  const offset = Math.floor(step / 2);
  const grid = Math.max(1, Math.round(step / 20));
  const stone = Math.max(3, Math.round(step * 0.9));
  canvas.width = canvas.height = size;
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Canvas drawing is unavailable.");
  context.imageSmoothingEnabled = false;
  context.fillStyle = `#${COLOR.BOARD_SURFACE.toString(16)}`;
  context.fillRect(0, 0, size, size);
  context.fillStyle = "#a17d42";
  for (let i = 0; i < BOARD_SIZE; i++) {
    const line = offset + i * step;
    const span = (BOARD_SIZE - 1) * step + grid;
    context.fillRect(offset, line, span, grid);
    context.fillRect(line, offset, grid, span);
  }
  for (let i = 0; i < position.opening; i++) {
    const cell = position.moves[i]!;
    const x = offset + (cell % BOARD_SIZE) * step - Math.floor(stone / 2);
    const y = offset + Math.floor(cell / BOARD_SIZE) * step - Math.floor(stone / 2);
    context.drawImage(stones[i % 2]!, x, y, stone, stone);
  }
}
