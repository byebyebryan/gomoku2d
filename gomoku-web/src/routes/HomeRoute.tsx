import { Link } from "react-router-dom";
import { PROJECT_SOURCE_URL } from "../app/links";
import { useDocumentTitle } from "../app/useDocumentTitle";
import { APP_VERSION } from "../app/version";
import { Icon } from "../ui/Icon";
import styles from "./HomeRoute.module.css";

export function HomeRoute() {
  useDocumentTitle();

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
            <span className={styles.footerLinkGroup}>
              <Link to="/rules/">Rules</Link>
              <span aria-hidden="true">/</span>
              <Link to="/guide/">Guide</Link>
              <span aria-hidden="true">/</span>
              <Link to="/lab/">Lab</Link>
              <span aria-hidden="true">/</span>
              <Link to="/visuals/">Visuals</Link>
            </span>
            <span className={styles.footerLinkGroup}>
              <a href={PROJECT_SOURCE_URL} rel="noreferrer" target="_blank">
                Source
              </a>
              <span aria-hidden="true">/</span>
              <Link to="/privacy/">Privacy</Link>
              <span aria-hidden="true">/</span>
              <Link to="/terms/">Terms</Link>
            </span>
          </nav>
        </div>
      </section>
    </main>
  );
}
