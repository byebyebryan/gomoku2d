import * as Phaser from "phaser";

export interface BoardInputHandlers {
  onPointerDown: (pointer: Phaser.Input.Pointer) => void;
  onPointerMove: (pointer: Phaser.Input.Pointer) => void;
  onPointerOut: () => void;
  onPointerUp: (pointer: Phaser.Input.Pointer) => void;
}

export class BoardInputController {
  constructor(
    private readonly input: Phaser.Input.InputPlugin,
    private readonly handlers: BoardInputHandlers,
  ) {}

  attach(): void {
    this.input.on("pointermove", this.handlers.onPointerMove);
    this.input.on("pointerdown", this.handlers.onPointerDown);
    this.input.on("pointerup", this.handlers.onPointerUp);
    this.input.on("pointerout", this.handlers.onPointerOut);
  }

  detach(): void {
    this.input.off("pointermove", this.handlers.onPointerMove);
    this.input.off("pointerdown", this.handlers.onPointerDown);
    this.input.off("pointerup", this.handlers.onPointerUp);
    this.input.off("pointerout", this.handlers.onPointerOut);
  }
}

export function pointerType(pointer: Phaser.Input.Pointer): string {
  return (pointer as { pointerType?: string }).pointerType ?? "mouse";
}
