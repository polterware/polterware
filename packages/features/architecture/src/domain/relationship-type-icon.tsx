import { Dynamic } from "solid-js/web";
import type { RelationshipType } from "@polterware/frontend-canvas";
import {
  Activity,
  Box,
  Cloud,
  Copy,
  Database,
  Download,
  ExternalLink,
  FileUp,
  GitBranch,
  Inbox,
  KeyRound,
  Link,
  Package,
  Pencil,
  Plug,
  RefreshCw,
  Route,
  Server,
  Webhook,
  Zap,
  type HugeiconComponent,
} from "@polterware/frontend-design-system";
import {
  hasRelationshipTypeIcon,
  relationshipTypeIconKey,
} from "./relationship-type-icon-keys";

export {
  hasRelationshipTypeIcon,
  relationshipTypeIconKey,
  RELATIONSHIP_TYPE_ICON_KEYS,
} from "./relationship-type-icon-keys";

const ICON_BY_KEY: Record<string, HugeiconComponent> = {
  "git-branch": GitBranch,
  webhook: Webhook,
  download: Download,
  "file-up": FileUp,
  zap: Zap,
  inbox: Inbox,
  route: Route,
  cloud: Cloud,
  plug: Plug,
  "key-round": KeyRound,
  package: Package,
  database: Database,
  activity: Activity,
  server: Server,
  copy: Copy,
  "refresh-cw": RefreshCw,
  "external-link": ExternalLink,
  box: Box,
  link: Link,
  pencil: Pencil,
};

export function relationshipTypeIcon(
  type: RelationshipType | string,
): HugeiconComponent {
  return ICON_BY_KEY[relationshipTypeIconKey(type)] ?? Link;
}

export function RelationshipTypeIcon(props: {
  type: RelationshipType | string;
  size?: number;
  class?: string;
}) {
  return (
    <Dynamic
      component={relationshipTypeIcon(props.type)}
      size={props.size ?? 14}
      class={props.class}
      aria-hidden
    />
  );
}
