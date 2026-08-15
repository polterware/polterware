import type { ArchitectureDocument } from "./types";
import { MAX_BODY_BYTES } from "./types";
import { createEmptyDocument, validateDocument, type ValidationIssue } from "./document";
import { isArchitectureDocumentV1, migrateLegacyToV1, type LegacyCanvasBody } from "./migrate";

export type ParseResult = {
  doc: ArchitectureDocument;
  warnings: ValidationIssue[];
  migrated: boolean;
};

export function serializeDocument(doc: ArchitectureDocument): string {
  const body = JSON.stringify({
    formatVersion: 1,
    id: doc.id,
    systemId: doc.systemId,
    workspaceId: doc.workspaceId,
    activeEnvironmentId: doc.activeEnvironmentId,
    nodes: doc.nodes,
    relationships: doc.relationships,
    boundaries: doc.boundaries,
    preferences: doc.preferences,
    viewport: doc.viewport,
    createdAt: doc.createdAt,
    updatedAt: doc.updatedAt,
    meta: doc.meta,
  });
  if (body.length > MAX_BODY_BYTES) {
    throw new Error(`Document exceeds max size (${MAX_BODY_BYTES} bytes)`);
  }
  return body;
}

export function deserializeAndMigrate(
  json: string,
  systemId = "sys-local",
): ParseResult {
  const warnings: ValidationIssue[] = [];
  if (!json || json === "{}" || json.trim() === "") {
    return { doc: createEmptyDocument({ systemId }), warnings, migrated: false };
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(json) as unknown;
  } catch {
    return {
      doc: createEmptyDocument({ systemId }),
      warnings: [{ level: "error", message: "Malformed JSON; loaded empty document" }],
      migrated: false,
    };
  }

  if (isArchitectureDocumentV1(parsed)) {
    const issues = validateDocument(parsed);
    const errors = issues.filter((i) => i.level === "error");
    if (errors.length) {
      return {
        doc: createEmptyDocument({ systemId }),
        warnings: issues,
        migrated: false,
      };
    }
    // drop dangling relationships
    const nodeIds = new Set(parsed.nodes.map((n) => n.id));
    const relationships = parsed.relationships.filter(
      (r) => nodeIds.has(r.sourceNodeId) && nodeIds.has(r.targetNodeId),
    );
    if (relationships.length !== parsed.relationships.length) {
      warnings.push({
        level: "warning",
        message: "Dropped relationships with missing endpoints",
      });
    }
    return {
      doc: {
        ...parsed,
        boundaries: parsed.boundaries ?? [],
        preferences: parsed.preferences ?? {},
        viewport: parsed.viewport ?? { x: 0, y: 0, zoom: 1 },
        relationships,
      },
      warnings: [...warnings, ...issues.filter((i) => i.level === "warning")],
      migrated: false,
    };
  }

  // Future unknown version
  const maybe = parsed as { formatVersion?: number };
  if (typeof maybe.formatVersion === "number" && maybe.formatVersion > 1) {
    return {
      doc: createEmptyDocument({ systemId }),
      warnings: [
        {
          level: "error",
          message: `Unknown format version ${maybe.formatVersion}; document not loaded`,
        },
      ],
      migrated: false,
    };
  }

  // Legacy format 0
  const legacy = parsed as LegacyCanvasBody;
  const doc = migrateLegacyToV1(legacy, systemId);
  const issues = validateDocument(doc);
  return {
    doc,
    warnings: [
      { level: "warning", message: "Migrated legacy layout body to formatVersion 1" },
      ...issues,
    ],
    migrated: true,
  };
}

/** Validate import without replacing — returns errors if unsafe. */
export function validateImportJson(json: string): {
  ok: boolean;
  result?: ParseResult;
  error?: string;
} {
  try {
    const result = deserializeAndMigrate(json);
    const hard = result.warnings.filter((w) => w.level === "error");
    if (hard.length && result.doc.nodes.length === 0 && !result.migrated) {
      // empty after error
      if (hard.some((h) => h.message.includes("Malformed") || h.message.includes("Unknown format"))) {
        return { ok: false, error: hard[0]!.message, result };
      }
    }
    if (hard.some((h) => h.message.includes("Unknown format"))) {
      return { ok: false, error: hard[0]!.message, result };
    }
    try {
      JSON.parse(json);
    } catch {
      return { ok: false, error: "Malformed JSON" };
    }
    // Reject empty parse of garbage that became empty without being intentionally empty
    if (json.trim() && result.doc.nodes.length === 0 && hard.length) {
      return { ok: false, error: hard.map((h) => h.message).join("; "), result };
    }
    return { ok: true, result };
  } catch (e) {
    return { ok: false, error: String(e) };
  }
}

export function sanitizeText(input: string, max = 2000): string {
  return input.replace(/[\u0000-\u0008\u000B\u000C\u000E-\u001F]/g, "").slice(0, max);
}
