export const SEQUENCE_FONT_FAMILY = '"PixelOperator8Bold", "PixelOperatorBold", "PixelOperator", monospace';

type BoardFontSet = Pick<FontFaceSet, "load">;

const BOARD_FONT_LOADS = ["24px PixelOperator8Bold"] as const;

function defaultFontSet(): BoardFontSet | undefined {
  if (typeof document === "undefined" || !("fonts" in document)) {
    return undefined;
  }

  return document.fonts;
}

export async function loadBoardFonts(fonts: BoardFontSet | undefined = defaultFontSet()): Promise<void> {
  if (!fonts) {
    return;
  }

  await Promise.all(BOARD_FONT_LOADS.map((font) => fonts.load(font)));
}
