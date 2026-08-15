# Contract: Architecture UI

**Owner**: `packages/features/architecture` + `@polterware/frontend-canvas`  
**Data wiring**: layout + resources/systems from core; fixtures fallback

## Views

### ArchitectureCanvasView

**Inputs**

| Prop | Type |
| --- | --- |
| model | CanvasModel |
| layers | optional layer metadata |
| loading | boolean |
| error | string \| null |
| environmentId | string \| null |
| search | string |
| selectedNodeId | string \| null |
| onSelect | (id: string \| null) => void |
| onOpenNode | (id: string) => void |
| onSaveLayout | (bodyJson: string, revision: number) => void |
| onPanZoom? | optional |
| onRetry | () => void |

**UI duties**

- Full-width canvas inside MainSurface (no permanent right inspector)  
- Neutral selection chrome  
- Evidence-aware edge styling  
- Empty / loading / error  
- Double-click or explicit “Open” → detail page via `onOpenNode`  

### ArchitectureNodeDetailView

**Inputs**: node view model, loading, error, onBack, onOpenRelated, onQuickAction  

**Sections**: identity, overview, connected architecture, relationship mini-view, activity, notes, docs, quick actions  

## Canvas model contract

Reuse `@polterware/frontend-canvas` serialize/deserialize.  
Layout RPC: existing `getLayout` / `updateLayout` (revision-checked).

## Forbidden

- Brand-green selection glow  
- Permanent inspector column as default  
- Claiming non-evidence edges as live runtime wiring  
)
