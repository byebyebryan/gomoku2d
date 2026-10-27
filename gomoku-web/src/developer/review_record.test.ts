import { describe, expect, it } from "vitest";
import records from "./reviews.json";
import { isReviewRecord, type ReviewRecord } from "./review_record";
import { selectedIcons } from "./icon_catalog";

const reference = records[0] as unknown as ReviewRecord;

function retrospectiveReview() {
  const base = structuredClone(reference);
  const normalComparisonIds = base.comparisons
    .filter((pair) => pair.purpose !== "layout-check")
    .map((pair) => pair.id);
  return {
    ...base,
    disposition: "retrospective review" as const,
    versions: ["0.5.1", "0.6.2"],
    process: [
      {
        id: "reconstruct",
        title: "Reconstruct the documented flow",
        detail: "Follow the source notes and compare each normal review pair.",
        comparisons: normalComparisonIds,
      },
    ],
  };
}

describe("curated public review contract", () => {
  it("accepts the deliberately selected historical records", () => {
    expect(records.every(isReviewRecord)).toBe(true);
    expect(new Set(records.map((record) => record.id)).size).toBe(records.length);
  });
  it("rejects an unsupported comparison purpose", () => {
    expect(
      isReviewRecord({
        ...reference,
        comparisons: [{ ...reference.comparisons[0], purpose: "unknown" }],
      }),
    ).toBe(false);
  });
  it("accepts a versioned retrospective review with complete process grouping", () => {
    expect(isReviewRecord(retrospectiveReview())).toBe(true);
  });
  it.each([
    ["empty", []],
    ["duplicate", ["0.6.2", "0.6.2"]],
    ["unordered", ["0.6.2", "0.5.1"]],
    ["noncanonical", ["v0.5.1", "0.6.2"]],
  ])("rejects %s version lists", (_label, versions) => {
    expect(isReviewRecord({ ...retrospectiveReview(), versions })).toBe(false);
  });
  it.each([
    ["an empty process", (value: ReturnType<typeof retrospectiveReview>) => {
      value.process = [];
    }],
    ["empty step references", (value: ReturnType<typeof retrospectiveReview>) => {
      value.process[0].comparisons = [];
    }],
    ["a blank step title", (value: ReturnType<typeof retrospectiveReview>) => {
      value.process[0].title = "  ";
    }],
    ["an unknown comparison", (value: ReturnType<typeof retrospectiveReview>) => {
      value.process[0].comparisons[0] = "missing-comparison";
    }],
    ["a layout check", (value: ReturnType<typeof retrospectiveReview>) => {
      const layoutCheck = {
        ...value.comparisons[0],
        id: "extra-layout-check",
        purpose: "layout-check" as const,
      };
      value.comparisons.push(layoutCheck);
      value.process[0].comparisons.push(layoutCheck.id);
    }],
    ["duplicate references", (value: ReturnType<typeof retrospectiveReview>) => {
      value.process[0].comparisons.push(value.process[0].comparisons[0]);
    }],
    ["a missing normal comparison", (value: ReturnType<typeof retrospectiveReview>) => {
      value.process[0].comparisons.pop();
    }],
    ["an unsafe step ID", (value: ReturnType<typeof retrospectiveReview>) => {
      value.process[0].id = "../reconstruct";
    }],
    ["duplicate step IDs", (value: ReturnType<typeof retrospectiveReview>) => {
      value.process.push({ ...value.process[0], comparisons: [value.process[0].comparisons[0]] });
    }],
  ])("rejects process grouping with %s", (_label, mutate) => {
    const value = retrospectiveReview();
    mutate(value);
    expect(isReviewRecord(value)).toBe(false);
  });
  it("requires versions and process grouping for a retrospective review", () => {
    const { versions: _versions, ...withoutVersions } = retrospectiveReview();
    const { process: _process, ...withoutProcess } = retrospectiveReview();
    expect(isReviewRecord(withoutVersions)).toBe(false);
    expect(isReviewRecord(withoutProcess)).toBe(false);
  });
  it("accepts capture provenance only for application captures", () => {
    const base = structuredClone(reference);
    const capture = {
      ...base.comparisons[0].media[0],
      captureType: "retrospective",
    };
    expect(
      isReviewRecord({
        ...base,
        comparisons: [{ ...base.comparisons[0], media: [capture] }],
      }),
    ).toBe(true);
    expect(
      isReviewRecord({
        ...base,
        comparisons: [
          {
            ...base.comparisons[0],
            media: [{ ...capture, captureType: "synthetic" }],
          },
        ],
      }),
    ).toBe(false);
    expect(
      isReviewRecord({
        ...base,
        comparisons: [
          {
            ...base.comparisons[0],
            media: [{ ...capture, kind: "design mockup" }],
          },
        ],
      }),
    ).toBe(false);
  });
  it.each(["../secret.png", "https://example.test/image.png", "nested/file.png"])(
    "rejects a media path outside its served record: %s",
    (file) => {
      const value = structuredClone(reference);
      value.comparisons[0].media[0].file = file;
      expect(isReviewRecord(value)).toBe(false);
    },
  );
  it("rejects unsupported schemas, missing source identity and private-host links", () => {
    const badSchema = { ...reference, schema: 2 };
    expect(isReviewRecord(badSchema)).toBe(false);
    const noIdentity = structuredClone(reference);
    noIdentity.comparisons[0].media[0].sha256 = "";
    expect(isReviewRecord(noIdentity)).toBe(false);
    expect(
      isReviewRecord({
        ...reference,
        sources: [{ label: "Private preview", url: "http://starship:8787/reports/private/" }],
      }),
    ).toBe(false);
  });
});
describe("canonical inspector URL selection", () => {
  it("allows only unique canonical names and a bounded comparison", () => {
    expect(
      selectedIcons(
        new URLSearchParams("compare=missing,share,share,../../profile,save,publish,home,bot"),
      ),
    ).toEqual(["share", "save", "publish", "home"]);
    expect(selectedIcons(new URLSearchParams("icon=unknown"))).toEqual(["share"]);
  });
});
