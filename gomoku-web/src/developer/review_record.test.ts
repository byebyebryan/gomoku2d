import { describe, expect, it } from "vitest";
import records from "./reviews.json";
import { isReviewRecord } from "./review_record";
import { selectedIcons } from "./icon_catalog";

describe("curated public review contract", () => {
  it("accepts the deliberately selected historical records", () => {
    expect(records.every(isReviewRecord)).toBe(true);
    expect(new Set(records.map((record) => record.id)).size).toBe(records.length);
  });
  it("rejects an unsupported comparison purpose", () => {
    expect(
      isReviewRecord({
        ...records[0],
        comparisons: [{ ...records[0].comparisons[0], purpose: "unknown" }],
      }),
    ).toBe(false);
  });
  it.each(["../secret.png", "https://example.test/image.png", "nested/file.png"])(
    "rejects a media path outside its served record: %s",
    (file) => {
      const value = structuredClone(records[0]);
      value.comparisons[0].media[0].file = file;
      expect(isReviewRecord(value)).toBe(false);
    },
  );
  it("rejects unsupported schemas, missing source identity and private-host links", () => {
    const badSchema = { ...records[0], schema: 2 };
    expect(isReviewRecord(badSchema)).toBe(false);
    const noIdentity = structuredClone(records[0]);
    noIdentity.comparisons[0].media[0].sha256 = "";
    expect(isReviewRecord(noIdentity)).toBe(false);
    expect(
      isReviewRecord({
        ...records[0],
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
