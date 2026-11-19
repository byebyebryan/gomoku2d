# 0.6.5 My games and publishing UI captures

Captured November 19, 2026 for `/dev/reviews/my-games-publishing-065/`.
The 0.6.4 baseline came from the Pages artifact identified in the
[release record](../../archive/v0_6_4_release.md). Its tarball hash was checked
before serving it on loopback. The candidate came from an emulator-configured
production-style build of the local application. The exact base and changed
application file hashes are in the [source receipt](source.json).

Chrome for Testing 156.0.8078.4 used UTC, en-US, reduced motion and default text.
Desktop used DPR 1. Compact captures used mobile and touch emulation at DPR 3.
Playwright captured both sets at CSS-pixel scale. The stored PNGs are the
capture outputs.

The matched desktop views seeded the same two finished Freestyle games with
the local name Lin and Normal Bot as the opponent. One win ended after nine
moves at 20:18 UTC, and one loss ended after ten moves at 22:42 UTC the same day.
The candidate then promoted this fixture via a disposable emulator sign-in.
The signed-in history was captured on desktop, at tablet width, in portrait and
in short landscape. The earlier game went through public profile setup, move 4
selection, browser Back from a temporary move 0, and explicit confirmation.
The capture checked heading focus on entry to setup and the draft, then waited
for the matching Edit action to receive focus before framing the listing.

## Image inventory

| Capture | CSS viewport / PNG dimensions | SHA-256 |
|---|---|---|
| [0.6.4 · history rows](my-games-before.png) | 1440 × 900 | `5b2b0d656b33e20573ad079c66aadb73329ace4a5c53028f2b4f6d123951fd23` |
| [0.6.5 candidate · final-board previews](my-games-after.png) | 1440 × 900 | `2ea7c9a2b3cc9cce8307b6319e8b6464481b5fb9acb6955f73d1f1f9062d537a` |
| [My games · portrait](my-games-publish-portrait.png) | 390 × 844 | `f4d73cde38a7767d284923c707cb19203d217d713617ed060dc043ae8a9aed69` |
| [Continue publishing · profile enabled](continue-after-setup.png) | 1440 × 900 | `e0ee3b2222311dc08cc1679507076c0537c7d9d3d72b6db37d38bb8f087b619b` |
| [Publish draft · move 4 restored](publish-draft-after-back.png) | 1440 × 900 | `e0987c4342165351ed10c7acee8588097b610acc342805cd907bd659ffc68a1a` |
| [Published games · Edit focused](published-listing-focus.png) | 1440 × 900 | `b58af44944352153b17f37c38a7919a57b2be27988ec04654fb95ed7f6a20cda` |
| [My games · signed-in desktop](my-games-publish-desktop.png) | 1440 × 900 | `7d527e8a8a01a8bd93ea8edd29723303eeb68c0b7b34f26a4e78383f75e8272e` |
| [My games · tablet width](my-games-publish-tablet.png) | 1024 × 960 | `4d219f4e63c3859478e3532f6263f76201ef20c0145d4ff73f12367717f0656b` |
| [My games · short landscape](my-games-publish-landscape.png) | 844 × 390 | `056e01c6760dae8678c05388f53bb2898b5b9c1efb0cbafb66b7ec10cf5d74a2` |

## Scope

The signed-in layouts show the corrections from the second visual review.
Inspect and Publish fit in the wide and short-landscape row. Headings are absent
when the compact row takes over. The remaining 320 px / 200% text tab wrapping
also occurs in the 0.6.4 release. These checks use Chrome layout emulation and
disposable accounts.

The implementation’s C23 journey covers profile setup, explicit publication,
opening and Back behavior, and matching focus after publication and duplicate
detection. The capture reused the same emulator launcher and fixture helpers.
