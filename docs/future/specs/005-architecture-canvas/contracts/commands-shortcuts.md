# Contract: Commands & Shortcuts

**Owner**: canvas engine command registry + architecture command contributions + App palette

## Principles

1. One command id → one implementation.  
2. Toolbar, context menu, shortcuts, and command palette **dispatch the same ids**.  
3. Shortcuts active only when canvas focused and not text-editing.  
4. Shell retains `⌘/Ctrl+K` palette and global commander shortcut.

## Core command ids (minimum)

| Id | Default shortcut | Notes |
| --- | --- | --- |
| `tool.select` | V | |
| `tool.hand` | H | Space = temporary |
| `tool.connect` | C | |
| `tool.add_node` | N | |
| `tool.add_boundary` | B | P2 |
| `tool.add_note` | T | |
| `history.undo` | ⌘/Ctrl+Z | |
| `history.redo` | ⌘/Ctrl+Shift+Z | |
| `edit.copy` | ⌘/Ctrl+C | |
| `edit.cut` | ⌘/Ctrl+X | |
| `edit.paste` | ⌘/Ctrl+V | |
| `edit.duplicate` | ⌘/Ctrl+D | |
| `edit.delete` | Delete / Backspace | |
| `select.all` | ⌘/Ctrl+A | |
| `view.fit_document` | Shift+1 | |
| `view.fit_selection` | Shift+2 | |
| `view.zoom_in` | + / = | |
| `view.zoom_out` | - | |
| `view.zoom_reset` | 0 | |
| `view.search` | ⌘/Ctrl+F | canvas find |
| `node.open` | Enter | when node selected |
| `app.command_palette` | ⌘/Ctrl+K | owned by App |

## Palette integration

Architecture contributes items (Add node, Fit document, …) to commander list without reimplementing handlers—call `dispatch(id)`.

## Conflicts

See research R7. Canvas must not steal palette shortcut.  
