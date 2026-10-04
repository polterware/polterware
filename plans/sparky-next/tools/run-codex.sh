#!/usr/bin/env bash
# Launch one Sparky Next workstream with Codex in its own git worktree.
# usage: run-codex.sh <ws-id lowercase, e.g. w1-store> <WS-ID upper> <model> <effort low|medium|high> "<extra docs to read>" [wave-file e.g. 08-wave1.md]
# Creates ../polterware-wt/<ws-id> (branch sparky/<ws-id>) from sparky-next if missing. Never pushes.
set -u
ws="$1"; WS="$2"; model="$3"; effort="$4"; docs="$5"; wave="${6:-07-wave0.md}"
repo="$(cd "$(dirname "$0")/../../.." && pwd)"                 # any worktree of the repo
root="$(cd "$repo/.." && pwd)"                                 # .../polterware-wt (or sibling dir)
wt="$(git -C "$repo" rev-parse --git-common-dir | xargs -I{} dirname {})/../polterware-wt/$ws"
mkdir -p "$(dirname "$wt")"
if [ ! -d "$wt" ]; then git -C "$repo" worktree add -q "$wt" -b "sparky/$ws" sparky-next || exit 1; fi
wt="$(cd "$wt" && pwd)"
logs="$HOME/.sparky-next-logs"; mkdir -p "$logs"
PROMPT="You are executing workstream $WS of the Sparky Next plan. Work ONLY inside this worktree ($wt, branch sparky/$ws). Read, in order: plans/sparky-next/README.md, AGENT-PROTOCOL.md, 00-decisions.md (read the Resolutions log: it records decisions that changed the original plan), $docs, then section \"$WS\" in plans/sparky-next/$wave. The integration branch sparky-next already contains everything merged so far. Run pnpm install first if you need TypeScript tooling. Follow the protocol exactly: file ownership (only your Owns paths), commits \"sparky($ws): ...\", run the Verify commands and acceptance criteria, write plans/sparky-next/status/$ws.md and commit everything. Do NOT push anything (no branches, no tags to the remote). Do not ask questions unless blocked; if blocked, write the status file with Result: blocked and explain."
# IMPORTANT: stdin must be closed (</dev/null) or codex waits forever on "Reading additional input from stdin".
# IMPORTANT: -c model_provider=openai, because ~/.codex/config.toml defaults to a local Ollama provider.
codex exec -C "$wt" -m "$model" -c model_reasoning_effort="$effort" -c model_provider=openai -s danger-full-access \
  -o "$logs/$ws.last.txt" "$PROMPT" < /dev/null > "$logs/$ws.log" 2>&1
echo "exit=$?" >> "$logs/$ws.log"
