# Contract: Resource Link Port

**Owner**: architecture feature domain + thin adapter in ArchitectureView  
**Consumers**: Configure dialog, Detail “Link resource”, Connected “Link from Resources”

---

## LinkableResource

```ts
type LinkableResource = {
  id: string;
  name: string;
  type: string;
  environmentId: string;
  environmentLabel?: string;
  provider?: string;
  dataSource: "fixture" | "core";
};
```

---

## ResourceLinkPort

```ts
type ListLinkCandidatesQuery = {
  projectId?: string;
  environmentId?: string;
  componentType?: string;  // NodeType / semantic type
  subtype?: string;
  query?: string;          // name search
  typeFilterStrict?: boolean; // default true
};

interface ResourceLinkPort {
  listCandidates(q: ListLinkCandidatesQuery): LinkableResource[] | Promise<LinkableResource[]>;
  getById?(id: string): LinkableResource | undefined;
}
```

### Filtering rules (v1)

1. If `environmentId` set: include resources whose normalized env matches; exclude hard mismatches.  
2. If `typeFilterStrict`: include only type-family matches (see map below).  
3. `query`: case-insensitive substring on name.  
4. Never invent rows; empty array is valid.

### Type family map (illustrative)

| componentType / category | resource.type examples |
| --- | --- |
| database / data postgres… | Database |
| cache | Cache, Redis |
| queue | Queue |
| stream | Stream, Kafka |
| api, service, worker, web | Service, API |
| storage | Bucket, Storage |
| host | VPS, Host, Node |

Unmapped types: soft-list all in env when strict filter yields empty (UI may show “No exact type match — showing environment resources”).

---

## validateLink(componentEnvId, resource)

| Condition | Result |
| --- | --- |
| resource missing | error |
| env mismatch (both set, different) | error “Resource environment does not match” |
| ok | `{ resourceId }` |

---

## applyLinkToNode (conceptual mutation)

```ts
updateNode(doc, nodeId, {
  resourceId,
  linkStatus: "linked",
  state: "linked",
})
// MUST NOT set observed / invent health
```

## applyUnlink (optional v1)

```ts
updateNode(doc, nodeId, {
  resourceId: null,
  linkStatus: "unlinked",
  state: "planned",
})
```

---

## Fixture adapter

- Map `RESOURCE_FIXTURES` / `RESOURCE_MOCKS` → `LinkableResource`  
- Normalize `Production` → `production`  
- Mark `dataSource: "fixture"`  
- UI may show subtle fixture context when shell fixture mode is on  

---

## Honesty

- Listing fixture resources MUST NOT claim live sync success beyond existing fixture chrome.  
- Link action labels: “Link resource”, never “Import live from AWS” unless real.  
