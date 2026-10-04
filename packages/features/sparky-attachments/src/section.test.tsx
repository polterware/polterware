import { render } from "solid-js/web";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { DesignSystemProvider } from "@polterware/frontend-design-system";
import { AUDIO_ID, FILE_ID, LINK_ID, MEMORY_ID, attachmentFixture, fakeBridge } from "./fake-bridge";
import { DROP_PATH_UNAVAILABLE, FETCH_TITLES_OFF, MICROPHONE_DENIED } from "./model";
import { attachmentsStore } from "./store";
import { AttachmentsSection } from "./section";

vi.mock("@polterware/feature-sparky-shell/feature", () => ({
  useSparkyShell: () => ({
    selectedMemoryId: () => MEMORY_ID,
    features: [],
    path: () => "/",
    params: () => ({}),
    searchParams: () => new URLSearchParams(),
    navigate: () => {},
    openMemory: () => {},
    closeDetail: () => {},
    registerMemoryDetail: () => () => {},
    registerCollectionShortcuts: () => () => {},
    deepLink: () => null,
    detailSections: () => [],
    settingsPanels: () => [],
  }),
}));

let dispose: (() => void) | undefined;
let fake = fakeBridge();

beforeEach(() => {
  fake = fakeBridge([
    attachmentFixture(),
    attachmentFixture({ id: AUDIO_ID, kind: "audio", name: "clip.webm", mime: "audio/webm", relPath: `${MEMORY_ID}/${AUDIO_ID}.webm`, sortOrder: 1 }),
    attachmentFixture({ id: FILE_ID, kind: "file", name: "notes.pdf", mime: "application/pdf", relPath: `${MEMORY_ID}/${FILE_ID}.pdf`, sortOrder: 2 }),
    attachmentFixture({ id: LINK_ID, kind: "link", url: "https://example.com/path", title: "Example", relPath: undefined, sortOrder: 3 }),
  ]);
  Reflect.set(globalThis, "sparky", fake.bridge);
  attachmentsStore.dispose();
  class FakeRecorder { static isTypeSupported() { return true; } }
  vi.stubGlobal("MediaRecorder", FakeRecorder);
  Object.defineProperty(navigator, "mediaDevices", { configurable: true, value: { getUserMedia: vi.fn() } });
  vi.spyOn(HTMLElement.prototype, "getClientRects").mockImplementation(function (this: HTMLElement) {
    return this.isConnected && !this.closest("[hidden]") ? [new DOMRect(0, 0, 100, 32)] as unknown as DOMRectList : [] as unknown as DOMRectList;
  });
});
afterEach(() => {
  dispose?.();
  dispose = undefined;
  attachmentsStore.dispose();
  document.body.replaceChildren();
  vi.restoreAllMocks();
});

function mount() {
  const host = document.createElement("div");
  document.body.append(host);
  dispose = render(() => <DesignSystemProvider storagePrefix="sparky-attachments-test"><AttachmentsSection /></DesignSystemProvider>, host);
  return host;
}
async function ready(host: HTMLElement) {
  await vi.waitFor(() => expect(host.querySelector("[data-attachment-id]")).not.toBeNull());
}

describe("attachments section", () => {
  it("registers the detail section and previews only through sparky-file", async () => {
    const feature = (await import("./index")).default;
    expect(feature.detailSections?.[0]).toMatchObject({ id: "attachments", order: 30 });
    const host = mount();
    await ready(host);
    const urls = [...host.querySelectorAll("[data-preview-url]")].map(element => element.getAttribute("data-preview-url"));
    expect(urls.length).toBeGreaterThan(0);
    expect(urls.every(url => url?.startsWith("sparky-file://attachment/"))).toBe(true);
    expect(urls.some(url => url?.startsWith("file:"))).toBe(false);
    expect(host.textContent).toContain("example.com");
    expect(host.querySelector("img[src^='file:']")).toBeNull();
    expect(host.querySelector("audio[src^='file:']")).toBeNull();
  });
  it("imports from Add file, shows core errors, and keeps denied microphone copy", async () => {
    vi.mocked(fake.bridge.app.pickFiles).mockResolvedValueOnce(["/tmp/oversize.png"]);
    vi.mocked(fake.bridge.app.startMicrophone).mockResolvedValue(false);
    const host = mount();
    await ready(host);
    await attachmentsStore.pickAndImport(MEMORY_ID);
    await vi.waitFor(() => expect(host.textContent).toContain("attachment exceeds kind size limit"));
    expect(await attachmentsStore.requestMicrophone()).toBe("denied");
    await vi.waitFor(() => expect(host.textContent).toContain(MICROPHONE_DENIED));
  });
  it("shows an honest error when a drop has no filesystem path", async () => {
    const host = mount();
    await ready(host);
    const zone = host.querySelector("[data-drop-zone]")!;
    const file = new File(["x"], "drop.png", { type: "image/png" });
    const event = new Event("drop", { bubbles: true, cancelable: true });
    Object.defineProperty(event, "dataTransfer", { value: { files: [file] } });
    zone.dispatchEvent(event);
    await vi.waitFor(() => expect(host.querySelector("[role='alert']")?.textContent).toContain(DROP_PATH_UNAVAILABLE));
    expect(fake.bridge.attachments.importFromPath).not.toHaveBeenCalled();
  });
  it("discloses links.fetchTitles copy", async () => {
    const host = mount();
    await ready(host);
    await vi.waitFor(() => expect(host.querySelector("[data-fetch-titles]")?.textContent).toContain(FETCH_TITLES_OFF));
  });
});
