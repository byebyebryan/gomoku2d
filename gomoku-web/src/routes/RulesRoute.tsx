import { useEffect } from "react";
import { Link } from "react-router-dom";

import styles from "./RulesRoute.module.css";

export function RulesRoute() {
  useEffect(() => {
    document.title = "Gomoku2D Rules";
  }, []);

  return (
    <main className={styles.page}>
      <div className={styles.shell}>
        <header className={styles.hero}>
          <div className={styles.headerRow}>
            <div>
              <p className="uiPageEyebrow">How to play</p>
              <h1 className={styles.title}>Rules</h1>
              <p className={styles.summary}>
                Gomoku is a five-in-a-row game. Players alternate placing stones; the
                first player to make five connected stones horizontally, vertically, or
                diagonally wins.
              </p>
            </div>
            <nav className={styles.links} aria-label="Rules links">
              <Link className="uiAction uiActionNeutral" to="/">
                <span className="uiActionLabel">Home</span>
              </Link>
              <Link className="uiAction uiActionPrimary" to="/match/local">
                <span className="uiActionLabel">Play</span>
              </Link>
            </nav>
          </div>
        </header>

        <div className={styles.content}>
          <section className={styles.panel}>
            <h2>Freestyle</h2>
            <ul>
              <li>Black moves first.</li>
              <li>Either side wins by making five or more in a row.</li>
              <li>There are no forbidden moves.</li>
            </ul>
          </section>

          <section className={styles.panel}>
            <h2>Renju</h2>
            <p>
              Renju keeps the game more balanced by restricting Black. White can win
              normally, but Black has forbidden moves.
            </p>
            <ul>
              <li>Overline: Black makes more than five in a row.</li>
              <li>Double-four: Black creates multiple real four threats.</li>
              <li>Double-three: Black creates multiple real three threats.</li>
            </ul>
          </section>

          <section className={styles.panel}>
            <h2>Real threats</h2>
            <p>
              The important detail is real. Gomoku2D checks whether a forbidden-looking
              Renju shape is actually live under the rules, then uses that same legality
              model for play, hints, bots, and replay analysis.
            </p>
          </section>
        </div>

        <p className={styles.callout}>Renju forbidden moves apply only to Black.</p>
      </div>
    </main>
  );
}
