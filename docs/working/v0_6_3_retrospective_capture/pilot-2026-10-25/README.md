# Retrospective UI pilot: v0.2.4 and v0.6.2

New captures from the exact release tag commits, using the same four local guest
scenes and two viewports. [capture.json](capture.json) owns source/build/tool,
fixture/context, readiness, network, dimensions and SHA-256 provenance. Its
capture-script hash identifies the exact harness used; the current script may
later evolve. No pixels were cropped, recoloured or reconstructed after capture.

The 16 PNGs are grouped by source tag and desktop/portrait profile. Application
and lockfile inputs remained unchanged. Node 26 was used with separately locked
installs although the tags declare 20/24. Both builds deliberately had no cloud
configuration. Portrait 412×800 is provisional generic Android touch emulation at
DPR 1, not a measured OnePlus/browser-chrome profile. Renderer type is unobserved;
Phaser animation phase is not fixed. Human and physical-device review remain open.

The retrospective viewer presents each scene/profile as a v0.2.4/v0.6.2 pair.
Original historical media and the earlier frozen 0.6.2 screen reference remain
intact. The agent inspected all 16 images. The desktop Profile action clipping
at 1280 and later long-name Match ellipsis are visible source behavior, recorded
for separate follow-up rather than corrected in tagged applications.

[Validation](validation.json) qualifies the integrated viewer, standalone export,
build/source delivery and cleanup. The [capture procedure](../../v0_6_3_retrospective_capture.md)
describes how to prepare new isolated sources and receipts. These captures do
not require keeping the temporary source worktrees or historical previews alive.
