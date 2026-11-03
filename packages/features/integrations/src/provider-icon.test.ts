import { describe, expect, it } from "vitest";
import { providerIconKey } from "./provider-icon";

describe("provider icon keys", () => {
  it("maps supported providers", () => {
    expect(providerIconKey({ name: "GitHub" })).toBe("github");
    expect(providerIconKey({ name: "Google Cloud" })).toBe("gcp");
    expect(providerIconKey({ name: "Terraform" })).toBe("terraform");
    expect(providerIconKey({ name: "Docker" })).toBe("docker");
  });
});
