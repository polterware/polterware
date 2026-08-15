import { describe, expect, it } from "vitest";
import { RESOURCE_FIXTURES, NODE_FIXTURES, PROCESS_FIXTURES } from "@polterware/feature-resources";
import { RUN_FIXTURES, WORKFLOW_FIXTURES, SCHEDULE_FIXTURES } from "@polterware/feature-operations";
import { CATALOG_PROVIDER_MOCKS, DOCUMENT_MOCKS, RUNBOOK_MOCKS } from "@polterware/feature-shell";

describe("retained fixture coverage", () => {
  it("covers every retained operational collection", () => {
    expect(RESOURCE_FIXTURES.length).toBeGreaterThan(0);
    expect(NODE_FIXTURES.length).toBeGreaterThan(0);
    expect(PROCESS_FIXTURES.length).toBeGreaterThan(0);
    expect(RUN_FIXTURES.length).toBeGreaterThan(0);
    expect(WORKFLOW_FIXTURES.length).toBeGreaterThan(0);
    expect(SCHEDULE_FIXTURES.length).toBeGreaterThan(0);
    expect(DOCUMENT_MOCKS.length).toBeGreaterThan(0);
    expect(RUNBOOK_MOCKS.length).toBeGreaterThan(0);
  });

  it("keeps the CLI catalog exact", () => {
    expect(CATALOG_PROVIDER_MOCKS.map((provider) => provider.name)).toEqual(["GitHub", "GitLab", "AWS", "Google Cloud", "Azure", "Vercel", "Supabase", "Terraform", "Docker"]);
  });
});
