import type { ArchitectureDocument } from "@polterware/frontend-canvas";
import type { FixtureMode } from "@polterware/frontend-state";

export interface ResolvedArchitectureDocument {
  document: ArchitectureDocument;
  dataSource: "fixture" | "empty";
}

export function canPersistArchitecture(fixtureMode: FixtureMode): boolean {
  return fixtureMode !== "force";
}

export function resolveArchitectureDocument(input: {
  loaded: ArchitectureDocument;
  fixture: ArchitectureDocument;
  fixtureMode: FixtureMode;
  dataSource?: "fixture" | "empty";
}): ResolvedArchitectureDocument {
  if (input.fixtureMode === "force") {
    return { document: input.fixture, dataSource: "fixture" };
  }

  if (
    input.loaded.nodes.length === 0
    && input.fixtureMode === "fallback"
    && input.fixture.nodes.length > 0
  ) {
    return { document: input.fixture, dataSource: "fixture" };
  }

  return {
    document: input.loaded,
    dataSource: input.loaded.nodes.length > 0
      ? (input.dataSource ?? "fixture")
      : (input.dataSource ?? "empty"),
  };
}

export function orderArchitectureDetailNodes<T extends { id: string }>(
  nodes: readonly T[],
  fixture: boolean,
): T[] {
  if (!fixture) return [...nodes];
  return [...nodes].sort((left, right) => {
    if (left.id === "n-pay") return -1;
    if (right.id === "n-pay") return 1;
    return 0;
  });
}
