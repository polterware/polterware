import { describe, expect, it } from "vitest";
import { canOpenExternalUrl, contentSecurityPolicy, isAllowedRendererUrl, parseDeepLink } from "./navigation-policy.js";
const id = "01K6N3QWJ8P9A7B6C5D4E3F2G1";
describe("navigation policy", () => {
  it.each([`sparky://memory/${id}`, `sparky://memory/${id}?action=focus`, "sparky://today", "sparky://calendar?date=2026-10-04", "sparky://search?q=hello"])("routes %s", url => expect(parseDeepLink(url)?.route).toMatch(/^\//));
  it.each(["https://today", "sparky://other", "sparky://memory/../today", "sparky://memory/bad", `sparky://memory/${id}?action=delete`,
    "sparky://calendar?date=2026-02-30", "sparky://today?token=x", "sparky://today#script", "sparky://user@today", "sparky://search?q=x&q=y"])("rejects %s", url => expect(parseDeepLink(url)).toBeNull());
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
});
