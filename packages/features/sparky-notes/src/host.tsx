import { Show } from "solid-js";
import type { DetailSectionDef } from "@polterware/feature-sparky-shell/feature";
import { NoteDetail, NoteProperties } from "./note-detail";
import { trySparkyShell } from "./shell-access";
import type { NotesStore } from "./store";

export function NoteDetailHost(props: { noteId?: string; sections?: DetailSectionDef[]; store?: NotesStore }) {
  const shell = trySparkyShell();
  const id = () => props.noteId || shell?.selectedNoteId() || shell?.params().id || "";
  return <Show when={id()}>{value => <NoteDetail noteId={value()} sections={props.sections ?? shell?.detailSections() ?? []} store={props.store} />}</Show>;
}

export function NotePropertiesHost(props: { noteId?: string; sections?: DetailSectionDef[]; store?: NotesStore }) {
  const shell = trySparkyShell();
  const id = () => props.noteId || shell?.selectedNoteId() || "";
  return <Show when={id()}>{value => <NoteProperties noteId={value()} sections={props.sections ?? shell?.detailSections() ?? []} store={props.store} />}</Show>;
}
