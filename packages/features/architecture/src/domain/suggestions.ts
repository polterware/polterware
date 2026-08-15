import type { ArchNode, RelationshipType } from "@polterware/frontend-canvas";

type Rule = {
  source?: string | string[];
  target?: string | string[];
  suggest: RelationshipType[];
};

const RULES: Rule[] = [
  {
    source: ["web_application", "mobile_application", "desktop_application"],
    target: ["api", "service"],
    suggest: ["calls", "depends_on", "connects_to"],
  },
  {
    source: ["api", "service", "worker", "serverless_function"],
    target: ["database", "cache", "search_engine", "data_warehouse"],
    suggest: ["reads_from", "writes_to", "connects_to", "depends_on"],
  },
  {
    source: ["api", "service"],
    target: ["queue", "event_stream"],
    suggest: ["publishes_to", "depends_on"],
  },
  {
    source: ["worker", "scheduled_job", "serverless_function"],
    target: ["queue", "event_stream"],
    suggest: ["consumes_from", "publishes_to", "depends_on"],
  },
  {
    source: ["web_application", "mobile_application", "api", "service"],
    target: ["auth_provider"],
    suggest: ["connects_to", "depends_on"],
  },
  {
    source: ["repository", "ci_pipeline", "package"],
    target: ["runtime_host", "api_gateway", "cdn"],
    suggest: ["deployed_on", "depends_on"],
  },
  {
    source: ["api_gateway", "load_balancer", "cdn"],
    target: ["service", "api", "web_application"],
    suggest: ["routes_to", "calls"],
  },
  {
    source: ["api", "service", "worker"],
    target: ["storage"],
    suggest: ["writes_to", "reads_from", "connects_to"],
  },
  {
    source: ["service", "external_service", "saas_provider"],
    target: [
      "web_application",
      "api",
      "service",
      "mobile_application",
      "worker",
    ],
    suggest: ["connects_to", "depends_on"],
  },
  {
    source: ["api", "service"],
    target: ["runtime_host"],
    suggest: ["deployed_on", "depends_on"],
  },
  {
    source: ["system"],
    target: undefined,
    suggest: ["connects_to", "depends_on"],
  },
];

function matchType(spec: string | string[] | undefined, actual: string): boolean {
  if (!spec) return true;
  if (Array.isArray(spec)) return spec.includes(actual);
  return spec === actual;
}

/**
 * Ordered relationship type suggestions for a source→target pair.
 * First entry is the default focus in the type popover.
 */
export function suggestRelationshipTypes(
  source: ArchNode,
  target: ArchNode,
): RelationshipType[] {
  const out: RelationshipType[] = [];
  const seen = new Set<string>();

  for (const rule of RULES) {
    if (!matchType(rule.source, source.type)) continue;
    if (!matchType(rule.target, target.type)) continue;
    for (const t of rule.suggest) {
      if (!seen.has(t)) {
        seen.add(t);
        out.push(t);
      }
    }
  }

  const fallback: RelationshipType[] = ["depends_on", "calls", "connects_to"];
  for (const t of fallback) {
    if (!seen.has(t)) {
      seen.add(t);
      out.push(t);
    }
  }
  return out;
}

/** @deprecated use suggestRelationshipTypes()[0] */
export function inferRelationshipType(
  source: ArchNode,
  target: ArchNode,
): RelationshipType {
  return suggestRelationshipTypes(source, target)[0] ?? "calls";
}
