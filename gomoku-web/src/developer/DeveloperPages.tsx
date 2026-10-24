import { Link } from "react-router-dom";
import { useDocumentTitle } from "../app/useDocumentTitle";
import { Icon } from "../ui/Icon";

export function DeveloperOverview() {
  useDocumentTitle("Developer");
  return (
    <>
      <div className="dev-heading">
        <p className="dev-eyebrow">The work behind the game</p>
        <h1>Inside Gomoku2D</h1>
        <p className="dev-lead">Design, experiments and the checks that shape the game.</p>
      </div>
      <div className="dev-feature-list">
        {(
          [
            [
              "/visuals/",
              "settings",
              "Design & assets",
              "The visual language, actual controls and the pixels behind them.",
            ],
            [
              "/lab/",
              "bot",
              "Inside the bot lab",
              "Inspect recorded experiments, search costs and where games turned.",
            ],
            [
              "/dev/reviews/",
              "replay",
              "Review the work",
              "Compare what changed, the findings and the evidence behind a fix.",
            ],
            [
              "/dev/process/",
              "help",
              "How development works",
              "Research, implement, inspect, validate—and improve the next iteration.",
            ],
          ] as const
        ).map(([to, icon, title, copy]) => (
          <Link className="dev-feature" to={to} key={to}>
            <Icon name={icon} />
            <span>
              <h2>{title}</h2>
              <p>{copy}</p>
            </span>
            <Icon name="next" />
          </Link>
        ))}
      </div>
      <div className="dev-columns dev-section">
        <section>
          <h2>AI-built. Human-directed.</h2>
          <p>
            Agents research, implement, inspect and test. Human judgment sets scope, domain meaning,
            visual taste and acceptance.
          </p>
          <p>
            The useful record is the decision, the result and the evidence that changed the next
            step.
          </p>
          <Link to="/dev/process/">Follow the development process</Link>
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
export function DeveloperProcess() {
  useDocumentTitle("Development process");
  return (
    <>
      <div className="dev-heading">
        <p className="dev-eyebrow">Development process</p>
        <h1>Build. Inspect. Improve.</h1>
        <p className="dev-lead">
          How the work moves forward—and how the workflow itself gets better.
        </p>
      </div>
      <p className="dev-readable">
        Human direction owns scope, domain meaning, taste and acceptance. Agents research,
        implement, inspect and validate. The next step follows the evidence, including failures and
        corrections.
      </p>
      <dl className="dev-process">
        {[
          [
            "Research",
            "Settle the question",
            "Trace current behavior, domain rules and constraints before choosing a change.",
          ],
          [
            "Design",
            "Make the choice reviewable",
            "Use real content, concrete alternatives and source-backed visual references.",
          ],
          [
            "Implement",
            "Bound the change",
            "Give work clear behavior, ownership and invariants. Keep unrelated work safe.",
          ],
          [
            "Review & fix",
            "Find what is still wrong",
            "Inspect source, browser behavior and captures. Preserve findings and follow-ups.",
          ],
          [
            "Validate",
            "Say what the evidence establishes",
            "Choose checks for the changed contract. Keep automation, human acceptance and deployment distinct.",
          ],
          [
            "Release",
            "Verify what was published",
            "Run the exact-source gate, then compare deployed bytes and exercise the live build.",
          ],
        ].map(([label, title, copy]) => (
          <div key={label}>
            <dt>{label}</dt>
            <dd>
              <h2>{title}</h2>
              <p>{copy}</p>
            </dd>
          </div>
        ))}
      </dl>
      <section className="dev-section">
        <h2>One product, clear responsibilities</h2>
        <div className="dev-architecture">
          <span>React · shell & state</span>
          <span>Phaser · board</span>
          <span>Rust / Wasm · rules & analysis</span>
        </div>
        <p className="dev-readable">
          The native lab runs experiments around the same core. Browser state keeps local games and
          settings. Optional Firebase continuity handles private accounts and explicitly published
          profiles/replays. Developer references and recorded reports need neither an account nor
          private history.
        </p>
      </section>
      <section className="dev-section dev-readable">
        <h2>Evidence has different jobs</h2>
        <h3>Automated checks</h3>
        <p>
          Unit and rules tests check domain/state contracts. Browser journeys check integrated
          behavior; emulator journeys check cloud continuity and security rules with synthetic
          accounts. Renderer checks inspect pixels; profiling records a stated workload and host.
          Each result belongs to its source, configuration and scope.
        </p>
        <h3>Visual review</h3>
        <p>
          Layout, density, icon consistency and communication need inspection.{" "}
          <Link to="/dev/reviews/">Review records</Link> keep the comparison, findings, decisions
          and follow-up together.
        </p>
        <h3>Human and device use</h3>
        <p>
          Physical touch, browser keyboards, assistive technology and real account round trips need
          their own evidence. A passing emulator run does not close those checks.
        </p>
      </section>
      <section className="dev-section dev-readable">
        <h2>A worked release</h2>
        <p>
          Version 0.6.2 passed its exact-source CI and publication gates. Its deployed browser
          checks recorded 108 passes and six intentional renderer skips. Real signed-in sharing
          acceptance remains pending in that dated record.
        </p>
        <a href="https://github.com/byebyebryan/gomoku2d/blob/4052d94/docs/archive/v0_6_2_release.md">
          Read the 0.6.2 release record
        </a>
      </section>
      <section className="dev-section dev-readable">
        <h2>Improving the workflow</h2>
        <p>
          Earlier reviews used individual HTML exports. The 0.6.3 workflow brings curated records
          and the same comparison viewer into both the site and standalone review exports. Current
          screen references stay separate from historical fixes; missing historical conditions
          remain unknown.
        </p>
        <p>
          Detailed maintainer and agent context stays in{" "}
          <a href="https://github.com/byebyebryan/gomoku2d/tree/main/docs">
            Markdown documentation
          </a>
          . This area prepares selected evidence for reading and inspection. Longer retrospective
          stories can build on that record later.
        </p>
      </section>
    </>
  );
}
