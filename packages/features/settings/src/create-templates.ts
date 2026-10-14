import type { IconTileTone } from "@polterware/frontend-design-system";
import { defaultProtectionForKind } from "./environment-status";
import type { EnvironmentKind, ProtectionLevel } from "./types";

export type EnvironmentCreateTemplate = {
  id: string;
  title: string;
  description: string;
  tone: IconTileTone;
  draft: {
    name: string;
    kind: EnvironmentKind;
    protection: ProtectionLevel;
  };
};

function environmentTemplate(
  kind: EnvironmentKind,
  description: string,
  tone: IconTileTone,
): EnvironmentCreateTemplate {
  const title = kind.charAt(0).toUpperCase() + kind.slice(1);
  return {
    id: `env-${kind}`,
    title,
    description,
    tone,
    draft: {
      name: title,
      kind,
      protection: defaultProtectionForKind(kind),
    },
  };
}

export const ENVIRONMENT_CREATE_TEMPLATES: EnvironmentCreateTemplate[] = [
  environmentTemplate("development", "Relaxed local development environment.", "execution"),
  environmentTemplate("staging", "Standard pre-production environment.", "infrastructure"),
  environmentTemplate("production", "Strict production environment.", "security"),
];
