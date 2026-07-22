# Archive

Historical docs, superseded plans, rejected paths, and old release notes.

Archive docs are not authoritative unless an active reference doc explicitly
links to them for historical evidence. Prefer updating current reference docs
over patching archive files into partial correctness.

| Doc | What it was | Superseded by |
|---|---|---|
| `progress_v0.1.md` | Status log for the Phaser-era single-player game | `docs/reference/product/roadmap.md` |
| `web_frontend_plan.md` | Retrospective on the v0.1 Phaser FE work | `docs/reference/app/architecture.md` |
| `game_framework.md` | Generic "native core + bots + frontends" design doc that originally inspired the repo layout | `docs/reference/app/architecture.md` |
| `online_backend_design.md` | First-pass BE design — Firebase/Firestore/Cloud Run | `docs/reference/backend/backend.md` |
| `fe_gap_analysis.md` | Gap analysis of the v0.1 FE vs. planned online features (invalidated by the FE rewrite decision) | `docs/reference/app/architecture.md` (constraints), `docs/reference/product/roadmap.md` (phases) |
| `ui_ux_design_doc.md` | Drafty exploration of IA, screen strategy, visual language | `docs/reference/app/app_design.md` |
| `ui_language_doc.md` | Drafty design-system reference (overlapped with ui_ux_design_doc) | `docs/reference/app/ui_design.md` |
| `ui_implementation_backlog.md` | Drafty 14-milestone UI execution plan | `docs/reference/product/roadmap.md` |
| `fe_architecture_options.md` | Drafty FE stack options writeup | `docs/reference/app/architecture.md` |
| `v0_2_4_ui_polish_notes.md` | Triage/working notes captured during the `v0.2.4` shell polish pass | `docs/reference/app/ui_design.md`, `docs/working/ui_screenshot_review.md` |

These drafts were exploratory and not grounded in the v0.1 codebase. They're
preserved as artifacts of the "about to pivot" state but should not be read as
current direction.

The archive also keeps a few visual-exploration artifacts from the early
`v0.2` design pass:

- `v0_2_mock.png`
- `v0_2_visual.png`
- `v0_2_ui.md`
- `v0_2_themes.png`
- `visual_reference_v0_2_2.png`
- `capture_v0_2_4_match_desktop.gif`

Historical current-line ad-hoc planning notes also live here when they are
useful as evidence but should not compete with canonical docs. Active working
plans now live in [`../working/`](../working/).

| Doc | What it is | Canonical docs |
|---|---|---|
| `v0_3_plan.md` | Working plan for the `v0.3` cloud-backed continuity line | `docs/reference/product/roadmap.md`, `docs/reference/backend/backend.md`, `docs/reference/app/app_design.md` |
| `v0_3_completion_plan.md` | Practical rest-of-`0.3` release slicing after the product-identity roadmap pivot | `docs/reference/product/roadmap.md`, `docs/reference/backend/backend.md`, `docs/reference/ops/backend_infra.md` |
| `release_history.md` | Condensed completed phase history | `docs/reference/product/roadmap.md` |
| `performance_history.md` | Condensed performance/search experiment history | `docs/working/performance_tuning.md`, `docs/reference/lab/search_bot.md`, `docs/reference/lab/corridor_search.md` |
| `v0_4_plan.md` | Working plan for the `v0.4` lab-powered product identity line, from bot discovery through corridor-bot work before settings UI | `docs/reference/product/roadmap.md`, `docs/reference/product/product_strategy.md`, `docs/reference/backend/backend.md` |
| `v0_4_search_bot_enhancement_plan.md` | Historical retrospective of `0.4.0`-`0.4.2` search-bot experiments and rejected paths | `docs/reference/lab/search_bot.md`, `docs/reference/lab/corridor_search.md`, `docs/working/performance_tuning.md` |
| `v0_4_2_game_analysis_impl_notes.md` | Historical implementation notes, telemetry, and rejected replay-analysis proof policies from the `v0.4.2` corridor-search pass | `docs/reference/lab/corridor_search.md`, `docs/reference/lab/game_analysis.md` |
| `v0_4_3_corridor_bot_plan.md` | Working plan for the lab-only corridor-search-in-bot pass before UI settings | `docs/reference/product/roadmap.md`, `docs/reference/lab/corridor_search.md`, `docs/reference/lab/search_bot.md` |
| `v0_4_4_frontier_plan.md` | Working plan for the rolling-frontier threat-view line | `docs/reference/lab/search_bot.md`, `docs/reference/lab/corridor_search.md`, `docs/working/performance_tuning.md` |
| `v0_4_5_bot_controls_plan.md` | Working plan for exposing tested bot presets and narrow advanced bot controls in the web settings UI | `docs/reference/lab/search_bot.md`, `docs/reference/app/app_design.md`, `docs/reference/backend/data_model.md` |
| `v0_4_6_replay_analysis_plan.md` | Working plan for bringing corridor-search replay analysis into the product replay page and replacing the overloaded warning sprite sheet | `docs/reference/lab/game_analysis.md`, `docs/reference/lab/corridor_search.md`, `docs/reference/app/game_visual.md` |
| `v0_4_7_lethal_threat_plan.md` | Working plan for the lethal-threat layer and analyzer-onset follow-up | `docs/reference/lab/lethal_threats.md`, `docs/reference/lab/corridor_search.md`, `docs/reference/lab/game_analysis.md` |
| `v0_5_public_release_plan.md` | Working plan for the report, explanation-page, housekeeping, and public-alpha reconciliation line | `docs/reference/product/roadmap.md`, `release_history.md` |
