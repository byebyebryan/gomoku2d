import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

// Keep native FFmpeg separate from the browser container's capture stage.
const root = resolve(import.meta.dirname, "../..");
const temporary = resolve(process.env.GOMOKU_MEDIA_TMP_DIR ?? resolve(root, "gomoku-web/test-results/v062-completion/media"));
const path = resolve(temporary, "capture.json");
const capture = JSON.parse(await readFile(path, "utf8"));
const run = promisify(execFile);
for (const [name, frames] of [["readme-sharing", capture.sharingFrames], ["readme-analysis", capture.analysisFrames]]) {
  if (!Number.isInteger(frames) || frames < 1 || frames > 500) throw new Error("Invalid captured frame count.");
  const input = resolve(temporary, name, "%04d.png");
  const palette = resolve(temporary, name, "palette.png");
  await run("ffmpeg", ["-y", "-framerate", "5", "-i", input, "-vf", "scale=960:-1:flags=lanczos,palettegen=max_colors=128", palette]);
  await run("ffmpeg", ["-y", "-framerate", "5", "-i", input, "-i", palette, "-lavfi", "scale=960:-1:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer:bayer_scale=5", "-frames:v", String(frames), "-loop", "0", resolve(root, "docs/assets", `${name}.gif`)]);
}
capture.encoded = true;
await writeFile(path, JSON.stringify(capture, null, 2) + "\n");
console.log("Encoded sharing and analysis clips using native FFmpeg.");
