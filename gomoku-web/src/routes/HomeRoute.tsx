import { Link } from "react-router-dom";
import { Icon } from "../ui/Icon";
import styles from "./HomeRoute.module.css";

export function HomeRoute() {
  return (
    <main className={styles.page}>
      <section className={styles.hero}>
        <p className="uiPageEyebrow">ByeByeBryan&apos;s</p>
        <h1 className={styles.title}>Gomoku2D</h1>
        <p className={styles.summary}>An old favorite, built properly.</p>
        <div className={styles.actions}>
          <Link className="uiAction uiActionPrimary" to="/match/local">
            <Icon className="uiIconDesktop" name="play" />
            <span className="uiActionLabel">Play</span>
          </Link>
          <Link className="uiAction uiActionSecondary" to="/profile">
            <Icon className="uiIconDesktop" name="profile" />
            <span className="uiActionLabel">Profile</span>
          </Link>
        </div>
      </section>
    </main>
  );
}
