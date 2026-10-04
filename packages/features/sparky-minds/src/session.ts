import { createSignal } from "solid-js";

const [revision, setRevision] = createSignal(0);

export const mindsRevision = revision;

export function bumpMinds(): void {
  setRevision((value) => value + 1);
}
