import type { SparkyFeature } from "@polterware/feature-sparky-shell/feature";
import attachments from "@polterware/feature-sparky-attachments";
import calendar from "@polterware/feature-sparky-calendar";
import focus from "@polterware/feature-sparky-focus";
import map from "@polterware/feature-sparky-map";
import memories from "@polterware/feature-sparky-memories";
import minds from "@polterware/feature-sparky-minds";
import review from "@polterware/feature-sparky-review";
import schedule from "@polterware/feature-sparky-schedule";
import search from "@polterware/feature-sparky-search";
import settings from "@polterware/feature-sparky-settings";
import shell from "@polterware/feature-sparky-shell";

export const features: SparkyFeature[] = [
  shell,
  memories,
  minds,
  schedule,
  calendar,
  attachments,
  focus,
  review,
  map,
  settings,
  search,
];
