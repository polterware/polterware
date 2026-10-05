# W2-ATTACH-UI request 1 — drop paths and recorded-audio temp files

Owner: W1-ELECTRON / integrator. No frozen RPC change.

The renderer cannot import dropped files or MediaRecorder output without an absolute path that Main has added to `grants.import`. `AttachmentService.ImportFromPath` is already gated by that grant (`Select attachment with the file dialog first`). Preload does not expose `webUtils.getPathForFile`, and `app` has no method that writes a granted temp file.

## Dropped files

Add a preload-only helper (do not put Node or `webUtils` in `packages/**`):

```ts
app.pathsForDroppedFiles(files: File[]): Promise<string[]>
```

Implementation: map each `File` with Electron `webUtils.getPathForFile`, ignore entries without an absolute path, add each path to `grants.import`, return the granted paths. Validate that the argument is a File list (or an IPC-safe identifier Main already understands). Keep `pickFiles()` as the dialog path.

Until this exists, W2-ATTACH-UI shows: "Dropped files have no filesystem path in the renderer. Use Add photo or Add file so Main can grant the path, then import."

## Recorded audio

`05-search-attachments-map.md` says the renderer streams MediaRecorder chunks to Main, Main writes a temp file, then ImportFromPath runs. There is no such app method. Add:

```ts
app.writeTempFile(bytes: Uint8Array, name: string): Promise<string>
```

Main writes under a core-owned temp directory, adds the path to `grants.import`, and returns the absolute path. The attachments feature then calls `attachments.importFromPath` with `expectedRev`. Validate name (basename only, audio extension) and size (audio ≤ 100 MB). Do not accept renderer-chosen destination paths.

Until this exists, W2-ATTACH-UI records with `app.startMicrophone()` + MediaRecorder and then shows: "Recording finished, but Main cannot write a granted temp file yet. Save the clip and import it with Add file."

## Out of scope for this request

Do not change `sparky-file://` handling. Do not add `file://` access. Do not persist provider credentials. W2-ATTACH-UI will call the new methods once they exist; it will not work around the missing path by copying bytes through another RPC.
