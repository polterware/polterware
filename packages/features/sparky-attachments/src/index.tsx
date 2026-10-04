import type { SparkyFeature } from "@polterware/feature-sparky-shell/feature";
import { AttachmentsSection } from "./section";

const feature: SparkyFeature = {
  id: "attachments",
  routes: [],
  detailSections: [{ id: "attachments", order: 30, component: AttachmentsSection }],
};

export default feature;
export { AttachmentsSection };
export { createAttachmentsBridge, browserBridge } from "./bridge";
export { createAttachmentsStore, attachmentsStore } from "./store";
export { previewUrl, droppedFilePath, revisionString } from "./model";
