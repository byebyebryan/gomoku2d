# Released 0.6.2 core-screen reference

Captured on October 24, 2026 from detached release source
`37a122d2bd8e899bcfc581055e685c0327f43cb4`, application source tree
`fcfe58b5924011ec78e053f64e8ea5a594e7563d`. Application source, assets, package
and Vite configuration were unchanged. This is a reference for the released
player screens, not the later developer-area implementation.

[capture.json](capture.json) retains browser, viewport, scale, fixtures, timestamps
and every PNG hash. The 24 files cover six screens × four layout conditions.
The curated public record selects desktop/portrait pairs for all six, plus
landscape/enlarged pairs for Match, Profile and Public player (18 files).

[capture.spec.ts](capture.spec.ts) is the frozen one-off capture harness, executed
in the detached checkout using the release cloud helper and offset-1000
emulators. Installed dependency bytes were reused. The container additionally
mounted the original checkout read-only so its dependency symlink could resolve;
the initial attempt without that mount failed before any capture. The final
run passed in Chromium. Its public replay carries the publisher UID and asserts
visible opponent metadata. Initial fixture and overly exact text-locator
corrections preceded this first frozen reference. No runtime source patch was made.

Capture commands and logs: `gomoku-web/test-results/v063-core-capture.log` and
`v063-core-capture-retry.log`, `v063-core-capture-qualified.log` and
`v063-core-capture-final.log`. The owned release-checkout emulators/preview were
stopped by their harness. New screenshots should use a new directory and record;
do not overwrite these historical bytes. Human/physical-device review remains
unperformed for this newly prepared set.
