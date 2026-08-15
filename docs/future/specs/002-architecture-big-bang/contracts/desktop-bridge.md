# Contract: Desktop Bridge

Thin typed façade over `polterware.local.v1`. Not a second domain API.

## Security

contextIsolation true · nodeIntegration false · sandbox when compatible · allowlisted methods only · no raw generic IPC · no permanent token in renderer storage

## Namespaces (illustrative)

```text
window.polterware = {
  health, identity, projects, architecture, knowledge,
  operations, processes, workflows, integrations,
  approvals, audit, logs,
  app: { getInfo, listIcons, getIcon, setIcon },  // Main-native
  window: { hideCommander, showMain },           // Main-native
  capabilities: { terminal: true, cloud: false, updater: 'main-only' }
}
```

**Removed from product bridge:** `skills.preview`, `skills.setup` (disposition REMOVE).

## Subscriptions

Connect streams; unsubscribe cancels; resync after core restart.

## Mapping from old IPC

commands/processes/pipelines/declarative/repos/config → local services  
mcp → MCP runtime product (not skill helpers)  
skills → **removed**  
app icons / commander window → Main-native only  
