import { For, type JSX } from "solid-js";
import {
  Bot,
  ConnectIcon,
  FolderKanban,
  Inbox,
  Layers,
  Scale,
  SchemeIcon,
  Settings,
} from "@polterware/frontend-design-system";
import {
  capabilityEnabled,
  type PlatformContext,
} from "@polterware/frontend-platform";
import type { NavItem } from "./route";
import "./sidebar.css";

/**
 * Sidebar glyphs — Hugeicons outline set tuned for operator IA.
 * Sidebar glyphs stay within the curated Hugeicons set.
 */
const icons = {
  inbox: (p) => <Inbox size={p.size ?? 14} class={p.class} />,
  atlas: (p) => <SchemeIcon size={p.size ?? 14} class={p.class} />,
  work: (p) => <FolderKanban size={p.size ?? 14} class={p.class} />,
  decisions: (p) => <Scale size={p.size ?? 14} class={p.class} />,
  reality: (p) => <Layers size={p.size ?? 14} class={p.class} />,
  agents: (p) => <Bot size={p.size ?? 14} class={p.class} />,
  integrations: (p) => <ConnectIcon size={p.size ?? 14} class={p.class} />,
  settings: (p) => <Settings size={p.size ?? 14} class={p.class} />,
} satisfies Record<string, (p: { size?: number; class?: string }) => JSX.Element>;

export function SidebarNav(props: {
  items: NavItem[];
  activeId: string;
  onSelect: (id: string) => void;
  platform?: PlatformContext;
}) {
  const groups = () => {
    const map = new Map<string, NavItem[]>();
    for (const item of props.items.filter((candidate) =>
      capabilityEnabled(props.platform, candidate.requiredCapability),
    )) {
      const g = item.group ?? "GENERAL";
      if (!map.has(g)) map.set(g, []);
      const arr = map.get(g);
      if (arr) arr.push(item);
    }
    return [...map.entries()];
  };

  return (
    <nav class="pw-sidebar-nav" aria-label="Primary">
      <For each={groups()}>
        {([group, items]) => (
          <div class="pw-sidebar-group">
            <p class="pw-sidebar-group-label">{group}</p>
            <For each={items}>
              {(item) => {
                const active = () => props.activeId === item.id;
                const renderIcon = icons[item.id] ?? icons.atlas!;
                return (
                  <button
                    type="button"
                    aria-current={active() ? "page" : undefined}
                    classList={{
                      "pw-sidebar-item": true,
                      "pw-sidebar-item--active": active(),
                    }}
                    onClick={() => props.onSelect(item.id)}
                  >
                    <span class="pw-sidebar-item-icon">{renderIcon({ size: 14 })}</span>
                    <span class="pw-sidebar-item-label">{item.label}</span>
                  </button>
                );
              }}
            </For>
          </div>
        )}
      </For>
    </nav>
  );
}

/** Fixed footer Settings control for project sidebar. */
export function SidebarSettingsItem(props: {
  active?: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      aria-current={props.active ? "page" : undefined}
      classList={{
        "pw-sidebar-item": true,
        "pw-sidebar-item--active": Boolean(props.active),
        "pw-sidebar-item--footer": true,
      }}
      onClick={() => props.onSelect()}
    >
      <span class="pw-sidebar-item-icon">{icons.settings!({ size: 14 })}</span>
      <span class="pw-sidebar-item-label">Settings</span>
    </button>
  );
}
