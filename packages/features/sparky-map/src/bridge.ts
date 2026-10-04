import type { DescService, JsonValue } from "@bufbuild/protobuf";
import { fromJson } from "@bufbuild/protobuf";
import * as Contracts from "@polterware/sparky-contracts";
import { revisionString } from "./model";

type Rpc = (input?: JsonValue) => Promise<JsonValue>;
type WireNamespace = Record<string, Rpc>;

/** Structural subset of window.sparky. Feature packages must not declare Window.sparky. */
export type MapWireBridge = {
  places: WireNamespace;
  memories: WireNamespace;
  settings: WireNamespace;
  onChange(callback: (event: { resync: boolean; changes: { entity: string; id: string; op: string }[] }) => void): () => void;
};

export type MapBridge = {
  places: { set: Rpc; clear: Rpc; listInBounds: Rpc };
  memories: { get: Rpc; list: Rpc };
  settings: { get: Rpc; set: Rpc };
  onChange: MapWireBridge["onChange"];
};

function jsonValue(value: unknown): JsonValue {
  return JSON.parse(JSON.stringify(value ?? {}, (_key, item) => typeof item === "bigint" ? item.toString() : item)) as JsonValue;
}

function withStringRev(value: JsonValue): JsonValue {
  if (!value || typeof value !== "object" || Array.isArray(value) || !("expectedRev" in value) || value.expectedRev == null) return value;
  return { ...value, expectedRev: revisionString(value.expectedRev as string | number | bigint | undefined) };
}

function bind(source: WireNamespace, service: DescService): WireNamespace {
  const result: WireNamespace = {};
  for (const method of service.methods) {
    result[method.localName] = async (input = {}) => {
      const request = withStringRev(jsonValue(input ?? {}));
      fromJson(method.input, request, { ignoreUnknownFields: true });
      const json = jsonValue(await source[method.localName]!(request));
      fromJson(method.output, json, { ignoreUnknownFields: true });
      return json;
    };
  }
  return result;
}

export function createMapBridge(wire: MapWireBridge): MapBridge {
  return {
    places: bind(wire.places, Contracts.PlaceService) as MapBridge["places"],
    memories: bind(wire.memories, Contracts.MemoryService) as MapBridge["memories"],
    settings: bind(wire.settings, Contracts.SettingsService) as MapBridge["settings"],
    onChange: callback => wire.onChange(callback),
  };
}

/** Reads the preload wire without augmenting Window. Sparky Desktop owns that type. */
export function browserBridge(): MapBridge {
  const wire = Reflect.get(globalThis, "sparky") as MapWireBridge | undefined;
  if (!wire?.places || !wire.memories || !wire.settings) throw new Error("Sparky bridge unavailable. Open this page in Sparky Desktop.");
  return createMapBridge(wire);
}

export { revisionString };
