import { describe, expect, it } from "vitest";

import { BOARD_RENDER_DEPTHS, COLOR, SPRITE } from "./constants";
import { BoardRenderer } from "./board_renderer";
import type { BoardSpriteResolver } from "./board_sprite_resolver";

type MockGameObject = {
  depth: number | null;
  scale: number | null;
  visible: boolean | null;
  setDepth: (depth: number) => MockGameObject;
  setScale: (scale: number) => MockGameObject;
  setVisible: (visible: boolean) => MockGameObject;
};

type MockContainer = {
  children: MockGameObject[];
  add: (gameObject: MockGameObject) => void;
};

function createMockContainer(): MockContainer {
  return {
    children: [],
    add(gameObject: MockGameObject) {
      this.children.push(gameObject);
    },
  };
}

function createMockGameObject(): MockGameObject {
  const gameObject: MockGameObject = {
    depth: null,
    scale: null,
    visible: null,
    setDepth(depth: number) {
      this.depth = depth;
      return this;
    },
    setScale(scale: number) {
      this.scale = scale;
      return this;
    },
    setVisible(visible: boolean) {
      this.visible = visible;
      return this;
    },
  };

  return gameObject;
}

function createMockSpriteResolver() {
  const creations: Array<{ color: number; frame: number; texture: string }> = [];
  const resolver = {
    createSprite(_x: number, _y: number, texture: string, frame: number, color: number) {
      creations.push({ color, frame, texture });
      return createMockGameObject();
    },
  } as unknown as BoardSpriteResolver;
  return { creations, resolver };
}

describe("BoardRenderer depths", () => {
  it("uses the configured board size for bounds and cell hit testing", () => {
    const renderer = new BoardRenderer({} as never, 10, 0, 0, 7);

    expect(renderer.getBounds()).toMatchObject({
      height: 70,
      width: 70,
    });
    expect(renderer.pixelToCell(60, 60)).toEqual({ row: 6, col: 6 });
    expect(renderer.pixelToCell(70, 60)).toBeNull();
  });

  it("renders the pointer below stones but above the board surface", () => {
    const scene = {
      add: {
        sprite: () => createMockGameObject(),
      },
    };
    const mock = createMockSpriteResolver();
    const renderer = new BoardRenderer(scene as never, 32, 0, 0, 15, undefined, mock.resolver);

    const stone = renderer.placeStone(7, 7, 0) as unknown as MockGameObject;
    const pointer = renderer.createPointer() as unknown as MockGameObject;

    expect(pointer.depth).toBe(BOARD_RENDER_DEPTHS.POINTER);
    expect(stone.depth).toBe(BOARD_RENDER_DEPTHS.STONE);
    expect(BOARD_RENDER_DEPTHS.BOARD).toBeLessThan(BOARD_RENDER_DEPTHS.POINTER);
    expect(BOARD_RENDER_DEPTHS.POINTER).toBeLessThan(BOARD_RENDER_DEPTHS.STONE);
    expect(mock.creations).toEqual([
      { color: COLOR.STONE_BLACK, frame: 0, texture: SPRITE.STONE },
      { color: COLOR.STONE_WHITE, frame: 0, texture: SPRITE.POINTER },
    ]);
  });

  it("can attach stones to a dedicated stone layer instead of the board layer", () => {
    const scene = {
      add: {
        sprite: () => createMockGameObject(),
      },
    };
    const boardLayer = createMockContainer();
    const stoneLayer = createMockContainer();
    const mock = createMockSpriteResolver();
    const renderer = new BoardRenderer(
      scene as never,
      32,
      0,
      0,
      15,
      boardLayer as never,
      mock.resolver,
    );

    const stone = renderer.placeStone(7, 7, 0, stoneLayer as never) as unknown as MockGameObject;

    expect(boardLayer.children).toEqual([]);
    expect(stoneLayer.children).toEqual([stone]);
  });
});
