import Phaser from "phaser";
import { BootScene } from "./scenes/boot";
import { GameScene } from "./scenes/game";
import { initWasm } from "./core/wasm_bridge";

const config: Phaser.Types.Core.GameConfig = {
  type: Phaser.AUTO,
  parent: "game",
  width: 1024,
  height: 768,
  backgroundColor: "#3a3a3a",
  pixelArt: true,
  scale: {
    mode: Phaser.Scale.FIT,
    autoCenter: Phaser.Scale.CENTER_BOTH,
  },
  scene: [BootScene, GameScene],
};

initWasm().then(() => {
  new Phaser.Game(config);
});
