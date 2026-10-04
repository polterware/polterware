import { describe, expect, it } from "vitest";
import { DROP_PATH_UNAVAILABLE, droppedFilePath, linkHost, previewUrl, reorderIds, revisionString } from "./model";

const memory = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
const file = "01ARZ3NDEKTSV4RRFFQ69G5FB0.png";

describe("attachment preview URLs", () => {
  it("builds sparky-file URLs from a two-segment relPath and rejects file://", () => {
    expect(previewUrl(`${memory}/${file}`)).toBe(`sparky-file://attachment/${memory}/${file}`);
    expect(previewUrl(`${memory}\\${file}`)).toBe(`sparky-file://attachment/${memory}/${file}`);
    expect(previewUrl("/etc/passwd")).toBeUndefined();
    expect(previewUrl(`file://${memory}/${file}`)).toBeUndefined();
    expect(previewUrl(undefined)).toBeUndefined();
    expect(previewUrl(`${memory}/${file}`)?.startsWith("file:")).toBe(false);
  });
  it("reads an absolute dropped path when Electron provides File.path, otherwise nothing", () => {
    expect(droppedFilePath({ path: "/tmp/photo.png" } as File & { path: string })).toBe("/tmp/photo.png");
    expect(droppedFilePath({ name: "photo.png" } as File)).toBeUndefined();
    expect(droppedFilePath({ path: "relative.png" } as File & { path: string })).toBeUndefined();
    expect(DROP_PATH_UNAVAILABLE).toContain("filesystem path");
  });
  it("reorders ids without dropping entries and exposes link hosts without a favicon", () => {
    expect(reorderIds(["a", "b", "c"], 0, 1)).toEqual(["b", "a", "c"]);
    expect(reorderIds(["a", "b"], 0, -1)).toEqual(["a", "b"]);
    expect(linkHost("https://example.com/path")).toBe("example.com");
    expect(revisionString(9007199254740993n)).toBe("9007199254740993");
    expect(revisionString(undefined)).toBe("0");
  });
});
