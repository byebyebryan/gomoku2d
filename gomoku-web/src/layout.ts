export const LANDSCAPE_GAME_SIZE = { width: 1200, height: 900 } as const;
export const PORTRAIT_GAME_SIZE = { width: 900, height: 1350 } as const;

export type LayoutMode = "landscape" | "portrait";

export function getViewportSize(): { width: number; height: number } {
  const viewport = window.visualViewport;

  return {
    width: viewport?.width ?? window.innerWidth,
    height: viewport?.height ?? window.innerHeight,
  };
}

export function getLayoutMode(width: number, height: number): LayoutMode {
  return width < height ? "portrait" : "landscape";
}

export function getGameSizeForViewport(width: number, height: number): { width: number; height: number } {
  return getLayoutMode(width, height) === "portrait"
    ? PORTRAIT_GAME_SIZE
    : LANDSCAPE_GAME_SIZE;
}
