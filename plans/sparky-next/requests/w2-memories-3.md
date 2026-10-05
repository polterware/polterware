# W2-MEMORIES: safe deletion undo

Owner: Erick / integration. Decision requested; no contract files changed.

The plan asks for immediate deletion and recreation from an in-memory payload for eight seconds. CreateMemoryRequest cannot recreate schedules, places, occurrence completions, attachment IDs or attachment bytes after the core has cascaded deletion. Recreating only title/note/checklist/tags would silently lose data.

The delivered implementation instead retains the existing aggregate, hides the row with an honest "Deletion queued · Undo available for 8 seconds" state, and sends Delete with expectedRev after eight seconds. Undo cancels the queued deletion. Failed/conflicting deletion restores the row and reports failure. Closing the window before the timer fires cancels the pending deletion rather than losing data. Completed-memory undo still calls SetStatus with active. Please accept this deviation or provide a frozen-contract-compatible aggregate restore API before requiring immediate-delete undo.
