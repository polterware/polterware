import { A11yDialog, type ComposerProperty } from "@polterware/frontend-design-system";
import type { PlatformContext } from "@polterware/frontend-platform";
import { PaletteBody } from "./palette-body";
import type { PaletteItem } from "./palette-items";
import "./commander.css";

/** In-app Commander: the Composer's frame (its width and Close), so a run reads exactly like Start run; overview cards stay global-only. */
export function CommandPalette(props: {
  open: boolean;
  onClose: () => void;
  onCommand?: (id: string) => void;
  items?: PaletteItem[];
  platform?: PlatformContext;
  /** Route, Milestone, and Card for Run an agent. */
  runProperties?: readonly ComposerProperty[];
  /** Off where a query that matches no command should not read as an agent run. */
  runAgent?: boolean;
  /** Off for an app with no search behind Show all results. */
  searchAll?: boolean;
  placeholder?: string;
}) {
  return (
    <A11yDialog
      open={props.open}
      onClose={props.onClose}
      title="Command palette"
      chrome="flush"
      size="lg"
      class="commander-panel"
      bodyClass="px-0 pb-0"
    >
      <PaletteBody
        items={props.items}
        platform={props.platform}
        onCommand={props.onCommand}
        onConsumed={props.onClose}
        runAgent={props.runAgent ?? true}
        runProperties={props.runProperties}
        searchAll={props.searchAll}
        placeholder={props.placeholder}
      />
    </A11yDialog>
  );
}
