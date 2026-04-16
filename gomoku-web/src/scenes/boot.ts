import Phaser from "phaser";
import { SPRITESHEET_CONFIG, FRAME_SIZE, SPRITE, STONE_ANIMS, WARNING_ANIMS } from "../board/constants";

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

    // Load button images (individual 18x18, not spritesheets)
    this.load.image("button_0", "assets/sprites/button_0.png");
    this.load.image("button_1", "assets/sprites/button_1.png");
    this.load.image("button_2", "assets/sprites/button_2.png");
    this.load.image("button_3", "assets/sprites/button_3.png");

    // Load bitmap font (pixel-perfect, no AA)
    this.load.bitmapFont("pixel", "assets/sprites/PixelOperator8-Bold.png", "assets/sprites/PixelOperator8-Bold.fnt");
  }

  create(): void {
    // Register all animations
    this.createAnimations();

    // All assets loaded — transition to game scene
    this.scene.start("GameScene");
  }

  private createAnimations(): void {
    // Stone animations
    this.anims.create({
      key: STONE_ANIMS.FORM.key,
      frames: this.anims.generateFrameNumbers(SPRITE.STONE, {
        start: STONE_ANIMS.FORM.start,
        end: STONE_ANIMS.FORM.end,
      }),
      frameRate: STONE_ANIMS.FORM.frameRate,
    });

    this.anims.create({
      key: STONE_ANIMS.DESTROY.key,
      frames: this.anims.generateFrameNumbers(SPRITE.STONE, {
        start: STONE_ANIMS.DESTROY.start,
        end: STONE_ANIMS.DESTROY.end,
      }),
      frameRate: STONE_ANIMS.DESTROY.frameRate,
    });

    for (const relax of [STONE_ANIMS.RELAX_1, STONE_ANIMS.RELAX_2, STONE_ANIMS.RELAX_3, STONE_ANIMS.RELAX_4]) {
      this.anims.create({
        key: relax.key,
        frames: this.anims.generateFrameNumbers(SPRITE.STONE, {
          start: relax.start,
          end: relax.end,
        }),
        frameRate: relax.frameRate,
      });
    }

    // Warning animation (loops)
    this.anims.create({
      key: WARNING_ANIMS.SURFACE.key,
      frames: this.anims.generateFrameNumbers(SPRITE.WARNING_L, {
        start: WARNING_ANIMS.SURFACE.start,
        end: WARNING_ANIMS.SURFACE.end,
      }),
      frameRate: WARNING_ANIMS.SURFACE.frameRate,
      repeat: -1,
    });
  }
}
