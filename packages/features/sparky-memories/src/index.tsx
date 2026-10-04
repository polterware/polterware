import type { Component } from "solid-js";
import type { SparkyFeature, DetailSectionDef } from "@polterware/feature-sparky-shell/feature";
import { memoriesStore } from "./store";
import { browserBridge } from "./model";
import { MemoryDetail } from "./memory-detail";
import { InboxPage, MemoryCollection, TodayPage } from "./memory-list";

// Optional registry additions are requested in w2-memories-1; no other feature import is required.
const feature: SparkyFeature & {
  memoryDetail: Component<{ memoryId: string; sections: DetailSectionDef[] }>;
  memoryCollection: Component<{ mindId: string }>;
} = {
  id: "memories",
  routes: [{ path: "/today", title: "Today", component: TodayPage }, { path: "/inbox", title: "Inbox", component: InboxPage }],
  commands: [{ id: "memories.create", label: "New memory", keywords: ["add", "create"], run: async () => { const memory = await memoriesStore.create({ title: "Untitled memory" }); await browserBridge().app.openDeepLink(`sparky://memory/${memory.id}`); } }],
  memoryDetail: MemoryDetail,
  memoryCollection: MemoryCollection,
};
export default feature;
export { MemoryDetail, MemoryCollection, TodayPage, InboxPage };
export { createMemoriesStore, memoriesStore } from "./store";
export type { MemoryDetailProps } from "./memory-detail";
export type { MemoryCollectionProps } from "./memory-list";
