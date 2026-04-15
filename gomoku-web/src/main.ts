import Phaser from "phaser";
import { BootScene } from "./scenes/boot";

const config: Phaser.Types.Core.GameConfig = {
  type: Phaser.AUTO,
  parent: "game",
  width: 800,
  height: 800,
  backgroundColor: "#1a1a2e",
  pixelArt: true,
  scale: {
    mode: Phaser.Scale.FIT,
    autoCenter: Phaser.Scale.CENTER_BOTH,
  },
  scene: [BootScene],
};

const game = new Phaser.Game(config);
