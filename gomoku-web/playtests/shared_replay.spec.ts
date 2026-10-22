import { expect, test } from "@playwright/test";

const WIN_MOVES = [110, 0, 111, 1, 112, 2, 113, 3, 114];

function sharedLink(overrides: Record<string, unknown> = {}) {
  const wire = {
    v: 1, r: "freestyle", m: Buffer.from(WIN_MOVES).toString("base64url"),
    b: { k: "h", n: "Alice", u: "alice-public" }, w: { k: "h", n: "Bob" },
    at: 4,
    ...overrides,
  };
  return `/replay/shared/#g2r1=${Buffer.from(JSON.stringify(wire)).toString("base64url")}`;
}

test("shared replay loads directly, preserves names and position, and never imports on open", async ({ page }) => {
  const databaseRequests: string[] = [];
  page.on("request", (request) => {
    if (/firestore\.googleapis\.com|google\.firestore/.test(request.url())) databaseRequests.push(request.url());
  });
  const response = await page.goto(sharedLink());
  expect(response?.status()).toBe(200);
  await expect(page.getByRole("heading", { name: "Replay Analysis" })).toBeVisible();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 4 / 9");
  await expect(page.getByTestId("replay-player-row-black")).toContainText("Alice");
  await expect(page.getByTestId("replay-player-row-white")).toContainText("Bob");
  await expect(page.getByRole("link", { name: "Alice" })).toHaveAttribute("href", "/player/?id=alice-public");
  await page.getByRole("button", { name: "Next move", exact: true }).click();
  await expect(page.getByTestId("replay-move-count")).toHaveText("Move 5 / 9");
  await expect(page.getByTestId("replay-analysis-status")).not.toHaveText("");
  const historyCount = await page.evaluate(() => {
    const local = JSON.parse(localStorage.getItem("gomoku2d.local-profile.v5") ?? "{}");
    return local.state?.matchHistory?.replayMatches?.length ?? 0;
  });
  expect(historyCount).toBe(0);
  expect(databaseRequests).toEqual([]);
});

test("share preview controls exported names and has a usable clipboard fallback", async ({ page }, testInfo) => {
  await page.goto(sharedLink());
  await page.getByRole("button", { name: "Share replay", exact: true }).click();
  const input = page.getByRole("textbox", { name: "Replay link", exact: true });
  await expect(input).toHaveValue(/\/replay\/shared\/#g2r1=/);
  const exported = async () => JSON.parse(Buffer.from(
    new URL(await input.inputValue()).hash.slice("#g2r1=".length), "base64url",
  ).toString("utf8"));
  await expect(page.getByRole("button", { name: "Move 4", exact: true })).toHaveAttribute("aria-pressed", "true");
  const finalMove = page.getByRole("button", { name: "Final move", exact: true });
  await finalMove.focus();
  await finalMove.press("Enter");
  expect((await exported()).at).toBe(9);
  await page.getByRole("button", { name: "Move 4", exact: true }).click();
  await page.getByRole("button", { name: "Generic names", exact: true }).click();
  const copied = await input.inputValue();
  const json = JSON.parse(Buffer.from(new URL(copied).hash.slice("#g2r1=".length), "base64url").toString("utf8"));
  expect(json.b).toEqual({ k: "h", n: "Player 1" });
  expect(json.w).toEqual({ k: "h", n: "Player 2" });
  expect(json.at).toBe(4);
  const sharing = page.getByRole("region", { name: "Replay sharing" });
  await sharing.scrollIntoViewIfNeeded();
  await page.screenshot({ path: testInfo.outputPath("share-desktop.png") });
  await page.setViewportSize({ width: 390, height: 844 });
  await sharing.scrollIntoViewIfNeeded();
  await page.screenshot({ path: testInfo.outputPath("share-mobile.png") });
  await page.evaluate(() => Object.defineProperty(navigator, "clipboard", {
    configurable: true, value: { writeText: async () => { throw new Error("Clipboard denied"); } },
  }));
  await page.getByRole("button", { name: "Copy link", exact: true }).click();
  await expect(sharing.getByRole("status"))
    .toContainText("Copy the selected link below.");
  await expect(input).toBeFocused();
});

test("sharing replaces the replay deck, pauses playback and returns to the same position", async ({ page }, testInfo) => {
  await page.goto(sharedLink());
  const board = page.getByRole("grid", { name: "Gomoku board" });
  const boardCanvas = page.locator("canvas");
  await expect(boardCanvas).toBeVisible();
  const canvasHandle = await boardCanvas.elementHandle();
  await page.getByRole("button", { name: "Auto play", exact: true }).click();
  await expect(page.getByRole("button", { name: "Pause", exact: true })).toBeVisible();
  const share = page.getByRole("button", { name: "Share replay", exact: true });
  await share.click();
  const back = page.getByRole("button", { name: "Back to replay", exact: true });
  await expect(back).toBeFocused();
  await expect(page.getByRole("heading", { name: "Share replay", exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Next move", exact: true })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Save replay", exact: true })).toHaveCount(0);
  await expect(board).toBeVisible();
  expect(await canvasHandle!.evaluate((element) => element.isConnected)).toBe(true);
  const input = page.getByRole("textbox", { name: "Replay link", exact: true });
  const opened = await input.inputValue();
  const at = JSON.parse(Buffer.from(new URL(opened).hash.slice("#g2r1=".length), "base64url").toString("utf8")).at;
  await page.getByRole("button", { name: "Generic names", exact: true }).click();
  await input.focus();
  await page.keyboard.press("Escape");
  await expect(share).toBeFocused();
  await expect(page.getByTestId("replay-move-count")).toHaveText(`Move ${at} / 9`);
  await expect(page.getByRole("button", { name: "Auto play", exact: true })).toBeVisible();
  await expect(back).toHaveCount(0);
  await share.click();
  await expect(page.getByRole("button", { name: "Generic names", exact: true })).toHaveAttribute("aria-pressed", "true");
  await back.click();
  await expect(share).toBeFocused();
  await page.setViewportSize({ width: 390, height: 844 });
  await share.click();
  await expect(back).toBeInViewport();
  await page.screenshot({ path: testInfo.outputPath("share-swap-mobile.png"), animations: "disabled" });
  await back.click();
  await expect(share).toBeFocused();
  await expect(page.getByTestId("replay-move-count")).toHaveText(`Move ${at} / 9`);
  await page.emulateMedia({ reducedMotion: "reduce" });
  await share.click();
  expect(await page.getByRole("region", { name: "Replay sharing" }).evaluate((element) => getComputedStyle(element).animationName)).toBe("none");
});

for (const [label, link] of [
  ["unsupported version", sharedLink({ v: 2 })],
  ["moves after a win", sharedLink({ m: Buffer.from([...WIN_MOVES, 224]).toString("base64url") })],
  ["malformed encoding", "/replay/shared/#g2r1=***"],
] as const) {
  test(`shared replay rejects ${label}`, async ({ page }) => {
    await page.goto(link);
    await expect(page.getByRole("heading", { name: "Replay unavailable" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Share replay", exact: true })).toHaveCount(0);
  });
}
