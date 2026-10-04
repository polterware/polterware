import { readFileSync, readdirSync } from "node:fs";
import { describe, expect, it } from "vitest";

const protoDir = new URL("../../../sparky/schemas/sparky/v1/", import.meta.url);

const expected = {
  HealthService: ["Check"],
  MindService: ["List", "Get", "Create", "Update", "Delete", "Reorder"],
  MemoryService: [
    "List",
    "Get",
    "Create",
    "Update",
    "Delete",
    "SetStatus",
    "SetPinned",
    "Reorder",
    "ToggleCheckItem",
    "ReplaceChecklist",
    "SetTags",
    "BulkAction",
  ],
  TagService: ["List", "Create", "Update", "Delete", "Merge"],
  ScheduleService: ["Set", "Clear", "Preview", "ListOccurrences", "CompleteOccurrence", "ReopenOccurrence", "SetFocusRecipe"],
  PlaceService: ["Set", "Clear", "ListInBounds"],
  AttachmentService: ["ImportFromPath", "AddLink", "List", "Delete", "Reorder", "OpenInfo"],
  FocusService: ["GetDefaults", "SetDefaults", "RecordSession", "ListSessions"],
  ReviewService: ["Get"],
  SearchService: ["Search", "Status", "Reindex", "ConfigureEmbedding", "TestEmbedding"],
  ReminderService: ["Plan", "Ack", "Snooze", "ArmOneOff", "CancelOneOff", "Heartbeat", "Status"],
  ActivityService: ["List"],
  SettingsService: ["Get", "Set"],
  AgentAccessService: ["GetInfo", "RotateToken", "SetEnabled", "SetAllowFileAttach"],
  ChangeService: ["Watch"],
  MaintenanceService: ["Export", "Backup", "Doctor"],
};

function rpcs(source) {
  const found = {};
  let service = "";
  for (const line of source.split("\n")) {
    const serviceMatch = line.match(/^service (\w+)/);
    if (serviceMatch) service = serviceMatch[1];
    const rpcMatch = line.match(/^\s*rpc (\w+)\(/);
    if (rpcMatch && service) {
      found[service] ??= [];
      found[service].push(rpcMatch[1]);
    }
  }
  return found;
}

describe("RPC surface", () => {
  it("declares every service method from the data contract", () => {
    const found = {};
    for (const name of readdirSync(protoDir)) {
      if (!name.endsWith(".proto")) continue;
      const part = rpcs(readFileSync(new URL(name, protoDir), "utf8"));
      Object.assign(found, part);
    }
    expect(found).toEqual(expected);
  });
});
