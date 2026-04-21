import { Link } from "react-router-dom";

import styles from "./HomeRoute.module.css";

export function HomeRoute() {
  return (
    <main className={styles.page}>
      <section className={styles.hero}>
        <p className={styles.eyebrow}>Five in a row</p>
        <h1 className={styles.title}>Gomoku2D</h1>
        <p className={styles.summary}>
          A quiet board, a stubborn Classic Bot, and one simple goal: make
          five before it does.
        </p>
        <div className={styles.actions}>
          <Link className={styles.primaryAction} to="/match/local">
            Play Bot
          </Link>
          <Link className={styles.secondaryAction} to="/profile">
            Profile
          </Link>
        </div>
      </section>
    </main>
  );
}
