# Contract: UI Chrome

**Visual source of truth**: `DESIGN.md`  
**Icons**: Hugeicons Free Stroke Rounded individual imports
**No permanent lateral inspector**

---

## 1. Add to architecture dialog

| Property | Value |
| --- | --- |
| Title | `Add to architecture` |
| Pattern | Searchable dialog (A11yDialog or equivalent) |
| Sections | Recents · Favorites · Categories (all types) |
| Row content | Icon, name, description, category, recent indicator |
| Input | Search field focused on open |
| Keyboard | Arrow keys, Enter select, Escape close |
| Virtualization | When list large (TanStack Virtual) |

Connected-resource rows: action handlers (link/import) may show toast “Not available yet” if stubbed — honest, not fake success.

---

## 2. Component card (normal zoom)

Compact structure (not strong multi-color types):

```text
[icon] Name
       Type label
       Env · Provider     (when zoom/data allow)
       State chip         (planned/linked/…)
```

LOD:

| Zoom | Show |
| --- | --- |
| < 0.45 | Name only (or icon+name) |
| mid | Type + name |
| > ~0.85 | State + env/provider |

Selection: existing surface-selected border tokens.

---

## 3. Relationship chrome

- Orthogonal path retained
- Arrow direction clear
- Type label (or short label) when zoom allows and label empty → show type human name
- Invisible wider stroke for hit-testing (`pointer-events` on hit path)
- Selected: brighter stroke

---

## 4. Contextual toolbars

### Single node

```text
Open details | Rename | Connect | Duplicate | More ▾
```

More: Change type, Copy, Cut, Move to boundary, Bring forward, Send backward, Highlight dependencies, Export selection, Delete

### Relationship

```text
{Type} ▾ | Edit label | Reverse | More ▾
```

More: Protocol, Reconnect source/target, Duplicate, Delete, Highlight ends, Open related

### Multi

```text
Duplicate | Align ▾ | Distribute ▾ | Group | Layout | More ▾
```

Position: floating near selection AABB top/center; must not permanently cover ports when possible.

---

## 5. Context menus

### Empty

Add component here · Add boundary here · Add text note · Paste · Select all · Fit architecture · Toggle grid

### Node

Open details · Rename · Connect · Duplicate · Copy · Cut · Delete · Highlight dependencies

### Relationship

Change type · Edit label · Reverse · Delete · Highlight ends

### Multi

Duplicate · Delete · Create boundary · Align · Distribute · Layout

---

## 6. Page header / filters

| Control | Contract |
| --- | --- |
| Environment | Accessible name **Environment**; options All environments + named envs (not bare unlabeled “All” alone) |
| Search | Placeholder clarifies architecture search; on canvas, Enter/result centers |
| Save | Not primary CTA; optional secondary “Save now” or none |
| Persistence | Visible Saving / Saved / Unsaved / Save failed + Retry |

---

## 7. Bottom chrome

| Control | Contract |
| --- | --- |
| Zoom − / % / + | Keep |
| Fit | Keep |
| Open | Label **Open details** when selection has node; disabled otherwise |

---

## 8. Minimap

- Real node positions scaled to document bounds
- Viewport rectangle accurate
- Click navigates
- Optional P2: drag viewport rect; show boundaries faintly

---

## 9. Connect quick mode indicator

Toolbar active state:

```text
Connect: Calls ▾
```

Clear visual difference from plain Connect without type.
