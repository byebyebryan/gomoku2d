import { Link } from "react-router-dom";
import styles from "./HomeRoute.module.css";

export function HomeRoute() {
  return (
    <main className={styles.page}>
      <section className={styles.hero}>
        <p className="uiPageEyebrow">Five in a row</p>
        <h1 className={styles.title}>Gomoku2D</h1>
        <p className={styles.summary}>
          Quiet board. Classic Bot. First to five takes it.
        </p>
        <div className={styles.actions}>
          <Link className="uiAction uiActionPrimary" to="/match/local">
            Play
          </Link>
          <Link className="uiAction uiActionSecondary" to="/profile">
            Profile
          </Link>
        </div>
      </section>
    </main>
  );
}
