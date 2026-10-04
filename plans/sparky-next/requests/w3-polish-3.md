# W3-POLISH request 3: Mind editor routing

Owner: W2-SHELL / W2-MINDS / integrator. Status: open.

The shell replaces every /mind/:id route with memoryCollection when registered. MindPage contains the name/color/icon, parent move, delete, breadcrumb and child Mind editor for the same route. Clicking a Mind therefore bypasses those management controls; /minds only exposes the top-level list and creation dialog.

Requested change: compose Mind management and the memory collection through the existing registry with one clear heading; keep management controls reachable by keyboard. Add an integration test that opens a Mind through the sidebar, edits its name and reaches Move to/Delete/child Minds plus its memory collection. W3-POLISH preserves route behavior and records this gap instead of changing registry or shell logic.
