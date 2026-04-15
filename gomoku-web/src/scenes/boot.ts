import Phaser from "phaser";
import { SPRITESHEET_CONFIG, FRAME_SIZE } from "../board/constants";

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
    // All assets loaded — transition to game scene
    this.scene.start("GameScene");
  }
}
