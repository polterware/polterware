import { describe, expect, it } from "vitest";
import { CATALOG_PROVIDER_MOCKS } from "@polterware/feature-shell";
import { cliStateLabel, providerByName } from "./setup-wizard-helpers";

describe("CLI visual state", () => {
  it("labels every supported state", () => {
    expect(["cli-unavailable", "ready", "authenticating", "connected", "failed"].map((state) => cliStateLabel(state as never))).toEqual(["CLI unavailable", "Ready", "Authenticating", "Connected", "Failed"]);
  });

  it("resolves providers by id and name", () => {
    expect(providerByName(CATALOG_PROVIDER_MOCKS, "aws")?.name).toBe("AWS");
    expect(providerByName(CATALOG_PROVIDER_MOCKS, "Google Cloud")?.binary).toBe("gcloud");
  });
});
