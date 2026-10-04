import type { DescMessage, DescMethodUnary, DescService, JsonValue, MessageShape as ContractMessage } from "@bufbuild/protobuf";
import { fromJson } from "@bufbuild/protobuf";
import * as Contracts from "@polterware/sparky-contracts";

type Wire<T> = T extends { $typeName: "google.protobuf.Timestamp" } ? string
  : T extends bigint ? string
  : T extends readonly (infer V)[] ? Wire<V>[]
  : T extends object ? { [K in keyof T as K extends "$typeName" ? never : K]?: Wire<T[K]> }
  : T;
export type JsonOf<S extends DescMessage> = Wire<ContractMessage<S>>;
type Namespace<S extends { method: object }> = {
  [K in keyof S["method"]]: S["method"][K] extends DescMethodUnary
    ? (input?: Wire<ContractMessage<S["method"][K]["input"]>>) => Promise<Wire<ContractMessage<S["method"][K]["output"]>>> : never;
};

type WireNamespace = Record<string, (input?: JsonValue) => Promise<JsonValue>>;

/** Structural subset of window.sparky. Feature packages must not declare Window.sparky. */
export type ReviewWireBridge = {
  review: WireNamespace;
  settings: WireNamespace;
  onChange(callback: (event: { resync: boolean; changes: { entity: string; id: string; op: string }[] }) => void): () => void;
};

export type ReviewBridge = {
  review: Namespace<typeof Contracts.ReviewService>;
  settings: Namespace<typeof Contracts.SettingsService>;
  onChange: ReviewWireBridge["onChange"];
};

export type ReviewJSON = JsonOf<typeof Contracts.GetReviewResponseSchema>;
export type ReviewDayJSON = NonNullable<ReviewJSON["days"]>[number];
export type SettingsJSON = JsonOf<typeof Contracts.SettingsSchema>;

function jsonValue(value: unknown): JsonValue {
  return JSON.parse(JSON.stringify(value ?? {}, (_key, item) => typeof item === "bigint" ? item.toString() : item)) as JsonValue;
}

function bind<S extends DescService>(source: WireNamespace, service: S) {
  const result: Record<string, (input?: JsonValue) => Promise<unknown>> = {};
  for (const method of service.methods) {
    result[method.localName] = async (input = {}) => {
      const request = jsonValue(input ?? {});
      fromJson(method.input, request, { ignoreUnknownFields: true });
      const json = jsonValue(await source[method.localName]!(request));
      fromJson(method.output, json, { ignoreUnknownFields: true });
      return json;
    };
  }
  return result as Namespace<S>;
}

export function createReviewBridge(wire: ReviewWireBridge): ReviewBridge {
  return {
    review: bind(wire.review, Contracts.ReviewService),
    settings: bind(wire.settings, Contracts.SettingsService),
    onChange: callback => wire.onChange(callback),
  };
}

/** Reads the preload wire without augmenting Window. Sparky Desktop owns that type. */
export function browserBridge(): ReviewBridge {
  const wire = Reflect.get(globalThis, "sparky") as ReviewWireBridge | undefined;
  if (!wire?.review || !wire.settings) {
    throw new Error("Sparky bridge unavailable. Open this page in Sparky Desktop.");
  }
  return createReviewBridge(wire);
}
