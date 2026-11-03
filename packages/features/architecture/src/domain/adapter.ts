import type { DomainAdapter } from "@polterware/frontend-canvas";
import { defaultNodeSize, defaultPorts } from "./registry";
import { canConnect } from "./connection-rules";

export function createArchitectureDomainAdapter(): DomainAdapter {
  return {
    defaultPorts,
    defaultNodeSize,
    canConnect,
  };
}
