import { memo, useEffect, useRef, useState } from "react";
import { drawReplayThumbnail, loadThumbnailStones, type ReplayThumbnailPosition } from "../replay/replay_thumbnail";
import styles from "./ReplayThumbnail.module.css";

export const ReplayThumbnail = memo(function ReplayThumbnail({ position }: { position: ReplayThumbnailPosition | null }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [ready, setReady] = useState<ReplayThumbnailPosition | null>(null);
  const [failed, setFailed] = useState<ReplayThumbnailPosition | null>(null);
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !position) return;
    let disposed = false;
    let observer: ResizeObserver | undefined;
    const signals = new AbortController();
    void loadThumbnailStones().then((stones) => {
      if (disposed) return;
      let lastWidth = 0;
      let lastRatio = 0;
      const draw = () => {
        const width = canvas.clientWidth;
        const ratio = window.devicePixelRatio;
        if (disposed || !width || (width === lastWidth && ratio === lastRatio)) return;
        try {
          drawReplayThumbnail(canvas, position, width, ratio, stones);
          lastWidth = width;
          lastRatio = ratio;
          setReady(position);
        } catch {
          setFailed(position);
        }
      };
      draw();
      if (typeof ResizeObserver !== "undefined") {
        observer = new ResizeObserver(draw);
        observer.observe(canvas);
      }
      window.addEventListener("resize", draw, { signal: signals.signal });
    }).catch(() => { if (!disposed) setFailed(position); });
    return () => {
      disposed = true;
      observer?.disconnect();
      signals.abort();
    };
  }, [position]);

  const unavailable = !position || failed === position;
  return <div className={styles.thumbnail} aria-hidden="true" data-testid="replay-thumbnail"
    data-opening={position?.opening}>
    <canvas ref={canvasRef} className={styles.canvas} hidden={unavailable} data-ready={ready === position && !unavailable} />
    {unavailable && <span className={styles.fallback}>Board unavailable</span>}
  </div>;
});
