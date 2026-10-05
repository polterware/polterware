import type { DescService, JsonValue } from "@bufbuild/protobuf";
import { fromJson } from "@bufbuild/protobuf";
import * as Contracts from "@polterware/sparky-contracts";
import { revisionString, type AttachmentsBridge } from "./model";

type WireNamespace = Record<string, (input?: JsonValue) => Promise<JsonValue>>;

/** Structural subset of window.sparky. Feature packages must not declare Window.sparky. */
export type AttachmentsWireBridge = {
  attachments: WireNamespace;
  memories: WireNamespace;
  settings: WireNamespace;
  onChange(callback: (event: { resync: boolean; changes: { entity: string; id: string; op: string }[] }) => void): () => void;
  app: AttachmentsBridge["app"];
};

function jsonValue(value: unknown): JsonValue {
  return JSON.parse(JSON.stringify(value ?? {}, (_key, item) => typeof item === "bigint" ? item.toString() : item)) as JsonValue;
}
function withStringRev(value: JsonValue): JsonValue {
  if (!value || typeof value !== "object" || Array.isArray(value) || !("expectedRev" in value) || value.expectedRev == null) return value;
  return { ...value, expectedRev: revisionString(value.expectedRev as string | number | bigint | undefined) };
}
function bind<S extends DescService>(source: WireNamespace, service: S): Record<string, (input?: JsonValue) => Promise<unknown>> {
  const result: Record<string, (input?: JsonValue) => Promise<unknown>> = {};
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

export function createAttachmentsBridge(wire: AttachmentsWireBridge): AttachmentsBridge {
  return {
    attachments: bind(wire.attachments, Contracts.AttachmentService) as AttachmentsBridge["attachments"],
    memories: bind(wire.memories, Contracts.MemoryService) as AttachmentsBridge["memories"],
    settings: bind(wire.settings, Contracts.SettingsService) as AttachmentsBridge["settings"],
    onChange: callback => wire.onChange(callback),
    app: wire.app,
  };
}

/** Reads the preload wire without augmenting Window. Sparky Desktop owns that type. */
export function browserBridge(): AttachmentsBridge {
  const wire = Reflect.get(globalThis, "sparky") as AttachmentsWireBridge | undefined;
  if (!wire?.attachments) throw new Error("Sparky bridge unavailable. Open this page in Sparky Desktop.");
  return createAttachmentsBridge(wire);
}

export { revisionString };
export type { AttachmentsBridge };
