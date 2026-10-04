import { beforeEach, describe, expect, it, vi } from "vitest";
import { createAttachmentsBridge, type AttachmentsWireBridge } from "./bridge";
import { fakeBridge, MEMORY_ID, PHOTO_ID, attachmentFixture, AUDIO_ID, FILE_ID, LINK_ID } from "./fake-bridge";
import { DROP_PATH_UNAVAILABLE, MICROPHONE_DENIED, RECORDING_IMPORT_UNAVAILABLE } from "./model";
import { createAttachmentsStore } from "./store";

beforeEach(() => {
  class FakeRecorder { static isTypeSupported() { return true; } }
  vi.stubGlobal("MediaRecorder", FakeRecorder);
  Object.defineProperty(navigator, "mediaDevices", { configurable: true, value: { getUserMedia: vi.fn() } });
});

function storeFrom(fake = fakeBridge()) {
  return { ...fake, store: createAttachmentsStore(() => createAttachmentsBridge(fake.bridge as unknown as AttachmentsWireBridge)) };
}

describe("attachments store", () => {
  it("imports twenty mixed files with expectedRev and keeps core size/quota messages", async () => {
    const { store, bridge } = storeFrom(fakeBridge([]));
    const paths = Array.from({ length: 18 }, (_, index) => `/tmp/file-${index}.pdf`).concat(["/tmp/photo.png", "/tmp/clip.webm"]);
    vi.mocked(bridge.app.pickFiles).mockResolvedValueOnce(paths);
    await store.pickAndImport(MEMORY_ID);
    expect(bridge.attachments.importFromPath).toHaveBeenCalledTimes(20);
    expect(vi.mocked(bridge.attachments.importFromPath).mock.calls[0]![0]).toMatchObject({ expectedRev: "1" });
    expect(store.items()).toHaveLength(20);
    expect(store.state()).toBe("live");
    await store.importPaths(MEMORY_ID, ["/tmp/oversize.png", "/tmp/quota.bin"]);
    expect(store.state()).toBe("failed");
    expect(store.error()).toContain("attachment exceeds kind size limit");
    expect(store.error()).toContain("attachment quota exceeded");
  });
  it("adds links with expectedRev and surfaces microphone denied plus recording import unavailability", async () => {
    const { store, bridge } = storeFrom(fakeBridge([]));
    await store.addLink(MEMORY_ID, "https://example.com/page", "Example");
    expect(bridge.attachments.addLink).toHaveBeenCalledWith(expect.objectContaining({ expectedRev: "1", url: "https://example.com/page" }));
    expect(store.items()[0]?.kind).toBe("link");
    vi.mocked(bridge.app.startMicrophone).mockResolvedValueOnce(false);
    expect(await store.requestMicrophone()).toBe("denied");
    expect(store.microphoneCopy("denied")).toBe(MICROPHONE_DENIED);
    store.recordingImportUnavailable();
    expect(store.error()).toBe(RECORDING_IMPORT_UNAVAILABLE);
  });
  it("deletes, reorders, reveals through sparky-file URLs, and refuses pathless drops", async () => {
    const seed = [
      attachmentFixture(),
      attachmentFixture({ id: AUDIO_ID, kind: "audio", name: "clip.webm", mime: "audio/webm", relPath: `${MEMORY_ID}/${AUDIO_ID}.webm`, sortOrder: 1 }),
      attachmentFixture({ id: FILE_ID, kind: "file", name: "notes.pdf", mime: "application/pdf", relPath: `${MEMORY_ID}/${FILE_ID}.pdf`, sortOrder: 2 }),
      attachmentFixture({ id: LINK_ID, kind: "link", url: "https://example.com", title: "Example", relPath: undefined, sortOrder: 3 }),
    ];
    const { store, bridge } = storeFrom(fakeBridge(seed));
    await store.load(MEMORY_ID);
    await store.move(MEMORY_ID, 0, 1);
    expect(bridge.attachments.reorder).toHaveBeenCalledWith({ memoryId: MEMORY_ID, orderedIds: [AUDIO_ID, PHOTO_ID, FILE_ID, LINK_ID] });
    await store.reveal(store.items()[0]!);
    expect(bridge.app.revealInFinder).toHaveBeenCalledWith(`sparky-file://attachment/${MEMORY_ID}/${AUDIO_ID}.webm`);
    await store.remove(MEMORY_ID, FILE_ID);
    expect(bridge.attachments.delete).toHaveBeenCalledWith({ id: FILE_ID });
    store.importDropped(MEMORY_ID, [{ name: "drop.png" } as File]);
    expect(store.error()).toBe(DROP_PATH_UNAVAILABLE);
    expect(bridge.attachments.importFromPath).not.toHaveBeenCalled();
  });
});
