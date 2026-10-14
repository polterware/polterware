import type { SparkyFeature } from "@polterware/feature-sparky-shell/feature";
import attachments from "@polterware/feature-sparky-attachments";
import calendar from "@polterware/feature-sparky-calendar";
import focus from "@polterware/feature-sparky-focus";
import map from "@polterware/feature-sparky-map";
import notes from "@polterware/feature-sparky-notes";
import spaces from "@polterware/feature-sparky-spaces";
import schedule from "@polterware/feature-sparky-schedule";
import search from "@polterware/feature-sparky-search";
import settings from "@polterware/feature-sparky-settings";
import shell from "@polterware/feature-sparky-shell";

export const features: SparkyFeature[] = [
  shell,
  notes,
  spaces,
  schedule,
  calendar,
  attachments,
  focus,
  map,
  settings,
  search,
];
