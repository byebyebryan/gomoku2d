import Phaser from "phaser";
import { BootScene } from "./scenes/boot";
import { GameScene } from "./scenes/game";
import { initWasm } from "./core/wasm_bridge";
import { getGameSizeForViewport, getViewportSize } from "./layout";

const initialViewport = getViewportSize();
const { width: gameWidth, height: gameHeight } = getGameSizeForViewport(
  initialViewport.width,
  initialViewport.height,
);

const config: Phaser.Types.Core.GameConfig = {
  type: Phaser.AUTO,
  parent: "game",
  width: gameWidth,
  height: gameHeight,
  backgroundColor: "#3a3a3a",
  pixelArt: true,
  scale: {
    mode: Phaser.Scale.FIT,
    autoCenter: Phaser.Scale.Center.CENTER_BOTH,
  },
  scene: [BootScene, GameScene],
};

initWasm().then(() => {
  new Phaser.Game(config);
});
