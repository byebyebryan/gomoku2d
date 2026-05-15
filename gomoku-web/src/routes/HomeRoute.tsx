import { Link } from "react-router-dom";
import { APP_VERSION } from "../app/version";
import { Icon } from "../ui/Icon";
import styles from "./HomeRoute.module.css";

const baseUrl = import.meta.env.BASE_URL;

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
          <Link className="uiAction uiActionSecondary" to="/settings">
            <Icon className="uiIconDesktop" name="settings" />
            <span className="uiActionLabel">Settings</span>
          </Link>
        </div>
        <div className={styles.footer}>
          <p className={styles.version}>{APP_VERSION}</p>
          <nav className={styles.footerLinks} aria-label="Footer links">
            <a href={`${baseUrl}assets/`}>Assets</a>
            <span aria-hidden="true">/</span>
            <a href={`${baseUrl}bot-report/`}>Bots</a>
            <span aria-hidden="true">/</span>
            <a href={`${baseUrl}analysis-report/`}>Analysis</a>
            <span aria-hidden="true">/</span>
            <a href={`${baseUrl}privacy/`}>Privacy</a>
            <span aria-hidden="true">/</span>
            <a href={`${baseUrl}terms/`}>Terms</a>
          </nav>
        </div>
      </section>
    </main>
  );
}
