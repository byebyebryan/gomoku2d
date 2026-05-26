# v0.5 Public Release Reconciliation Plan

Purpose: capture the post-`0.4` alignment pass before implementation starts.
This is an ad-hoc working plan. The canonical phase summary lives in
[`Roadmap`](../reference/product/roadmap.md).

## Context

Gomoku2D started as a revival of an old, almost decade-old project. The early
versions proved the stack and made the web game credible:

- `0.1` proved Rust + Wasm + Phaser + GitHub Pages.
- `0.2` turned the prototype into a local-first browser product.
- `0.3` added optional cloud continuity without breaking guest-first play.
- `0.4` built the lab-powered identity: configurable bots, reports, rolling
  threat facts, replay analysis, lethal onset, Renju correctness, and
  mistake-aware explanations.

The important lesson from `0.4` is not that every experiment worked. Many bot
tuning directions failed their promotion gates. The bigger achievement is that
the project now has a real lab under the board and can explain finished games
in a way a normal Gomoku clone cannot.

`0.5` should therefore be a reconciliation and public-release line. The goal is
to make the existing achievement legible, maintainable, and presentable enough
for a first public push, not to start another broad research phase.

## Product Thesis

The public hook should be:

> Play instantly, then learn where the game turned.

For players, lead with the familiar loop:

1. Play a quick Gomoku/Renju game.
2. Review the ending.
3. See the last escape, missed response, or lethal onset.
4. Branch from the replay and try again.

For developers and dev-log readers, the secondary hook is the production
experiment: one developer using agents to cover more of a real product loop
without dropping the quality bar.

## What Worked

- The Rust-core-first architecture worked. Rules, bots, eval tools, wasm, and
  replay analysis share semantics instead of duplicating game logic.
- The React/Phaser boundary worked. The browser shell can evolve while Phaser
  stays focused on board rendering and animation.
- The release spine worked. Each line produced a coherent product state rather
  than a loose task bundle.
- The lab discipline worked. Tournaments, reports, benchmarks, screenshot
  reviews, and release notes turned experiments into evidence.
- The biggest product win was the pivot from raw bot strength to explainable
  play: corridor search, lethal onset, Renju correctness, and mistake
  classification give Gomoku2D a distinctive identity.

## What Did Not Go As Planned

- Most direct bot-strength experiments were not worth promoting. Corridor
  portals, broad shape eval, partial local-threat eval, and several tactical
  shortcuts either cost too much, hurt strength, or only fixed narrow
  diagnostics.
- Corridor search as a live search extension is still not broadly useful under
  browser-scale compute budgets. Its strongest role today is replay analysis
  and report-backed diagnosis.
- Renju correctness was deeper than expected. The old shape shortcut was not
  reliable enough, and the project needed a proper recursive legality checker
  plus external reference fixtures.
- Presentation is behind capability. The app can do interesting things, but a
  stranger can still miss why the lab, reports, and replay analysis matter.
- Some repo artifacts are now heavier than they should be for a public-facing
  project. The checked-in report data and generated report HTML are highlights,
  but they should be handled intentionally.

## Revisions To The `0.5` Direction

### Keep The House In Shape

Cleanup is not optional polish. How the project is built is part of the project
claim. `0.5` should include code, test, doc, and artifact cleanup before public
release.

This does not mean churn for its own sake. It means removing dead paths,
aligning stale docs, trimming generated artifacts, and keeping the repo
reviewable after the intense `0.4` lab line.

### Productize Reports

The bot report and replay analysis report are highlights of the project, not
throwaway developer artifacts. They show the lab working.

Current state worth cleaning up:

- Historical `gomoku-bot-lab/reports/latest.json` artifacts were about `31 MB`.
- Historical generated report HTML artifacts were about `4 MB` each.
- Published reports now target compact `report.json` data rendered by the web app.

The report generation model should move toward:

- Rust eval emits structured data.
- The web app owns report presentation.
- Published report pages use viewer + data instead of checked-in monolithic
  generated HTML.
- Only curated report data is versioned, and only when it is part of the
  release story.
- The report routes feel like first-class Gomoku2D pages, not lab dumps.

This should also reduce GitHub language skew from generated HTML and make the
repo easier to review.

### Add Product Explanation Pages

The app now has features that deserve short explanations. Add static/product
pages in the same spirit as privacy/terms, but player-facing:

- `About`: old favorite, built properly; product plus production experiment.
- `Rules`: Gomoku, Freestyle, Renju, forbidden moves, and why Renju is tricky.
- `Analysis`: last escape, lethal onset, missed response, setup corridor, and
  bounded-model caveats.
- `Bot Lab`: Easy/Normal/Hard, configurable bot settings, and what reports
  measure.
- Optional `Devlog` or `Lab Notes`: a bridge for public writing if we decide to
  publish the build process.

These pages should not be walls of documentation. They should make the product
features understandable from inside the app.

### Package For First Public Release

Once the repo and product story are reconciled, prepare a public alpha:

- refreshed README and homepage copy;
- current hero GIF/video and screenshots;
- Open Graph/social images;
- itch.io page or equivalent first listing;
- short dev-log series;
- public-release smoke pass covering mobile, replay analysis, sign-in,
  settings persistence, report pages, and no-config fallback.

## Suggested Slices

### Slice 1: Repo And Doc Reconciliation (`0.5.0`)

- Sync `docs/reference/lab/search_bot.md`, `docs/working/performance_tuning.md`, and roadmap notes
  with the latest pooled-budget reports.
- Mark generated curated report artifacts explicitly so GitHub language stats and
  diffs do not treat them as hand-authored source.
- Keep the current generated bot and analysis report artifacts tracked for now:
  the viewer-plus-data rewrite is a bigger architecture change reserved for the
  next slice.
- Review test/runtime cleanup opportunities after the `0.4` analyzer and Renju
  work.
- Make release builds fail if curated report artifacts are accidentally missing,
  with a local-only opt-out for development builds.
- Make sure release docs describe how report data is generated, committed,
  verified, and published.

### Slice 2: Report Viewer Architecture (`0.5.1`)

- Decide which report data remains checked in.
- Move report presentation out of Rust-generated monolithic HTML and into web
  viewer components.
- Keep compact `report.json` as the source data for published pages.
- Keep compact report JSON under `/bot-report/report.json` and
  `/analysis-report/report.json`, while making `/lab/` the public viewer route.
- Make report pages visually consistent with the game shell and visual-design
  pages.

### Slice 3: Product Explanation Pages

- Productize concise rules and guide explanations inside the app.
- Keep `Rules` basic: Gomoku, Freestyle, Renju, forbidden moves, and why Renju
  legality is more than rough shape counting.
- Use `Guide` for actual play lessons covering threes, fours, combo threats,
  and forced corridors.
- Keep About on GitHub/README; keep bot and analyzer explanations in the Lab
  Report instead of duplicating them as shallow app pages.
- Link it from Home/footer in a way that does not crowd the main play path.
- Use concrete screenshots or small board diagrams where text alone would be
  unclear.
- Keep model caveats honest: replay analysis is bounded explanation, not a full
  solver.

### Slice 4: Public Release Packaging

- Refresh hero capture and screenshot review.
- Update README and release copy around the current product loop.
- Prepare itch.io/dev-log copy.
- Run a public-alpha QA pass.
- Cut the first public-facing `0.5` release.

## Boundaries

- Do not start another broad bot research line by default.
- Do not add online/ranked/public-share scope in `0.5`.
- Do not hide the lab; translate it into product language.
- Do not make skins the headline. Visual polish matters, but only as part of
  presentation and public-release readiness.
- Do not overclaim replay analysis. Keep the bounded-model caveats visible.
