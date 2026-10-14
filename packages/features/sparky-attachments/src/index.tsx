import type { SparkyFeature } from "@polterware/feature-sparky-shell/feature";
import { AttachmentsSection } from "./section";
import { noteAttachments } from "./embed";
import "./embed.css";

const feature: SparkyFeature = {
  id: "attachments",
  routes: [],
  noteAttachments,
};

export default feature;
export { AttachmentsSection };
export { createAttachmentsBridge, browserBridge } from "./bridge";
export { createAttachmentsStore, attachmentsStore } from "./store";
export { previewUrl, recordingFileName, revisionString } from "./model";
