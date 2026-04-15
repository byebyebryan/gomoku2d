import Phaser from "phaser";
import { SPRITESHEET_CONFIG, FRAME_SIZE, SPRITE } from "../board/constants";

export class BootScene extends Phaser.Scene {
  constructor() {
    super({ key: "BootScene" });
  }

  preload(): void {
    // Load all spritesheets (16x16 per frame, horizontal strips)
    for (const [key, cfg] of Object.entries(SPRITESHEET_CONFIG)) {
      this.load.spritesheet(key, cfg.url, {
        frameWidth: FRAME_SIZE,
        frameHeight: FRAME_SIZE,
      });
    }

    // Load font
    this.load.font("minecraft", "assets/sprites/minecraft.ttf");
  }

  create(): void {
    // Verify sprites loaded — place a static black stone and white stone
    const cx = this.cameras.main.centerX;
    const cy = this.cameras.main.centerY;

    // Black stone (frame 0 = static)
    this.add.sprite(cx - 32, cy, SPRITE.STONE, 0).setScale(3);

    // White stone (frame 0)
    this.add.sprite(cx + 32, cy, SPRITE.STONE, 0).setScale(3).setTint(0xffffff);

    // Pointer preview
    this.add.sprite(cx, cy - 64, SPRITE.POINTER, 0).setScale(3);

    // Title text
    this.add
      .text(cx, cy + 80, "Gomoku 2D — Assets Loaded", {
        fontFamily: "minecraft",
        fontSize: "20px",
        color: "#7fffaa",
      })
      .setOrigin(0.5);
  }
}
