# Contract: Resources UI

**Owner**: `packages/features/resources`  
**Data wiring**: Desktop App via `@polterware/frontend-data` + fixtures

## Views

### ResourcesIndexView

**Inputs**

| Prop | Type |
| --- | --- |
| resources | Resource[] |
| loading | boolean |
| error | string \| null |
| environmentId | string \| null |
| environments | EnvironmentOption[] |
| search | string |
| filter | string (e.g. all \| cloud \| local \| nodes) |
| dataSource | `core` \| `fixture` \| `empty` |
| onSearch | (q: string) => void |
| onFilter | (f: string) => void |
| onEnvironment | (id: string) => void |
| onOpen | (id: string) => void |
| onCreate | () => void |
| onRetry | () => void |

**Outputs / UI duties**

- Dense table/list with columns: Name, Type, Environment, Provider, Status, Last sync  
- Empty / loading / error states  
- No permanent detail inspector  

### ResourceDetailView

**Inputs**: `resource: Resource | null`, loading, error, onBack, onEdit, onQuickAction  

**Sections**: identity header, status chips, overview, linked architecture, activity, notes, quick actions  

### ResourceFormDialog

**Inputs**: open, mode create|edit, draft, error, submitting, onChange, onSubmit, onClose  

## Core mapping

| UI field | Core source |
| --- | --- |
| id, name, type | listResources |
| source, externalKey, systemId | listResources |
| environment, provider, status, lastSync | enrichment or fixture until core fields exist |

If core lacks display fields, view model fills defaults (`unknown`, `healthy` only when justified) without claiming live provider health falsely—use `unknown` when not observed.
)
