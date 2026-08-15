import { createLocalStorageTypePrefs } from "./canvas/type-prefs-local";
import { AddToArchitectureDialog } from "./canvas/add-to-architecture-dialog";
import type { ComponentTypeDefinition } from "./domain/registry";

export type ArchNodeDraft = {
  name: string;
  type: string;
  environment: string;
  owner: string;
  description: string;
  linkedResourceId: string;
  /** Stable registry type id when created via catalog */
  typeId?: string;
  /** Product preset subtype (e.g. postgres). */
  subtype?: string;
  provider?: string;
};

export function emptyArchNodeDraft(): ArchNodeDraft {
  return {
    name: "",
    type: "Service",
    environment: "Production",
    owner: "Platform",
    description: "",
    linkedResourceId: "",
  };
}

const prefs = createLocalStorageTypePrefs();

/**
 * Catalog entry for Components tab — same discover surface as canvas.
 * Submits a draft with typeId; parent places with planned defaults.
 * Prefer Architecture canvas host configure flow for full component details.
 */
export function CreateNodeDialog(props: {
  open: boolean;
  onClose: () => void;
  onSubmit?: (draft: ArchNodeDraft) => void;
  submitting?: boolean;
  error?: string | null;
}) {
  function onSelect(def: ComponentTypeDefinition) {
    if (!def.placeable || def.placementKind !== "node") {
      props.onClose();
      return;
    }
    props.onSubmit?.({
      name: def.label,
      type: def.label,
      typeId: String(def.nodeType ?? def.id),
      subtype: def.subtype,
      provider: def.provider,
      environment: "production",
      owner: "",
      // Do not copy catalog blurb as instance purpose
      description: "",
      linkedResourceId: "",
    });
    props.onClose();
  }

  return (
    <AddToArchitectureDialog
      open={props.open}
      prefs={prefs}
      onClose={() => props.onClose()}
      onSelect={onSelect}
      onUnavailableAction={() => props.onClose()}
    />
  );
}
