import Phaser from "phaser";

// All measurements in source pixels (pre-scale). button sprites are 18×18: 8px corner + 2px center + 8px corner.
const BUTTON_BORDER  = 8;
const HOVER_OFFSET   = 1;  // button_1 surface starts 1 source px below button_0
const PRESS_OFFSET   = 3;  // button_2 surface starts 3 source px below button_0
const PAD_H_SRC      = 6;  // horizontal text indent inside card
const PAD_V_SRC      = 10; // vertical padding above/below text
const GAP_SRC        = 3;  // gap between name and wins lines
const FONT_PX        = 16; // fixed display size — matches atlas exactly (1:1, no scaling)

export interface PlayerInfo {
  name: string;
  wins: number;
  isHuman: boolean;
}

export class PlayerCard {
  private container: Phaser.GameObjects.Container;
  private bgNormal!: Phaser.GameObjects.NineSlice;
  private bgActive!: Phaser.GameObjects.NineSlice;
  private nameText!: Phaser.GameObjects.BitmapText;
  private winsText!: Phaser.GameObjects.BitmapText;
  private isActive: boolean = false;
  private textNeutralY: number = 0;
  private textRaisedY: number = 0;
  private winsOffsetY: number = 0;
  readonly height: number; // canvas pixels, for layout stacking

  constructor(
    scene: Phaser.Scene,
    x: number,
    y: number,
    playerColor: 0 | 1,
    player: PlayerInfo,
    scale: number,
    cardWidth: number, // canvas pixels — caller drives width to fill the sidebar
  ) {
    const padH       = Math.round(PAD_H_SRC * scale);
    const padV       = Math.round(PAD_V_SRC * scale);
    const gap        = Math.round(GAP_SRC * scale);
    const nameFontPx = FONT_PX;
    const subFontPx  = FONT_PX;

    const textTint = playerColor === 0 ? 0xffffff : 0x1a1a2e;
    const subTint  = playerColor === 0 ? 0xaaaaaa : 0x555555;

    // Create texts first so we can measure rendered bounds (canvas pixels).
    this.nameText = scene.add.bitmapText(0, 0, "minecraft", player.name, nameFontPx)
      .setTint(textTint);
    this.winsText = scene.add.bitmapText(0, 0, "minecraft", `${player.wins} wins`, subFontPx)
      .setTint(subTint);

    const nB = this.nameText.getBounds();
    const wB = this.winsText.getBounds();

    // Width is caller-specified; height wraps content with vertical padding.
    const cardW = cardWidth;
    const cardH = Math.round(nB.height + gap + wB.height + 2 * padV);
    this.height = cardH;

    // NineSlice width/height are given in source pixels; setScale(scale) brings
    // them to canvas size so corners render at the same density as game sprites.
    const nsW = cardW / scale;
    const nsH = cardH / scale;
    const tint = playerColor === 0 ? 0x404040 : 0xffffff;

    this.bgNormal = scene.add.nineslice(
      0, 0, "button_0", undefined,
      nsW, nsH,
      BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER,
    ).setScale(scale).setTint(tint);

    this.bgActive = scene.add.nineslice(
      0, 0, "button_2", undefined,
      nsW, nsH,
      BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER,
    ).setScale(scale).setTint(tint).setVisible(false);

    // Position texts (canvas pixels, relative to container center).
    const left = -cardW / 2 + padH;
    const top  = -cardH / 2 + padV;
    this.textNeutralY = top;
    this.textRaisedY  = top - Math.round(PRESS_OFFSET * scale);
    this.winsOffsetY  = nB.height + gap;
    // Start in inactive state: button_0 surface, content raised up
    this.nameText.setPosition(left, this.textRaisedY);
    this.winsText.setPosition(left, this.textRaisedY + this.winsOffsetY);

    this.container = scene.add.container(x, y, [
      this.bgNormal, this.bgActive, this.nameText, this.winsText,
    ]);
    this.container.setDepth(20);
  }

  setPosition(x: number, y: number): void {
    this.container.setPosition(x, y);
  }

  setActive(active: boolean): void {
    if (this.isActive === active) return;
    this.isActive = active;
    this.bgNormal.setVisible(!active);
    this.bgActive.setVisible(active);
    const textY = active ? this.textNeutralY : this.textRaisedY;
    this.nameText.setY(textY);
    this.winsText.setY(textY + this.winsOffsetY);
  }

  setWins(wins: number): void {
    this.winsText.setText(`${wins} wins`);
  }

  setName(name: string): void {
    this.nameText.setText(name);
  }
}

export class ResetButton {
  private container: Phaser.GameObjects.Container;
  readonly height: number; // canvas pixels

  constructor(
    scene: Phaser.Scene,
    x: number,
    y: number,
    onClick: () => void,
    scale: number,
    width: number, // canvas pixels — match card width
  ) {
    const pad    = Math.round(PAD_H_SRC * scale);
    const fontPx = FONT_PX;

    const label = scene.add.bitmapText(0, 0, "minecraft", "RESET", fontPx)
      .setTint(0xffffff)
      .setOrigin(0.5);

    const b    = label.getBounds();
    const btnW = width;
    const btnH = Math.round(b.height + 2 * pad);
    this.height = btnH;

    const nsW = btnW / scale; // source pixels for NineSlice
    const nsH = btnH / scale;

    const normal  = scene.add.nineslice(0, 0, "button_0", undefined, nsW, nsH, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER).setScale(scale).setTint(0xcc2222);
    const hover   = scene.add.nineslice(0, 0, "button_1", undefined, nsW, nsH, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER).setScale(scale).setTint(0xff3333).setVisible(false);
    const pressed = scene.add.nineslice(0, 0, "button_2", undefined, nsW, nsH, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER).setScale(scale).setTint(0xaa1111).setVisible(false);

    this.container = scene.add.container(x, y, [normal, hover, pressed, label]);
    this.container.setDepth(20);
    this.container.setSize(btnW, btnH); // canvas pixels for hit zone
    this.container.setInteractive({ useHandCursor: true });

    const labelPressedY = 0;                                                         // button_2: centered
    const labelHoverY   = -Math.round((PRESS_OFFSET - HOVER_OFFSET) * scale);       // button_1: 2px up
    const labelBaseY    = -Math.round(PRESS_OFFSET * scale);                        // button_0: 3px up
    label.setY(labelBaseY);

    this.container.on("pointerover",  () => { normal.setVisible(false); hover.setVisible(true);  pressed.setVisible(false); label.setY(labelHoverY);   });
    this.container.on("pointerout",   () => { normal.setVisible(true);  hover.setVisible(false); pressed.setVisible(false); label.setY(labelBaseY);    });
    this.container.on("pointerdown",  () => { normal.setVisible(false); hover.setVisible(false); pressed.setVisible(true);  label.setY(labelPressedY); });
    this.container.on("pointerup",    () => { onClick(); normal.setVisible(false); hover.setVisible(true); pressed.setVisible(false); label.setY(labelHoverY); });
  }

  setPosition(x: number, y: number): void {
    this.container.setPosition(x, y);
  }
}
