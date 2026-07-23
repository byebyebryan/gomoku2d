import { Link } from "react-router-dom";

import { PROJECT_SOURCE_URL } from "../app/links";
import { useDocumentTitle } from "../app/useDocumentTitle";
import styles from "./PolicyRoute.module.css";

type PolicyPageKind = "privacy" | "terms";

interface PolicyRouteProps {
  kind: PolicyPageKind;
}

const UPDATED: Record<PolicyPageKind, string> = {
  privacy: "July 22, 2026",
  terms: "July 22, 2026",
};

export function PolicyRoute({ kind }: PolicyRouteProps) {
  const isPrivacy = kind === "privacy";
  const title = isPrivacy ? "Privacy" : "Terms";

  useDocumentTitle(isPrivacy ? "Privacy Policy" : "Terms of Service");

  return (
    <main className={styles.page}>
      <div className={styles.shell}>
        <header className={styles.hero}>
          <div className={styles.headerRow}>
            <div>
              <p className="uiPageEyebrow">Gomoku2D policy</p>
              <h1 className={styles.title}>{title}</h1>
              <p className={styles.updated}>Last updated: {UPDATED[kind]}</p>
            </div>
            <nav className={styles.links} aria-label={`${title} links`}>
              <Link className="uiAction uiActionNeutral" to="/">
                <span className="uiActionLabel">Home</span>
              </Link>
              <a
                className="uiAction uiActionNeutral"
                href={PROJECT_SOURCE_URL}
                rel="noreferrer"
                target="_blank"
              >
                <span className="uiActionLabel">Source</span>
              </a>
              <Link
                className="uiAction uiActionNeutral"
                to={isPrivacy ? "/terms/" : "/privacy/"}
              >
                <span className="uiActionLabel">{isPrivacy ? "Terms" : "Privacy"}</span>
              </Link>
            </nav>
          </div>
        </header>

        <article className={styles.panel}>
          {isPrivacy ? <PrivacyContent /> : <TermsContent />}
        </article>
      </div>
    </main>
  );
}

function PrivacyContent() {
  return (
    <>
      <p className={styles.summary}>
        Gomoku2D is a local-first browser game. Most play data stays in your
        browser. Google sign-in is optional; when you use it, Gomoku2D stores a
        cloud profile and private match history so your profile and games can sync
        across browsers.
      </p>

      <div className={styles.sections}>
        <section className={styles.section}>
          <h2>Information stored locally</h2>
          <p>
            Gomoku2D stores your local profile, game and bot settings, board-hint
            and touch preferences, match history, and cached replay analyses in
            your browser&apos;s local storage. When you sign in, the browser also
            keeps a local cache and pending sync queue for your private cloud
            history. This data stays on the device unless you reset the profile or
            clear the site&apos;s browser data.
          </p>
        </section>

        <section className={styles.section}>
          <h2>Information stored for cloud profiles and history</h2>
          <p>
            If you sign in with Google, Gomoku2D may store a private cloud profile
            in Firebase/Firestore. That profile can include your Google account
            identifier, profile name, avatar URL, provider IDs, saved game and bot
            settings, board-hint and touch preferences, and login/update
            timestamps. The app-owned profile does not copy your email address
            into Firestore. Gomoku2D may also store private match history,
            including game results, rule variants, players, moves, timestamps, and
            replay metadata. Replay-analysis results are cached locally and are
            not synced to your cloud profile.
          </p>
        </section>

        <section className={styles.section}>
          <h2>Google sign-in</h2>
          <p>
            Google handles authentication. Gomoku2D does not receive or store your
            Google password. Google may process sign-in data according to
            Google&apos;s own privacy policy and account settings.
          </p>
        </section>

        <section className={styles.section}>
          <h2>Analytics and advertising</h2>
          <p>
            Gomoku2D does not currently use third-party analytics, advertising
            trackers, or sell user data.
          </p>
        </section>

        <section className={styles.section}>
          <h2>Experimental cloud features</h2>
          <p className={styles.callout}>
            Cloud profiles and history are experimental. Cloud data may be changed,
            migrated, or reset while backend work continues.
          </p>
        </section>

        <section className={styles.section}>
          <h2>Data deletion</h2>
          <p>
            Use Reset Profile from the Profile screen to clear the current profile.
            Signed out, it clears local settings, match history, and cached replay
            analyses on this device. Signed in, it resets your cloud profile and
            history, then clears the corresponding local profile data and caches
            on this device.
          </p>
          <p>
            Signed-in users can choose Delete Cloud inside Reset Profile to delete
            the Gomoku2D cloud profile and history, then sign out. That action
            preserves the local profile, games, and analyses on this device.
          </p>
        </section>

        <section className={styles.section}>
          <h2>Contact</h2>
          <p>
            For privacy questions or deletion requests, email{" "}
            <a href="mailto:gomoku2d@byebyebryan.com">gomoku2d@byebyebryan.com</a>.
          </p>
        </section>
      </div>
    </>
  );
}

function TermsContent() {
  return (
    <>
      <p className={styles.summary}>
        Gomoku2D is a browser Gomoku/Renju game. You can play locally without an
        account, or sign in for cloud profile and private history features.
      </p>

      <div className={styles.sections}>
        <section className={styles.section}>
          <h2>Use of the game</h2>
          <p>
            Use Gomoku2D for normal play, testing, and feedback. Do not use the
            hosted app to abuse, disrupt, overload, attack, or interfere with the
            service or other users.
          </p>
        </section>

        <section className={styles.section}>
          <h2>Cloud profile and history status</h2>
          <p className={styles.callout}>
            Cloud profiles and private history are experimental. Cloud data may be
            changed, migrated, or reset while backend work continues.
          </p>
        </section>

        <section className={styles.section}>
          <h2>Your content</h2>
          <p>
            If you set a profile name, keep it lawful and appropriate. Gomoku2D may
            remove or reset cloud-backed content that is abusive, unlawful, or
            harmful to the service.
          </p>
        </section>

        <section className={styles.section}>
          <h2>Profile reset and deletion</h2>
          <p>
            You can reset your local or cloud profile from the Profile screen. If
            you are signed in, open Reset Profile and choose Delete Cloud to delete
            your Gomoku2D cloud profile and sign out. Local browser data remains
            local unless you reset it or clear site data manually.
          </p>
        </section>

        <section className={styles.section}>
          <h2>No warranty</h2>
          <p>
            Gomoku2D is provided as-is. The app may contain bugs, downtime,
            incomplete features, or data loss, especially while cloud features are
            still experimental.
          </p>
        </section>

        <section className={styles.section}>
          <h2>Open source license</h2>
          <p>
            The source code is available under the license in the repository. These
            terms apply to use of the hosted app and do not replace the repository
            license for source code usage.
          </p>
        </section>

        <section className={styles.section}>
          <h2>Changes</h2>
          <p>
            These terms may change as Gomoku2D evolves. Continued use of the hosted
            app after changes means you accept the updated terms.
          </p>
        </section>

        <section className={styles.section}>
          <h2>Contact</h2>
          <p>
            For questions about these terms, email{" "}
            <a href="mailto:gomoku2d@byebyebryan.com">gomoku2d@byebyebryan.com</a>.
          </p>
        </section>
      </div>
    </>
  );
}
