import { Link } from "react-router-dom";
import { useDocumentTitle } from "../app/useDocumentTitle";
import { Icon } from "../ui/Icon";
import { DEVELOPER_SECTIONS, DEV_SITE_LABEL } from "../app/links";

export function DeveloperOverview() {
  useDocumentTitle(DEV_SITE_LABEL);
  return (
    <>
      <div className="dev-heading">
        <p className="dev-eyebrow">The work behind the game</p>
        <h1>Inside Gomoku2D</h1>
        <p className="dev-lead">Design, experiments and the checks that shape the game.</p>
      </div>
      <div className="dev-feature-list">
        {DEVELOPER_SECTIONS.map(({ to, icon, label, description }) => (
          <Link className="dev-feature" to={to} key={to}>
            <Icon name={icon} />
            <span>
              <h2>{label}</h2>
              <p>{description}</p>
            </span>
            <Icon name="next" />
          </Link>
        ))}
      </div>
      <div className="dev-columns dev-section">
        <section>
          <h2>The released screens</h2>
          <p>
            Home, match, replay, settings, profile and public player, captured from the 0.6.2
            release. Compare desktop and portrait layouts, then inspect landscape and enlarged text.
          </p>
          <Link to="/dev/reviews/core-screens-062/">Open the screen reference</Link>
        </section>
        <section>
          <h2>Start with a concrete example</h2>
          <ul className="dev-record-list">
            <li>
              <Link to="/visuals/?view=icons&compare=share,save,publish">
                Compare the sharing icons
              </Link>
              <small>Canonical SVGs · native size and pixel grid</small>
            </li>
            <li>
              <Link to="/dev/reviews/mobile-records-062/">Mobile records, before and after</Link>
              <small>Dated checkpoint · follow-up identified</small>
            </li>
            <li>
              <Link to="/lab/?tab=analysis&match=match_0095__search-d1__vs__search-d3_pattern-eval">
                A possible last escape
              </Link>
              <small>Recorded analysis · bounded model</small>
            </li>
          </ul>
        </section>
      </div>
    </>
  );
}
