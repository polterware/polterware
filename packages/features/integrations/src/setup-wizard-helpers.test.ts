import { describe, expect, it } from "vitest";
import { CATALOG_TEST_PROVIDERS } from "./test-fixtures";
import { cliStateLabel, providerByName } from "./setup-wizard-helpers";

describe("CLI visual state", () => {
  it("labels every supported state", () => {
    const states: Array<Parameters<typeof cliStateLabel>[0]> = [
      "cli-unavailable",
      "ready",
      "authenticating",
      "connected",
      "failed",
    ];
    expect(states.map(cliStateLabel)).toEqual([
      "CLI unavailable",
      "Ready",
      "Authenticating",
      "Connected",
      "Failed",
    ]);
  });

  it("resolves providers by id and name", () => {
    expect(providerByName(CATALOG_TEST_PROVIDERS, "aws")?.name).toBe("AWS");
    expect(providerByName(CATALOG_TEST_PROVIDERS, "Google Cloud")?.binary).toBe("gcloud");
  });
});
