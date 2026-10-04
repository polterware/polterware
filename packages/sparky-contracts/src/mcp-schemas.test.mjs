import { readFileSync, readdirSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { parameterDescriptions, validate } from "./json-schema.mjs";

const schemaDir = new URL("../../../sparky/backend/providers/mcp/schemas/", import.meta.url);

const toolNames = [
  "get_current_time",
  "list_minds",
  "list_tags",
  "list_memories",
  "search_memories",
  "get_memory",
  "list_occurrences",
  "get_review",
  "list_activity",
  "create_memories",
  "update_memories",
  "set_memory_status",
  "toggle_check_item",
  "delete_memories",
  "create_mind",
  "update_mind",
  "delete_mind",
  "add_attachments",
];

const samples = {
  get_current_time: {},
  list_minds: { parent: "Work" },
  list_tags: {},
  list_memories: { mind: "Work", includeDescendants: true, status: "active", limit: 20, order: "due" },
  search_memories: { query: "milk", limit: 20 },
  get_memory: { id: "01ARZ3NDEKTSV4RRFFQ69G5FAV", include: ["note", "checklist", "schedule"] },
  list_occurrences: { from: "2026-10-03T00:00:00.000Z", to: "2026-10-10T00:00:00.000Z", includeCompleted: true },
  get_review: { period: "week" },
  list_activity: { limit: 20, actor: "mcp:claude" },
  create_memories: {
    items: [
      {
        title: "Buy milk",
        mind: "Work",
        tags: ["home"],
        due: "2026-10-04",
        pinned: false,
        checklist: ["Milk", { title: "Bread", detail: "whole" }],
        schedule: {
          at: "2026-10-05T08:00:00.000Z",
          tz: "UTC",
          repeat: { freq: "weekly", every: 1, weekdays: ["mon", "wed"] },
          focus: { workMin: 25, shortBreakMin: 5, longBreakMin: 15, cycles: 4, autoContinue: true },
        },
        place: { name: "Store", lat: -23.5, lng: -46.6, radiusM: 200, event: "enter" },
        links: [{ url: "https://example.com", title: "Example" }],
      },
    ],
  },
  update_memories: {
    items: [{ id: "01ARZ3NDEKTSV4RRFFQ69G5FAV", expectedRev: 2, note: null, addTags: ["home"], schedule: null }],
  },
  set_memory_status: { id: "01ARZ3NDEKTSV4RRFFQ69G5FAV", status: "completed", occurrence: "2026-10-05T08:00:00.000Z" },
  toggle_check_item: { memoryId: "01ARZ3NDEKTSV4RRFFQ69G5FAV", itemTitle: "Milk" },
  delete_memories: { ids: ["01ARZ3NDEKTSV4RRFFQ69G5FAV"], confirm: true },
  create_mind: { name: "Work", parent: "Life", color: "#112233", icon: "briefcase", sortOrder: 0 },
  update_mind: { name: "Work", newName: "Office", color: null, expectedRev: 2 },
  delete_mind: { name: "Work", confirm: true },
  add_attachments: {
    memoryId: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    links: [{ url: "https://example.com" }],
    files: [{ path: "/tmp/note.txt" }],
  },
};

function loadTool(name) {
  return JSON.parse(readFileSync(new URL(`${name}.json`, schemaDir), "utf8"));
}

describe("MCP tool schemas", () => {
  const tools = toolNames.map(loadTool);

  it("covers the 18 tools", () => {
    expect(tools.map((tool) => tool.name)).toEqual(toolNames);
    const files = readdirSync(schemaDir).filter((name) => name.endsWith(".json") && !name.endsWith(".schema.json"));
    expect(files.sort()).toEqual(toolNames.map((name) => `${name}.json`).sort());
  });

  it("keeps descriptions and the tools/list payload inside the token budget", () => {
    for (const tool of tools) {
      expect(tool.description.length).toBeLessThanOrEqual(300);
      for (const description of parameterDescriptions(tool.inputSchema)) {
        expect(description.length).toBeLessThanOrEqual(120);
      }
    }
    const payload = JSON.stringify(tools.map(({ name, description, inputSchema }) => ({ name, description, inputSchema })));
    expect(Buffer.byteLength(payload)).toBeLessThanOrEqual(6144);
  });

  it.each(toolNames)("accepts a sample call for %s", (name) => {
    const tool = loadTool(name);
    expect(validate(tool.inputSchema, samples[name])).toEqual([]);
  });

  it("keeps the search tool on the text-search contract", () => {
    const tool = loadTool("search_memories");
    expect(Object.keys(tool.inputSchema.properties)).toEqual(["query", "mind", "tag", "limit"]);
    expect(Object.keys(tool.outputSchema.properties)).toEqual(["hits", "notice"]);
    expect(tool.outputSchema.required).toEqual(["hits"]);
    expect(validate(tool.outputSchema, { hits: [] })).toEqual([]);
  });

  it("rejects a create without a title and a delete without confirm", () => {
    expect(validate(loadTool("create_memories").inputSchema, { items: [{}] }).length).toBeGreaterThan(0);
    expect(validate(loadTool("delete_memories").inputSchema, { ids: ["x"] }).length).toBeGreaterThan(0);
    expect(validate(loadTool("delete_memories").inputSchema, { ids: ["x"], confirm: false }).length).toBeGreaterThan(0);
  });

  it("validates the shared schedule and place shapes", () => {
    const schedule = JSON.parse(readFileSync(new URL("schedule.schema.json", schemaDir), "utf8"));
    const place = JSON.parse(readFileSync(new URL("place.schema.json", schemaDir), "utf8"));
    const sample = samples.create_memories.items[0];
    expect(validate(schedule, sample.schedule)).toEqual([]);
    expect(validate(place, sample.place)).toEqual([]);
    expect(validate(schedule, { repeat: { freq: "daily" } }).length).toBeGreaterThan(0);
    expect(validate(place, { lat: 120, lng: 0 }).length).toBeGreaterThan(0);
  });
});
