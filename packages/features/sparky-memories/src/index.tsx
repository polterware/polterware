import type { Component } from "solid-js";
import type { SparkyFeature, DetailSectionDef } from "@polterware/feature-sparky-shell/feature";
import { browserBridge } from "./bridge";
import { MemoryDetailHost } from "./host";
import { MemoryDetail } from "./memory-detail";
import { InboxPage, MemoryCollection, MindMemoriesPage, TodayPage } from "./memory-list";
import { memoriesStore } from "./store";

const feature: SparkyFeature & {
  memoryDetail: Component<{ memoryId: string; sections: DetailSectionDef[] }>;
  memoryCollection: Component<{ mindId: string }>;
} = {
  id: "memories",
  routes: [
    { path: "/today", title: "Today", component: TodayPage },
    { path: "/inbox", title: "Inbox", component: InboxPage },
    { path: "/memory/:id", title: "Memory", component: MemoryDetailHost },
  ],
  commands: [{
    id: "memories.create",
    label: "Create untitled memory",
    keywords: ["add", "create", "new", "memory"],
    run: async () => {
      const memory = await memoriesStore.create({ title: "Untitled memory" });
      await browserBridge().app.openDeepLink(`sparky://memory/${memory.id}`);
    },
  }],
  memoryDetail: MemoryDetailHost,
  memoryCollection: MindMemoriesPage,
};
export default feature;
export { MemoryDetail, MemoryDetailHost, MemoryCollection, MindMemoriesPage, TodayPage, InboxPage };
export { createMemoriesStore, memoriesStore } from "./store";
export { createMemoriesBridge, browserBridge } from "./bridge";
export type { MemoryDetailProps } from "./memory-detail";
export type { MemoryCollectionProps } from "./memory-list";
