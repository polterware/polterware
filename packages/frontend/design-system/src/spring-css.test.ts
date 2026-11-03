import { describe, expect, it } from "vitest";
import { springCssProperties, springLinearEasing } from "./spring-css";
import { spring } from "./springs";

function points(easing: string): number[] {
  return easing
    .replace(/^linear\(|\)$/g, "")
    .split(",")
    .map((value) => Number.parseFloat(value));
}

describe("springLinearEasing", () => {
  it("starts at 0 and lands on 1 for every tier", () => {
    for (const tier of ["fast", "moderate", "slow"] as const) {
      const values = points(springLinearEasing(tier));
      expect(values[0]).toBe(0);
      expect(values.at(-1)).toBeCloseTo(1, 3);
    }
  });

  it("keeps the critically damped tiers from overshooting", () => {
    for (const tier of ["fast", "moderate"] as const) {
      expect(Math.max(...points(springLinearEasing(tier)))).toBeLessThanOrEqual(1.0001);
    }
  });

  it("lets the slow tier overshoot a little (bounce 0.12)", () => {
    const peak = Math.max(...points(springLinearEasing("slow")));
    expect(peak).toBeGreaterThan(1);
    expect(peak).toBeLessThan(1.05);
  });
});

describe("springCssProperties", () => {
  it("mirrors the tier durations in milliseconds", () => {
    const properties = springCssProperties();
    expect(properties["--spring-slow-duration"]).toBe(`${spring.slow.duration * 1000}ms`);
    expect(properties["--spring-moderate-exit-duration"]).toBe(
      `${spring.moderate.exit.duration * 1000}ms`,
    );
    expect(properties["--spring-fast-ease"]).toMatch(/^linear\(/);
  });
});
