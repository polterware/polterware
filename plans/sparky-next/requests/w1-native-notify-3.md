# W1-NATIVE-NOTIFY request 3 — notification-relevant change hints

- Owners: integrator / W1-CORE / W1-STORE, contract owner if wire fields change.
- Blocked sub-task: avoiding Plan calls for note edits beyond the first 140-character notification body (doc 03).
- Frozen Change fields are only `entity`, `id`, `op`; a title edit and a note-tail edit both appear as `memories/update`. Settings changes likewise cannot identify the key. Main cannot distinguish them without fetching every changed memory/note or rebuilding the plan, defeating the requested lazy behavior.
- Current safe bounded behavior: filter irrelevant entities, debounce 500 ms with a 1.5 s maximum wait under sustained writes, serialize Plan calls, then content hash avoids OS re-arming when the visible body is unchanged. Main does not fetch full notes.
- Requested decision: allow that bounded behavior for v1.1, or have the core emit an explicit notification relevance hint based on committed title/status/schedule/completion/body-preview/sound/allDayTime changes. Do not change frozen schemas without approval. Any protocol addition needs matching generated Go/TS and the approved baseline update.
- Core-triggered hidden launches should use the same relevance test, coalesced per transaction/burst. Heartbeat and activity changes must not cause a relaunch loop.
