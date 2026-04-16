import Phaser from "phaser";
import { SPRITESHEET_CONFIG, FRAME_SIZE, BUTTON_FRAME_SIZE, SPRITE, STONE_ANIMS, POINTER_ANIMS, WARNING_ANIMS } from "../board/constants";

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

    // Button spritesheet: 18x18 per frame, 3 frames (0=up, 1=hover, 2=down)
    this.load.spritesheet(SPRITE.BUTTON, "assets/sprites/button.png", {
      frameWidth: BUTTON_FRAME_SIZE,
      frameHeight: BUTTON_FRAME_SIZE,
    });

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

    // Pointer animations
    for (const anim of [POINTER_ANIMS.OUT, POINTER_ANIMS.IN, POINTER_ANIMS.FULL]) {
      this.anims.create({
        key: anim.key,
        frames: this.anims.generateFrameNumbers(SPRITE.POINTER, { start: anim.start, end: anim.end }),
        frameRate: anim.frameRate,
      });
    }

    // Warning animations (both loop)
    for (const anim of [WARNING_ANIMS.POINTER, WARNING_ANIMS.HOVER]) {
      this.anims.create({
        key: anim.key,
        frames: this.anims.generateFrameNumbers(SPRITE.WARNING, {
          start: anim.start,
          end: anim.end,
        }),
        frameRate: anim.frameRate,
        repeat: -1,
      });
    }
  }
}
