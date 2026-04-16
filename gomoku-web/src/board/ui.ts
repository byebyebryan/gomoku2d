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
  private pendingWinText!: Phaser.GameObjects.BitmapText;
  private timerText!: Phaser.GameObjects.BitmapText;
  private pendingTimerText!: Phaser.GameObjects.BitmapText;
  private isActive: boolean = false;
  private textNeutralY: number = 0;
  private textRaisedY: number = 0;
  private winsOffsetY: number = 0;
  private timerOffsetY: number = 0;
  private textLeft: number = 0;
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
    this.nameText = scene.add.bitmapText(0, 0, "pixel", player.name, nameFontPx)
      .setTint(textTint);
    this.winsText = scene.add.bitmapText(0, 0, "pixel", `${player.wins} wins`, subFontPx)
      .setTint(subTint);
    this.pendingWinText = scene.add.bitmapText(0, 0, "pixel", "+1", subFontPx)
      .setTint(0x44dd44)
      .setVisible(false);
    this.timerText = scene.add.bitmapText(0, 0, "pixel", "0:00", subFontPx)
      .setTint(subTint);
    this.pendingTimerText = scene.add.bitmapText(0, 0, "pixel", "", subFontPx)
      .setTint(0xffcc44)
      .setVisible(false);

    const nB = this.nameText.getBounds();
    const wB = this.winsText.getBounds();
    const tB = this.timerText.getBounds();

    // Width is caller-specified; height wraps content with vertical padding.
    const cardW = cardWidth;
    const cardH = Math.round(nB.height + gap + wB.height + gap + tB.height + 2 * padV);
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
    this.textLeft     = left;
    this.textNeutralY = top;
    this.textRaisedY  = top - Math.round(PRESS_OFFSET * scale);
    this.winsOffsetY  = nB.height + gap;
    this.timerOffsetY = nB.height + gap + wB.height + gap;
    // Start in inactive state: button_0 surface, content raised up
    this.nameText.setPosition(left, this.textRaisedY);
    this.winsText.setPosition(left, this.textRaisedY + this.winsOffsetY);
    this.pendingWinText.setPosition(left, this.textRaisedY + this.winsOffsetY);
    this.timerText.setPosition(left, this.textRaisedY + this.timerOffsetY);
    this.pendingTimerText.setPosition(left, this.textRaisedY + this.timerOffsetY);

    this.container = scene.add.container(x, y, [
      this.bgNormal, this.bgActive,
      this.nameText,
      this.winsText, this.pendingWinText,
      this.timerText, this.pendingTimerText,
    ]);
    this.container.setDepth(20);
  }

  setPosition(x: number, y: number): void {
    this.container.setPosition(x, y);
  }

  setVisible(v: boolean): void {
    this.container.setVisible(v);
  }

  setActive(active: boolean): void {
    if (this.isActive === active) return;
    this.isActive = active;
    this.bgNormal.setVisible(!active);
    this.bgActive.setVisible(active);
    const textY = active ? this.textNeutralY : this.textRaisedY;
    this.nameText.setY(textY);
    this.winsText.setY(textY + this.winsOffsetY);
    this.pendingWinText.setY(textY + this.winsOffsetY);
    this.timerText.setY(textY + this.timerOffsetY);
    this.pendingTimerText.setY(textY + this.timerOffsetY);
  }

  setWins(wins: number): void {
    this.winsText.setText(`${wins} wins`);
    this.pendingWinText.setVisible(false);
  }

  showPendingWin(baseWins: number): void {
    this.winsText.setText(`${baseWins} wins`);
    this.showPendingText(this.winsText, this.pendingWinText, this.winsOffsetY);
  }

  private showPendingText(
    primary: Phaser.GameObjects.BitmapText,
    secondary: Phaser.GameObjects.BitmapText,
    offsetY: number,
  ): void {
    secondary.setX(this.textLeft + primary.width + 4);
    secondary.setY((this.isActive ? this.textNeutralY : this.textRaisedY) + offsetY);
    secondary.setVisible(true);
  }

  setName(name: string): void {
    this.nameText.setText(name);
  }

  setTimer(formatted: string): void {
    this.timerText.setText(formatted);
    this.pendingTimerText.setVisible(false);
  }

  showPendingTimer(base: string, delta: string): void {
    this.timerText.setText(base);
    this.pendingTimerText.setText(delta);
    this.showPendingText(this.timerText, this.pendingTimerText, this.timerOffsetY);
  }
}

export class TextButton {
  container: Phaser.GameObjects.Container;
  readonly height: number;

  constructor(
    scene: Phaser.Scene,
    x: number,
    y: number,
    text: string,
    tints: [number, number, number], // normal, hover, pressed
    onClick: () => void,
    scale: number,
    width: number,
  ) {
    const pad    = Math.round(PAD_H_SRC * scale);
    const fontPx = FONT_PX;

    const label = scene.add.bitmapText(0, 0, "pixel", text, fontPx)
      .setTint(0xffffff)
      .setOrigin(0.5);

    const b    = label.getBounds();
    const btnW = width;
    const btnH = Math.round(b.height + 2 * pad);
    this.height = btnH;

    const nsW = btnW / scale;
    const nsH = btnH / scale;

    const normal  = scene.add.nineslice(0, 0, "button_0", undefined, nsW, nsH, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER).setScale(scale).setTint(tints[0]);
    const hover   = scene.add.nineslice(0, 0, "button_1", undefined, nsW, nsH, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER).setScale(scale).setTint(tints[1]).setVisible(false);
    const pressed = scene.add.nineslice(0, 0, "button_2", undefined, nsW, nsH, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER).setScale(scale).setTint(tints[2]).setVisible(false);

    this.container = scene.add.container(x, y, [normal, hover, pressed, label]);
    this.container.setDepth(20);
    this.container.setSize(btnW, btnH);
    this.container.setInteractive({ useHandCursor: true });

    const labelPressedY = 0;
    const labelHoverY   = -Math.round((PRESS_OFFSET - HOVER_OFFSET) * scale);
    const labelBaseY    = -Math.round(PRESS_OFFSET * scale);
    label.setY(labelBaseY);

    this.container.on("pointerover",  () => { normal.setVisible(false); hover.setVisible(true);  pressed.setVisible(false); label.setY(labelHoverY);   });
    this.container.on("pointerout",   () => { normal.setVisible(true);  hover.setVisible(false); pressed.setVisible(false); label.setY(labelBaseY);    });
    this.container.on("pointerdown",  () => { normal.setVisible(false); hover.setVisible(false); pressed.setVisible(true);  label.setY(labelPressedY); });
    this.container.on("pointerup",    () => { onClick(); normal.setVisible(false); hover.setVisible(true); pressed.setVisible(false); label.setY(labelHoverY); });
  }

  setPosition(x: number, y: number): void {
    this.container.setPosition(x, y);
  }

  setVisible(v: boolean): void {
    this.container.setVisible(v);
  }
}

const RED_TINTS: [number, number, number] = [0xcc2222, 0xff3333, 0xaa1111];
const GREEN_TINTS: [number, number, number] = [0x22aa44, 0x33cc55, 0x118833];

export class ResetButton extends TextButton {
  constructor(
    scene: Phaser.Scene,
    x: number,
    y: number,
    onClick: () => void,
    scale: number,
    width: number,
  ) {
    super(scene, x, y, "RESET", RED_TINTS, onClick, scale, width);
  }
}

export class SettingsButton extends TextButton {
  constructor(
    scene: Phaser.Scene,
    x: number,
    y: number,
    onClick: () => void,
    scale: number,
    width: number,
  ) {
    super(scene, x, y, "SETTINGS", GREEN_TINTS, onClick, scale, width);
  }
}

export class ToggleGroup {
  container: Phaser.GameObjects.Container;
  private buttons: { container: Phaser.GameObjects.Container; normal: Phaser.GameObjects.NineSlice; hover: Phaser.GameObjects.NineSlice; pressed: Phaser.GameObjects.NineSlice; label: Phaser.GameObjects.BitmapText; labelBaseY: number; labelHoverY: number }[] = [];
  private selectedIdx: number;
  private scale: number;
  private onSelectedClick?: (idx: number) => void;
  readonly height: number;

  constructor(
    scene: Phaser.Scene,
    x: number,
    y: number,
    options: string[],
    selectedIdx: number,
    scale: number,
    width: number,
    vertical: boolean = false,
    onSelectedClick?: (idx: number) => void,
  ) {
    this.onSelectedClick = onSelectedClick;
    this.selectedIdx = selectedIdx;
    this.scale = scale;
    const pad     = Math.round(PAD_H_SRC * scale);
    const fontPx  = FONT_PX;
    const btnGap  = 0.2 * scale;

    // Measure button height with a temporary text, then destroy it
    const measure = scene.add.bitmapText(0, 0, "pixel", options[0], fontPx);
    const b = measure.getBounds();
    measure.destroy();
    const btnH = Math.round(b.height + 2 * pad);
    const btnW = width;

    this.container = scene.add.container(x, y);
    this.container.setDepth(20);

    if (vertical) {
      this.height = options.length * btnH + (options.length - 1) * btnGap;
    } else {
      this.height = btnH;
    }

    for (let i = 0; i < options.length; i++) {
      const nsW = btnW / scale;
      const nsH = btnH / scale;

      const isSelected = i === selectedIdx;
      const tint = isSelected ? 0x44aa66 : 0x666666;
      const hoverTint = 0x888888;

      const normal  = scene.add.nineslice(0, 0, "button_0", undefined, nsW, nsH, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER).setScale(scale).setTint(tint);
      const hover   = scene.add.nineslice(0, 0, "button_1", undefined, nsW, nsH, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER).setScale(scale).setTint(hoverTint).setVisible(false);
      const pressed = scene.add.nineslice(0, 0, "button_2", undefined, nsW, nsH, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER, BUTTON_BORDER).setScale(scale).setTint(tint).setVisible(false);

      const optLabel = scene.add.bitmapText(0, 0, "pixel", options[i], fontPx)
        .setTint(0xffffff)
        .setOrigin(0.5);

      const labelBaseY  = -Math.round(PRESS_OFFSET * scale);
      const labelHoverY = -Math.round((PRESS_OFFSET - HOVER_OFFSET) * scale);
      optLabel.setY(labelBaseY);

      if (isSelected) {
        normal.setVisible(false);
        hover.setVisible(false);
        pressed.setVisible(true);
        optLabel.setY(0);
      }

      const btnContainer = scene.add.container(0, 0, [normal, hover, pressed, optLabel]);
      btnContainer.setSize(btnW, btnH);
      btnContainer.setInteractive({ useHandCursor: true });

      const idx = i;
      btnContainer.on("pointerover",  () => {
        if (idx === this.selectedIdx) return;
        normal.setVisible(false); hover.setVisible(true); pressed.setVisible(false);
        optLabel.setY(labelHoverY);
      });
      btnContainer.on("pointerout",   () => {
        if (idx === this.selectedIdx) return;
        normal.setVisible(true); hover.setVisible(false); pressed.setVisible(false);
        optLabel.setY(labelBaseY);
      });
      btnContainer.on("pointerdown",  () => {
        if (idx === this.selectedIdx) return;
        normal.setVisible(false); hover.setVisible(false); pressed.setVisible(true);
        optLabel.setY(0);
      });
      btnContainer.on("pointerup",    () => {
        if (idx !== this.selectedIdx) {
          this.select(idx);
        } else {
          this.onSelectedClick?.(idx);
        }
      });

      this.buttons.push({ container: btnContainer, normal, hover, pressed, label: optLabel, labelBaseY, labelHoverY });

      if (vertical) {
        const offsetY = -this.height / 2 + i * (btnH + btnGap) + btnH / 2;
        btnContainer.setPosition(0, offsetY);
      } else {
        const offsetX = (i - (options.length - 1) / 2) * btnW;
        btnContainer.setPosition(offsetX, 0);
      }
      this.container.add(btnContainer);
    }
  }

  private select(idx: number): void {
    const prev = this.selectedIdx;
    this.selectedIdx = idx;

    const prevBtn = this.buttons[prev];
    prevBtn.normal.setVisible(true);
    prevBtn.hover.setVisible(false);
    prevBtn.pressed.setVisible(false);
    prevBtn.normal.setTint(0x666666);
    prevBtn.hover.setTint(0x888888);
    prevBtn.label.setY(prevBtn.labelBaseY);

    const newBtn = this.buttons[idx];
    newBtn.normal.setVisible(false);
    newBtn.hover.setVisible(false);
    newBtn.pressed.setVisible(true);
    newBtn.pressed.setTint(0x44aa66);
    newBtn.label.setY(0);
  }

  getSelected(): number {
    return this.selectedIdx;
  }

  setButtonLabel(idx: number, text: string): void {
    this.buttons[idx].label.setText(text);
  }

  setPosition(x: number, y: number): void {
    this.container.setPosition(x, y);
  }
}

export class SettingsPanel {
  private scene: Phaser.Scene;
  private container: Phaser.GameObjects.Container;
  private variantToggle!: ToggleGroup;
  private blackToggle!: ToggleGroup;
  private whiteToggle!: ToggleGroup;
  private confirmBtn!: TextButton;
  private backBtn!: TextButton;
  private p1Name: string = "Human";
  private p2Name: string = "Human";
  private editingPlayer: 0 | 1 | null = null;
  private inputBuffer: string = "";
  private cursorOn: boolean = true;
  private cursorTimer: Phaser.Time.TimerEvent | null = null;
  private keydownHandler: ((e: KeyboardEvent) => void) | null = null;
  private pointerupHandler: (() => void) | null = null;
  readonly height: number;

  constructor(
    scene: Phaser.Scene,
    x: number,
    y: number,
    scale: number,
    width: number,
    initialVariant: "freestyle" | "renju",
    initialP1IsHuman: boolean,
    initialP2IsHuman: boolean,
    initialP1Name: string,
    initialP2Name: string,
    onConfirm: (variant: "freestyle" | "renju", p1IsHuman: boolean, p2IsHuman: boolean, p1Name: string, p2Name: string) => void,
    onBack: () => void,
  ) {
    this.scene = scene;
    this.p1Name = initialP1Name;
    this.p2Name = initialP2Name;

    const innerGap   = Math.round(1 * scale);  // label → its toggles
    const sectionGap = Math.round(12 * scale); // between sections
    const fontPx     = FONT_PX;

    const rulesLabel = scene.add.bitmapText(0, 0, "pixel", "RULES", fontPx).setTint(0xcccccc).setOrigin(0, 0);
    const rulesH = rulesLabel.getBounds().height;

    this.variantToggle = new ToggleGroup(scene, 0, 0, ["FREESTYLE", "RENJU"], initialVariant === "renju" ? 1 : 0, scale, width, true);

    const blackLabel = scene.add.bitmapText(0, 0, "pixel", "PLAYER 1", fontPx).setTint(0xcccccc).setOrigin(0, 0);
    const blackH = blackLabel.getBounds().height;

    this.blackToggle = new ToggleGroup(scene, 0, 0, [this.p1Name, "BOT"], initialP1IsHuman ? 0 : 1, scale, width, true,
      (idx) => { if (idx === 0 && this.editingPlayer === null) this.startEditing(0); },
    );

    const whiteLabel = scene.add.bitmapText(0, 0, "pixel", "PLAYER 2", fontPx).setTint(0xcccccc).setOrigin(0, 0);
    const whiteH = whiteLabel.getBounds().height;

    this.whiteToggle = new ToggleGroup(scene, 0, 0, [this.p2Name, "BOT"], initialP2IsHuman ? 0 : 1, scale, width, true,
      (idx) => { if (idx === 0 && this.editingPlayer === null) this.startEditing(1); },
    );

    this.confirmBtn = new TextButton(scene, 0, 0, "NEW GAME", GREEN_TINTS, () => {
      this.stopEditing(true);
      const variant    = this.variantToggle.getSelected() === 1 ? "renju" : "freestyle";
      const p1IsHuman  = this.blackToggle.getSelected() === 0;
      const p2IsHuman  = this.whiteToggle.getSelected() === 0;
      onConfirm(variant, p1IsHuman, p2IsHuman, this.p1Name, this.p2Name);
    }, scale, width);

    this.backBtn = new TextButton(scene, 0, 0, "BACK", RED_TINTS, onBack, scale, width);

    // Content height: tight within sections, larger gaps between sections
    this.height =
      (rulesH + innerGap + this.variantToggle.height) + sectionGap
      + (blackH + innerGap + this.blackToggle.height) + sectionGap
      + (whiteH + innerGap + this.whiteToggle.height) + sectionGap
      + (this.confirmBtn.height + innerGap + this.backBtn.height);

    this.container = scene.add.container(x, y);
    this.container.setDepth(20);

    let currentY = -this.height / 2;

    rulesLabel.setPosition(-width / 2, currentY);
    this.container.add(rulesLabel);
    currentY += rulesH + innerGap;

    this.variantToggle.setPosition(0, currentY + this.variantToggle.height / 2);
    this.container.add(this.variantToggle.container);
    currentY += this.variantToggle.height + sectionGap;

    blackLabel.setPosition(-width / 2, currentY);
    this.container.add(blackLabel);
    currentY += blackH + innerGap;

    this.blackToggle.setPosition(0, currentY + this.blackToggle.height / 2);
    this.container.add(this.blackToggle.container);
    currentY += this.blackToggle.height + sectionGap;

    whiteLabel.setPosition(-width / 2, currentY);
    this.container.add(whiteLabel);
    currentY += whiteH + innerGap;

    this.whiteToggle.setPosition(0, currentY + this.whiteToggle.height / 2);
    this.container.add(this.whiteToggle.container);
    currentY += this.whiteToggle.height + sectionGap;

    this.confirmBtn.setPosition(0, currentY + this.confirmBtn.height / 2);
    this.container.add(this.confirmBtn.container);
    currentY += this.confirmBtn.height + innerGap;

    this.backBtn.setPosition(0, currentY + this.backBtn.height / 2);
    this.container.add(this.backBtn.container);
  }

  setPosition(x: number, y: number): void {
    this.container.setPosition(x, y);
  }

  setVisible(v: boolean): void {
    if (!v) this.stopEditing(false);
    this.container.setVisible(v);
  }

  private startEditing(playerIdx: 0 | 1): void {
    this.editingPlayer = playerIdx;
    this.inputBuffer   = playerIdx === 0 ? this.p1Name : this.p2Name;
    this.cursorOn      = true;
    this.updateEditLabel();

    this.cursorTimer = this.scene.time.addEvent({
      delay: 530,
      callback: () => { this.cursorOn = !this.cursorOn; this.updateEditLabel(); },
      loop: true,
    });

    this.keydownHandler = (e: KeyboardEvent) => this.onKeydown(e);
    this.scene.input.keyboard!.on("keydown", this.keydownHandler);

    // Defer the click-away listener by one frame to avoid catching the triggering click.
    this.scene.time.delayedCall(1, () => {
      if (this.editingPlayer === null) return;
      this.pointerupHandler = () => this.stopEditing(true);
      this.scene.input.on("pointerup", this.pointerupHandler);
    });
  }

  private stopEditing(confirm: boolean): void {
    if (this.editingPlayer === null) return;
    const playerIdx    = this.editingPlayer;
    this.editingPlayer = null;

    if (confirm) {
      const name = this.inputBuffer.trim();
      if (name.length > 0) {
        if (playerIdx === 0) this.p1Name = name;
        else                 this.p2Name = name;
      }
    }
    // Restore clean label (no cursor).
    const toggle = playerIdx === 0 ? this.blackToggle : this.whiteToggle;
    toggle.setButtonLabel(0, playerIdx === 0 ? this.p1Name : this.p2Name);

    this.inputBuffer = "";
    this.cursorTimer?.destroy();      this.cursorTimer = null;
    if (this.keydownHandler)  { this.scene.input.keyboard!.off("keydown", this.keydownHandler); this.keydownHandler = null; }
    if (this.pointerupHandler){ this.scene.input.off("pointerup", this.pointerupHandler);        this.pointerupHandler = null; }
  }

  private updateEditLabel(): void {
    if (this.editingPlayer === null) return;
    const toggle = this.editingPlayer === 0 ? this.blackToggle : this.whiteToggle;
    toggle.setButtonLabel(0, this.inputBuffer + (this.cursorOn ? "_" : " "));
  }

  private onKeydown(e: KeyboardEvent): void {
    if (this.editingPlayer === null) return;
    if (e.key === "Enter") {
      e.preventDefault();
      this.stopEditing(true);
    } else if (e.key === "Escape") {
      this.stopEditing(false);
    } else if (e.key === "Backspace") {
      e.preventDefault();
      this.inputBuffer = this.inputBuffer.slice(0, -1);
      this.updateEditLabel();
    } else if (e.key.length === 1 && this.inputBuffer.length < 12) {
      this.inputBuffer += e.key;
      this.updateEditLabel();
    }
  }
}

export class InfoBar {
  private container: Phaser.GameObjects.Container;
  private variantText: Phaser.GameObjects.BitmapText;
  private timerText: Phaser.GameObjects.BitmapText;
  readonly height: number;

  constructor(
    scene: Phaser.Scene,
    x: number, y: number,
    scale: number,
    width: number,
    variant: "freestyle" | "renju",
  ) {
    const fontPx = FONT_PX;
    const gap = Math.round(GAP_SRC * scale);

    const titleText = scene.add.bitmapText(0, 0, "pixel", "GOMOKU2D", fontPx)
      .setTint(0xffffff)
      .setOrigin(0.5);

    this.variantText = scene.add.bitmapText(0, 0, "pixel", variant === "renju" ? "RENJU" : "FREESTYLE", fontPx)
      .setTint(0xcccccc)
      .setOrigin(0.5);

    this.timerText = scene.add.bitmapText(0, 0, "pixel", "00:00", fontPx)
      .setTint(0xcccccc)
      .setOrigin(0.5);

    const tiH = titleText.getBounds().height;
    const vB  = this.variantText.getBounds();
    const tB  = this.timerText.getBounds();
    this.height = tiH + gap + vB.height + gap + tB.height;

    // Stack top-to-bottom, centered on container origin
    titleText.setPosition(0,       -this.height / 2 + tiH / 2);
    this.variantText.setPosition(0, -this.height / 2 + tiH + gap + vB.height / 2);
    this.timerText.setPosition(0,   -this.height / 2 + tiH + gap + vB.height + gap + tB.height / 2);

    this.container = scene.add.container(x, y, [titleText, this.variantText, this.timerText]);
    this.container.setDepth(20);
  }

  setTimer(formatted: string): void {
    this.timerText.setText(formatted);
  }

  setVariant(variant: "freestyle" | "renju"): void {
    this.variantText.setText(variant === "renju" ? "RENJU" : "FREESTYLE");
  }

  setPosition(x: number, y: number): void {
    this.container.setPosition(x, y);
  }

  setVisible(v: boolean): void {
    this.container.setVisible(v);
  }
}
