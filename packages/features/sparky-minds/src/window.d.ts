import type { Mind } from "@polterware/sparky-contracts";

type SparkyBridge = {
  minds: {
    list(): Promise<{ minds: Mind[] }>;
    create(input: { parentId: string; name: string; color: string; icon: string; sortOrder: number }): Promise<unknown>;
    update(input: { id: string; expectedRev: string; name: string; color: string; icon: string }): Promise<unknown>;
    delete(input: { id: string; expectedRev: string }): Promise<{ deletedMindIds: string[]; movedMemoryCount: number }>;
    reorder(input: { parentId: string; orderedIds: string[] }): Promise<unknown>;
  };
  memories: {
    list(input: { mindId: string; includeDescendants: boolean; limit: number }): Promise<{ total: number }>;
  };
};

declare global {
  interface Window {
    sparky: SparkyBridge;
  }
}

export {};
