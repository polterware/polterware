import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { canOpenExternalUrl, contentSecurityPolicy, isAllowedRendererUrl, parseDeepLink } from "./navigation-policy.js";
const id = "01K6N3QWJ8P9A7B6C5D4E3F2G1";
describe("navigation policy", () => {
  it.each([`sparky://memory/${id}`, `sparky://memory/${id}?action=focus`, "sparky://today", "sparky://focus", "sparky://map", "sparky://calendar?date=2026-10-04", "sparky://search?q=hello"])("routes %s", url => expect(parseDeepLink(url)?.route).toMatch(/^\//));
  it("routes the Start focus command to /focus", () => expect(parseDeepLink("sparky://focus")).toEqual({ url: "sparky://focus", route: "/focus" }));
  it("routes the Map command to /map", () => expect(parseDeepLink("sparky://map")).toEqual({ url: "sparky://map", route: "/map" }));
  it.each(["https://today", "sparky://other", "sparky://memory/../today", "sparky://memory/bad", `sparky://memory/${id}?action=delete`,
    "sparky://calendar?date=2026-02-30", "sparky://today?token=x", "sparky://today#script", "sparky://user@today", "sparky://search?q=x&q=y",
    "sparky://focus?token=x", "sparky://focus/extra", "sparky://map?token=x", "sparky://map/extra"])("rejects %s", url => expect(parseDeepLink(url)).toBeNull());
  it("limits renderer navigation to its document", () => {
    const doc = "http://localhost:5173/";
    expect(isAllowedRendererUrl(`${doc}?surface=x#/today`, doc)).toBe(true);
    expect(isAllowedRendererUrl("http://evil.example/", doc)).toBe(false);
    expect(isAllowedRendererUrl("http://localhost:5173/other", doc)).toBe(false);
    expect(isAllowedRendererUrl("file:///tmp/secret", "file:///app/index.html")).toBe(false);
    expect(isAllowedRendererUrl("file:///app/index.html#/today", "file:///app/index.html")).toBe(true);
  });
  it.each(["javascript:alert(1)", "file:///etc/passwd", "sparky://today", "https://user:pass@example.com"])("blocks external %s", url => expect(canOpenExternalUrl(url)).toBe(false));
  it("opens only HTTP(S) in the system browser", () => expect(canOpenExternalUrl("https://example.com")).toBe(true));
  it("keeps remote maps off until enabled and disallows remote scripts", () => {
    expect(contentSecurityPolicy(false)).not.toContain("https://tiles.openfreemap.org");
    expect(contentSecurityPolicy(true)).toContain("connect-src 'self' https://tiles.openfreemap.org");
    expect(contentSecurityPolicy(true)).toContain("script-src 'self';");
    expect(contentSecurityPolicy(false)).not.toContain("unsafe-eval");
  });
  it("adds Nominatim to connect-src only while geocoding is enabled", () => {
    expect(contentSecurityPolicy(false)).not.toContain("nominatim.openstreetmap.org");
    expect(contentSecurityPolicy(true)).not.toContain("nominatim.openstreetmap.org");
    expect(contentSecurityPolicy(false, false, true)).toContain("https://nominatim.openstreetmap.org");
    expect(contentSecurityPolicy(true, false, true)).toContain("https://tiles.openfreemap.org https://nominatim.openstreetmap.org");
    expect(contentSecurityPolicy(true, false, true)).not.toContain("http://127.0.0.1");
    expect(contentSecurityPolicy(false, true, true)).not.toMatch(/connect-src[^;]*http:\/\/127\.0\.0\.1/);
  });
  it("does not ship a document CSP meta; Main owns the enforced policy", () => {
    const html = readFileSync(new URL("../renderer/index.html", import.meta.url), "utf8");
    expect(html).not.toMatch(/http-equiv=["']Content-Security-Policy["']/i);
  });
});
