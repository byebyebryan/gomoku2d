import { describe, expect, it } from "vitest";

import { BOARD_RENDER_DEPTHS } from "./constants";
import { BoardRenderer } from "./board_renderer";

type MockGameObject = {
  depth: number | null;
  scale: number | null;
  tint: number | null;
  visible: boolean | null;
  setDepth: (depth: number) => MockGameObject;
  setScale: (scale: number) => MockGameObject;
  setTint: (tint: number) => MockGameObject;
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
    tint: null,
    visible: null,
    setDepth(depth: number) {
      this.depth = depth;
      return this;
    },
    setScale(scale: number) {
      this.scale = scale;
      return this;
    },
    setTint(tint: number) {
      this.tint = tint;
      return this;
    },
    setVisible(visible: boolean) {
      this.visible = visible;
      return this;
    },
  };

  return gameObject;
}

describe("BoardRenderer depths", () => {
  it("renders the pointer below stones but above the board surface", () => {
    const scene = {
      add: {
        sprite: () => createMockGameObject(),
      },
    };
    const renderer = new BoardRenderer(scene as never, 32, 0, 0);

    const stone = renderer.placeStone(7, 7, 0) as unknown as MockGameObject;
    const pointer = renderer.createPointer() as unknown as MockGameObject;

    expect(pointer.depth).toBe(BOARD_RENDER_DEPTHS.POINTER);
    expect(stone.depth).toBe(BOARD_RENDER_DEPTHS.STONE);
    expect(BOARD_RENDER_DEPTHS.BOARD).toBeLessThan(BOARD_RENDER_DEPTHS.POINTER);
    expect(BOARD_RENDER_DEPTHS.POINTER).toBeLessThan(BOARD_RENDER_DEPTHS.STONE);
  });

  it("can attach stones to a dedicated stone layer instead of the board layer", () => {
    const scene = {
      add: {
        sprite: () => createMockGameObject(),
      },
    };
    const boardLayer = createMockContainer();
    const stoneLayer = createMockContainer();
    const renderer = new BoardRenderer(scene as never, 32, 0, 0, boardLayer as never);

    const stone = renderer.placeStone(7, 7, 0, stoneLayer as never) as unknown as MockGameObject;

    expect(boardLayer.children).toEqual([]);
    expect(stoneLayer.children).toEqual([stone]);
  });
});
