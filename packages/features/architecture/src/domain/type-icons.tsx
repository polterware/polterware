import type { JSX } from "solid-js";
import {
  Activity,
  Archive,
  Box,
  Boxes,
  Clock,
  Cloud,
  CloudCog,
  Cog,
  CreditCard,
  Database,
  Download,
  ExternalLink,
  FileUp,
  GitBranch,
  GitMerge,
  Globe,
  Globe2,
  Hammer,
  HardDrive,
  LayoutDashboard,
  Link,
  Link2,
  ListOrdered,
  MapPin,
  MemoryStick,
  Monitor,
  Network,
  Package,
  Plug,
  Route,
  Scale,
  Search,
  Server,
  Shield,
  Smartphone,
  Square,
  StickyNote,
  Warehouse,
  Webhook,
  Zap,
  type HugeiconComponent,
} from "@polterware/frontend-design-system";
import { hasTypeIcon, type TypeIconKey } from "./type-icon-keys";

export { hasTypeIcon, type TypeIconKey } from "./type-icon-keys";

/** Registry icon key (kebab-case domain id) → Hugeicons component. */
const ICON_MAP: Record<TypeIconKey, HugeiconComponent> = {
  globe: Globe,
  smartphone: Smartphone,
  monitor: Monitor,
  webhook: Webhook,
  boxes: Boxes,
  cog: Cog,
  clock: Clock,
  zap: Zap,
  database: Database,
  "memory-stick": MemoryStick,
  "list-ordered": ListOrdered,
  activity: Activity,
  "hard-drive": HardDrive,
  search: Search,
  warehouse: Warehouse,
  server: Server,
  scale: Scale,
  route: Route,
  cloud: Cloud,
  network: Network,
  "globe-2": Globe2,
  "map-pin": MapPin,
  "git-branch": GitBranch,
  package: Package,
  "git-merge": GitMerge,
  hammer: Hammer,
  archive: Archive,
  "external-link": ExternalLink,
  "cloud-cog": CloudCog,
  plug: Plug,
  shield: Shield,
  "credit-card": CreditCard,
  link: Link,
  "layout-dashboard": LayoutDashboard,
  square: Square,
  "sticky-note": StickyNote,
  box: Box,
  "link-2": Link2,
  download: Download,
  "file-up": FileUp,
};

/** Hugeicons icon for a registry `icon` key. Fallback: Box. */
export function TypeIcon(props: {
  name: string;
  size?: number;
  class?: string;
}): JSX.Element {
  const Icon =
    hasTypeIcon(props.name) ? ICON_MAP[props.name as TypeIconKey] : Box;
  return <Icon size={props.size ?? 16} class={props.class} />;
}
