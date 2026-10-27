import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { expect, test } from "@playwright/test";
import type { ReviewRecord } from "../src/developer/review_record.js";

const DESIGN_CASE_IDS = [
  "board-first-shell-022",
  "portrait-play-024",
  "replay-analysis-048",
  "product-pages-052",
  "replay-sidebar-061",
  "public-player-062",
  "action-icons-061",
  "mobile-records-062",
] as const;
const NEW_AND_EXPANDED_CASE_IDS = [
  "board-first-shell-022",
  "portrait-play-024",
  "replay-analysis-048",
  "product-pages-052",
  "replay-sidebar-061",
  "public-player-062",
] as const;
const REFERENCE_GALLERY_IDS = ["core-screens-062", "ui-evolution-024-062"] as const;
const EXPECTED_PHASES = ["0.6", "0.5", "0.4", "0.2"];

async function curatedReviews(): Promise<ReviewRecord[]> {
  return JSON.parse(await readFile(resolve("src/developer/reviews.json"), "utf8")) as ReviewRecord[];
}

function recordFor(records: ReviewRecord[], id: string): ReviewRecord {
  const record = records.find((item) => item.id === id);
  expect(record, `review record ${id} must exist in the canonical catalog`).toBeDefined();
  return record!;
}

function latestVersion(record: ReviewRecord) {
  return record.versions?.at(-1) ?? "";
}

function phase(record: ReviewRecord) {
  return latestVersion(record).split(".").slice(0, 2).join(".");
}

function versionLabel(record: ReviewRecord) {
  const versions = record.versions;
  if (!versions?.length) return record.scope;
  return versions.length === 1
    ? `v${versions[0]}`
    : `v${versions[0]}–${versions[versions.length - 1]}`;
}

function captureDescription(media: ReviewRecord["comparisons"][number]["media"][number]) {
  return media.captureType === "historical"
    ? "Original capture"
    : media.captureType === "retrospective"
      ? "Retrospective capture"
      : media.kind;
}

test("design review index keeps source versions, order, and legacy destinations", async ({
  page,
}) => {
  const records = await curatedReviews();
  expect(records).toHaveLength(10);
  expect(new Set(records.map((record) => record.id)).size).toBe(records.length);

  const designReviews = records.filter(
    (record) =>
      record.disposition !== "screen reference" &&
      record.disposition !== "retrospective comparison",
  );
  expect(designReviews).toHaveLength(8);
  expect(designReviews.map((record) => record.id).sort()).toEqual([...DESIGN_CASE_IDS].sort());

  const expectedOrder = [...designReviews].sort(
    (a, b) =>
      latestVersion(b).localeCompare(latestVersion(a), "en", { numeric: true }) ||
      b.date.localeCompare(a.date) ||
      a.title.localeCompare(b.title),
  );
  const expectedPhases = [...new Set(expectedOrder.map(phase))];
  expect(expectedPhases).toEqual(EXPECTED_PHASES);

  const referenceReviews = records.filter(
    (record) =>
      record.disposition === "screen reference" ||
      record.disposition === "retrospective comparison",
  );
  expect(referenceReviews.map((record) => record.id).sort()).toEqual(
    [...REFERENCE_GALLERY_IDS].sort(),
  );

  await page.goto("/dev/reviews/");
  await expect(page.getByRole("heading", { name: "Design reviews", exact: true })).toBeVisible();
  const reviewNavigation = page.getByRole("navigation", { name: "UI review views" });
  await expect(reviewNavigation.getByRole("link")).toHaveCount(3);
  await expect(
    reviewNavigation.getByRole("link", { name: "Current screens", exact: true }),
  ).toHaveAttribute("href", "/dev/reviews/core-screens-062/");
  await expect(
    reviewNavigation.getByRole("link", { name: "UI history", exact: true }),
  ).toHaveAttribute("href", "/dev/reviews/history/");
  await expect(
    reviewNavigation.getByRole("link", { name: "Design reviews", exact: true }),
  ).toHaveAttribute("aria-current", "page");
  await expect(page.locator(".review-phase > h2")).toHaveText(expectedPhases);

  const displayedIds = await page
    .locator(".review-phase article[data-review]")
    .evaluateAll((cards) =>
      cards.map((card) => (card as HTMLElement).dataset.review),
    );
  expect(displayedIds).toEqual(expectedOrder.map((record) => record.id));
  for (const record of expectedOrder) {
    const card = page.locator(`.review-phase article[data-review="${record.id}"]`);
    await expect(card.getByRole("heading", { name: record.title, exact: true })).toBeVisible();
    await expect(card.locator(".dev-meta").first()).toHaveText(versionLabel(record));
    await expect(
      card.getByRole("link", { name: `Review: ${record.title}`, exact: true }),
    ).toHaveAttribute("href", `/dev/reviews/${record.id}/`);
  }

  const references = page.locator("details.review-reference-links");
  await expect(references).not.toHaveAttribute("open");
  await references.locator("summary").click();
  const referenceHrefs = await references
    .getByRole("link")
    .evaluateAll((links) => links.map((link) => (link as HTMLAnchorElement).getAttribute("href")));
  expect(referenceHrefs).toEqual(
    referenceReviews.map((record) => `/dev/reviews/${record.id}/`),
  );

  for (const [path, activeLink] of [
    ["/dev/reviews/core-screens-062/", "Current screens"],
    ["/dev/reviews/history/", "UI history"],
    ["/dev/reviews/", "Design reviews"],
  ]) {
    await page.goto(path);
    const navigation = page.getByRole("navigation", { name: "UI review views" });
    await expect(navigation.getByRole("link")).toHaveCount(3);
    await expect(navigation.getByRole("link", { name: activeLink, exact: true })).toHaveAttribute(
      "aria-current",
      "page",
    );
  }
});

test("new and expanded reviews keep process grouping and canonical media evidence", async ({
  page,
}) => {
  test.setTimeout(120_000);
  const records = await curatedReviews();
  const replayOriginals = recordFor(records, "replay-analysis-048").comparisons
    .flatMap((comparison) => comparison.media)
    .filter((media) => media.captureType === "historical");
  const publicPlayerMockups = recordFor(records, "public-player-062").comparisons
    .flatMap((comparison) => comparison.media)
    .filter((media) => media.kind === "design mockup");
  const publicPlayerMockupFiles = new Set(publicPlayerMockups.map((media) => media.file));
  expect(replayOriginals).toHaveLength(4);
  expect(publicPlayerMockupFiles.size).toBe(5);

  for (const id of NEW_AND_EXPANDED_CASE_IDS) {
    const record = recordFor(records, id);
    expect(record.versions?.length, `${id} needs source versions`).toBeGreaterThan(0);
    expect(record.process?.length, `${id} needs review process steps`).toBeGreaterThan(0);
    const process = record.process!;
    const normalComparisons = record.comparisons.filter(
      (comparison) => comparison.purpose !== "layout-check",
    );
    const groupedIds = process.flatMap((step) => step.comparisons);
    expect(new Set(groupedIds).size, `${id} process references must be unique`).toBe(
      groupedIds.length,
    );
    expect([...groupedIds].sort()).toEqual(normalComparisons.map((pair) => pair.id).sort());

    if (id === "public-player-062") expect(groupedIds.at(-1)).toBe("responsive");

    await page.goto(`/dev/reviews/${id}/`);
    await expect(page.getByRole("heading", { name: record.title, exact: true })).toBeVisible();
    await expect(
      page
        .getByRole("navigation", { name: "UI review views" })
        .getByRole("link", { name: "Design reviews", exact: true }),
    ).toHaveAttribute("aria-current", "page");
    const metadata = page.locator(".review-case > .dev-heading > .dev-meta");
    await expect(metadata).toContainText(record.disposition);
    if (record.disposition === "retrospective review")
      await expect(metadata).toContainText("Assembled");

    const processSections = page.locator(
      ".review-process-step[data-review-step]",
    );
    await expect(processSections).toHaveCount(process.length);
    const jumpNavigation = page.getByRole("navigation", { name: "Screens in this review" });
    await expect(jumpNavigation.getByRole("link")).toHaveCount(normalComparisons.length);

    for (const step of process) {
      const stepSection = page.locator(`[data-review-step="${step.id}"]`);
      await expect(stepSection.getByRole("heading", { name: step.title, exact: true })).toBeVisible();
      await expect(stepSection.locator(":scope > .review-step-copy")).toHaveText(step.detail);
      const groupedComparisonIds = await stepSection
        .locator(".review-comparison[data-comparison]")
        .evaluateAll((sections) =>
          sections.map((section) => (section as HTMLElement).dataset.comparison),
        );
      expect(groupedComparisonIds).toEqual(step.comparisons);

      for (const comparisonId of step.comparisons) {
        const comparison = record.comparisons.find((pair) => pair.id === comparisonId)!;
        const section = stepSection.locator(`[data-comparison="${comparison.id}"]`);
        await expect(
          section.getByRole("heading", { name: comparison.title, exact: true }),
        ).toBeVisible();
        const figures = section.locator("figure");
        await expect(figures).toHaveCount(comparison.media.length);

        for (const [index, media] of comparison.media.entries()) {
          const figure = figures.nth(index);
          await expect(figure.locator("figcaption strong")).toHaveText(media.label);
          await expect(figure.locator("figcaption span")).toHaveText(
            captureDescription(media),
          );
          const image = figure.locator(".review-capture img");
          await expect(image).toHaveAttribute("alt", media.alt);
          await expect(image).toHaveAttribute("width", String(media.width));
          await expect(image).toHaveAttribute("height", String(media.height));
          const source = await image.getAttribute("src");
          expect(new URL(source!, page.url()).pathname).toContain(
            `/reviews/${record.id}/${media.file}`,
          );
          await image.scrollIntoViewIfNeeded();
          const dimensions = await image.evaluate(async (element) => {
            const node = element as HTMLImageElement;
            node.loading = "eager";
            await node.decode();
            return { width: node.naturalWidth, height: node.naturalHeight };
          });
          expect(dimensions).toEqual({ width: media.width, height: media.height });
        }
        expect(
          await section.evaluate((element) => {
            const image = element.querySelector(".review-pair img");
            const control = element.querySelector(".dev-tools button");
            return (
              !!image &&
              !!control &&
              !!(image.compareDocumentPosition(control) & Node.DOCUMENT_POSITION_FOLLOWING)
            );
          }),
          `${record.id}/${comparison.id} controls follow the comparison images`,
        ).toBe(true);
      }
    }

    const layoutIds = record.comparisons
      .filter((comparison) => comparison.purpose === "layout-check")
      .map((comparison) => comparison.id);
    if (layoutIds.length)
      await expect(page.locator(".review-layout-checks")).not.toHaveAttribute("open");
    expect(
      await page
        .locator(".review-layout-checks [data-comparison]")
        .evaluateAll((sections) =>
          sections.map((section) => (section as HTMLElement).dataset.comparison),
        ),
    ).toEqual(layoutIds);

    const findings = page.locator(".review-findings > li");
    await expect(findings).toHaveCount(record.findings.length);
    for (const finding of record.findings) {
      await expect(findings.filter({ hasText: finding.title })).toContainText(finding.detail);
    }
  }
});

test("nested review comparisons retain deep links, control state, Back and reload", async ({
  page,
}) => {
  const records = await curatedReviews();
  const record = recordFor(records, "public-player-062");
  const comparisonId = record.process!
    .flatMap((step) => step.comparisons)
    .find((id) => id !== "responsive");
  expect(comparisonId).toBeTruthy();
  const comparison = record.comparisons.find((pair) => pair.id === comparisonId)!;

  await page.goto(`/dev/reviews/${record.id}/?pair=${comparison.id}`);
  const nestedSection = page.locator(
    `.review-process-step [data-comparison="${comparison.id}"]`,
  );
  await expect(nestedSection).toBeFocused();
  await expect.poll(() => new URL(page.url()).searchParams.get("pair")).toBe(comparison.id);

  await nestedSection.getByRole("button", { name: "Native pixels", exact: true }).click();
  await expect.poll(() => new URL(page.url()).searchParams.get("native")).toBe("1");
  await expect(nestedSection.locator(".review-native")).toBeVisible();
  await nestedSection.getByRole("button", { name: "Full captures", exact: true }).click();
  await expect.poll(() => new URL(page.url()).searchParams.get("full")).toBe("0");
  await expect(nestedSection.locator(".review-native")).toBeVisible();
  await expect(nestedSection.locator(".review-full")).toHaveCount(0);

  await page.reload();
  await expect(nestedSection.locator(".review-native")).toBeVisible();
  await expect(nestedSection.locator(".review-full")).toHaveCount(0);
  await page.goBack();
  await expect.poll(() => new URL(page.url()).searchParams.get("full")).toBeNull();
  await expect.poll(() => new URL(page.url()).searchParams.get("native")).toBe("1");
  await expect(nestedSection.locator(".review-native")).toBeVisible();
  await expect(nestedSection.locator(".review-full")).toBeVisible();
  await page.goBack();
  await expect.poll(() => new URL(page.url()).searchParams.get("native")).toBeNull();
  await expect(nestedSection.locator(".review-native")).toHaveCount(0);
  await page.goForward();
  await expect(nestedSection.locator(".review-native")).toBeVisible();
});

test("all new review cases reflow at desktop, 412px, 320px and enlarged text", async ({
  page,
}) => {
  test.setTimeout(120_000);
  const states = [
    [1280, 900, "100%"],
    [412, 800, "100%"],
    [320, 800, "100%"],
    [320, 800, "200%"],
  ] as const;

  for (const id of NEW_AND_EXPANDED_CASE_IDS) {
    await page.goto(`/dev/reviews/${id}/`);
    await expect(page.locator(".review-case h1")).toBeVisible();
    for (const [width, height, fontSize] of states) {
      await page.setViewportSize({ width, height });
      await page.evaluate((size) => (document.documentElement.style.fontSize = size), fontSize);
      await page.evaluate(() => document.fonts.ready);
      expect(
        await page
          .locator(".dev-page")
          .evaluate((element) => element.scrollWidth - element.clientWidth),
        `${id} at ${width}px and ${fontSize} text`,
      ).toBeLessThanOrEqual(1);
    }
  }
});
