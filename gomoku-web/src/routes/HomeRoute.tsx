import { Link } from "react-router-dom";

import styles from "./HomeRoute.module.css";

export function HomeRoute() {
  return (
    <main className={styles.page}>
      <section className={styles.hero}>
        <p className={styles.eyebrow}>Phase 1 / React shell</p>
        <h1 className={styles.title}>Gomoku2D</h1>
        <p className={styles.summary}>
          The board stays in Phaser. Everything around it moves to a DOM shell.
          This first cut keeps the product narrow: one offline bot match,
          routed through the new architecture.
        </p>
        <div className={styles.actions}>
          <Link className={styles.primaryAction} to="/match/local">
            Play Bot
          </Link>
        </div>
      </section>
    </main>
  );
}
