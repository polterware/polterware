import type { DescMessage, DescMethodUnary, MessageShape, DescService, JsonValue } from "@bufbuild/protobuf";
import { fromJson } from "@bufbuild/protobuf";
import * as Contracts from "@polterware/sparky-contracts";
import { revisionString } from "./mapping";

type Wire<T> = T extends { $typeName: "google.protobuf.Timestamp" } ? string
  : T extends bigint ? string
  : T extends readonly (infer V)[] ? Wire<V>[]
  : T extends object ? { [K in keyof T as K extends "$typeName" ? never : K]?: Wire<T[K]> }
  : T;
export type JsonOf<S extends DescMessage> = Wire<MessageShape<S>>;
type Namespace<S extends { method: object }> = {
  [K in keyof S["method"]]: S["method"][K] extends DescMethodUnary
    ? (input?: Wire<MessageShape<S["method"][K]["input"]>>) => Promise<Wire<MessageShape<S["method"][K]["output"]>>> : never;
};
export type SettingsJSON = JsonOf<typeof Contracts.SettingsSchema>;
export type OccurrenceJSON = JsonOf<typeof Contracts.OccurrenceSchema>;

type Rpc = (input?: JsonValue) => Promise<JsonValue>;
type WireNamespace = Record<string, Rpc>;

/** Structural subset of window.sparky. Feature packages must not declare Window.sparky. */
export type CalendarWireBridge = {
  memories: WireNamespace;
  minds: WireNamespace;
  schedule: WireNamespace;
  settings: WireNamespace;
  onChange(callback: (event: { resync: boolean; changes: { entity: string; id: string; op: string }[] }) => void): () => void;
};

export type CalendarBridge = {
  memories: { create: Rpc; get: Rpc };
  minds: { list: Rpc };
  schedule: Pick<Namespace<typeof Contracts.ScheduleService>, "listOccurrences"> & { set: Rpc };
  settings: Pick<Namespace<typeof Contracts.SettingsService>, "get">;
  onChange: CalendarWireBridge["onChange"];
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

export function createCalendarBridge(wire: CalendarWireBridge): CalendarBridge {
  return {
    memories: bind(wire.memories, Contracts.MemoryService) as CalendarBridge["memories"],
    minds: bind(wire.minds, Contracts.MindService) as CalendarBridge["minds"],
    schedule: bind(wire.schedule, Contracts.ScheduleService) as CalendarBridge["schedule"],
    settings: bind(wire.settings, Contracts.SettingsService) as CalendarBridge["settings"],
    onChange: callback => wire.onChange(callback),
  };
}

/** Reads the preload wire without augmenting Window. Sparky Desktop owns that type. */
export function browserBridge(): CalendarBridge {
  const wire = Reflect.get(globalThis, "sparky") as CalendarWireBridge | undefined;
  if (!wire?.schedule || !wire.memories) throw new Error("Sparky bridge unavailable. Open this page in Sparky Desktop.");
  return createCalendarBridge(wire);
}

export { revisionString };
