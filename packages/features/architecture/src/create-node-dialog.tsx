import { createLocalStorageTypePrefs } from "./canvas/type-prefs-local";
import { AddToArchitectureDialog } from "./canvas/add-to-architecture-dialog";
import { createFixtureResourceLinkPort } from "./domain/fixture-resource-link-port";
import { suggestPlaceFromResource, type LinkableResource } from "./domain/place-link";
import type { ComponentTypeDefinition } from "./domain/registry";
import { getComponentType, listCatalogTypes } from "./domain/registry";

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
const resourcePort = createFixtureResourceLinkPort();

/**
 * Catalog entry for placing a component on the Architecture canvas.
 * Submits a draft with typeId; parent places with planned defaults.
 */
export function CreateNodeDialog(props: {
  open: boolean;
  onClose: () => void;
  onSubmit?: (draft: ArchNodeDraft) => void;
  onAddResource?: () => void;
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

  function onSelectResource(resource: LinkableResource) {
    const suggestion = suggestPlaceFromResource(resource);
    const typeKey = suggestion.suggestedComponentType ?? "service";
    const def =
      listCatalogTypes().find(
        (t) =>
          t.placeable &&
          t.placementKind === "node" &&
          (String(t.nodeType) === typeKey || t.id === typeKey || t.subtype === typeKey),
      ) ??
      getComponentType(typeKey) ??
      listCatalogTypes().find((t) => t.placeable && t.placementKind === "node");
    if (!def || !def.placeable) {
      props.onClose();
      return;
    }
    props.onSubmit?.({
      name: suggestion.name,
      type: def.label,
      typeId: String(def.nodeType ?? def.id),
      subtype: def.subtype,
      provider: def.provider,
      environment: suggestion.environmentId,
      owner: "",
      description: "",
      linkedResourceId: suggestion.resourceId,
    });
    props.onClose();
  }

  return (
    <AddToArchitectureDialog
      open={props.open}
      prefs={prefs}
      resources={resourcePort.listCandidates({ typeFilterStrict: false })}
      onClose={() => props.onClose()}
      onSelect={onSelect}
      onSelectResource={onSelectResource}
      onAddResource={props.onAddResource}
      onUnavailableAction={() => props.onClose()}
    />
  );
}
