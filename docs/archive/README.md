# Archive

Superseded docs from the pre-pivot era (tag `v0.1`). Kept for reference — the
thinking, tradeoffs, and rejected options live here so the canonical docs in
`docs/` don't have to constantly caveat "but we considered X."

| Doc | What it was | Superseded by |
|---|---|---|
| `progress_v0.1.md` | Status log for the Phaser-era single-player game | `docs/roadmap.md` |
| `web_frontend_plan.md` | Retrospective on the v0.1 Phaser FE work | `docs/architecture.md` |
| `game_framework.md` | Generic "native core + bots + frontends" design doc that originally inspired the repo layout | `docs/architecture.md` |
| `online_backend_design.md` | First-pass BE design — Firebase/Firestore/Cloud Run | `docs/backend.md` |
| `fe_gap_analysis.md` | Gap analysis of the v0.1 FE vs. planned online features (invalidated by the FE rewrite decision) | `docs/architecture.md` (constraints), `docs/roadmap.md` (phases) |
| `ui_ux_design_doc.md` | Drafty exploration of IA, screen strategy, visual language | `docs/app_design.md` |
| `ui_language_doc.md` | Drafty design-system reference (overlapped with ui_ux_design_doc) | `docs/ui_design.md` |
| `ui_implementation_backlog.md` | Drafty 14-milestone UI execution plan | `docs/roadmap.md` |
| `fe_architecture_options.md` | Drafty FE stack options writeup | `docs/architecture.md` |
| `v0_2_4_ui_polish_notes.md` | Triage/working notes captured during the `v0.2.4` shell polish pass | `docs/ui_design.md`, `docs/ui_screenshot_review.md` |

These drafts were exploratory and not grounded in the v0.1 codebase. They're
preserved as artifacts of the "about to pivot" state but should not be read as
current direction.

The archive also keeps a few visual-exploration artifacts from the early
`v0.2` design pass:

- `v0_2_mock.png`
- `v0_2_visual.png`
- `v0_2_ui.md`
- `v0_2_themes.png`

Current-line ad-hoc planning notes can also live here when they are useful
during implementation but should not compete with canonical docs:

| Doc | What it is | Canonical docs |
|---|---|---|
| `v0_3_plan.md` | Working plan for the `v0.3` cloud-backed continuity line | `docs/roadmap.md`, `docs/backend.md`, `docs/app_design.md` |
| `v0_3_completion_plan.md` | Practical rest-of-`0.3` release slicing after the product-identity roadmap pivot | `docs/roadmap.md`, `docs/backend.md`, `docs/backend_infra.md` |
| `v0_4_plan.md` | Working plan for the `v0.4` lab-powered product identity line, starting with bot discovery before settings UI | `docs/roadmap.md`, `docs/product.md`, `docs/backend.md` |
