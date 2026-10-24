# 0.6.3 implementation execution

Bryan accepted the [visual study](v0_6_3_design/README.md) and authorized full
implementation on October 24, 2026. Scope follows the
[brief](v0_6_3_developer_site_plan.md), checkpoints C–F. Commits stay local;
release/deployment remain separate. The 0.6.2 manual preview, emulators, Carbon
tunnel and demo fixtures remain owned by the existing services.

## Integration checkpoint C–D

The root entry now selects a lazy public developer tree or the player tree.
Fresh developer entries import no private controllers/stores. Leaving the player
unsubscribes its UI controllers without clearing singleton stores or cancelling
in-flight promises; returning re-evaluates pending synchronization. Public
profiles/shared replays keep their existing authentication-aware player boundary.

One developer frame owns scrolling, navigation, skip/focus and public recovery.
Overview and Process follow the approved study. Lab retains its recorded report
loaders, proof-board renderer and drill-down. Secondary ranking metrics move into
expanded rows; report JSON is unchanged. Report loads now survive local-view
changes rather than being cancelled into a permanently loading state.

Assets use canonical SVGs and actual CSS values. Full icon inspection is URL
based. Sprite previews reuse the pure RGB multiplication operation and runtime
black/white/overlay values, with one selected loop, stepping, reduced motion,
visibility and offscreen controls. Historical asset-only teal remains a named
preview colour, not a claim about the game renderer.

Catalog, current screen reference, reusable export, documentation reconciliation
and overall browser/emulator review are in progress. No release claim yet.
