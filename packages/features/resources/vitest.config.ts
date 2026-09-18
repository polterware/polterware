import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    // Solid component icon tests need a DOM environment; run under desktop vitest.
    include: [
      "src/filter.test.ts",
      "src/map.test.ts",
      "src/provisioning.test.ts",
      "src/resource-status.test.ts",
      "src/resource-detail.test.ts",
      "src/node-detail.test.ts",
      "src/resources-create-dialog.test.ts",
      "src/create-host-node-dialog.test.ts",
      "src/create-process-dialog.test.ts",
    ],
  },
});
