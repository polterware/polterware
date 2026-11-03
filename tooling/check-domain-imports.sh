#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DOMAINS="$ROOT/backend/domains"
if [[ ! -d "$DOMAINS" ]]; then
  echo "OK: no domains yet"
  exit 0
fi
# Forbidden import patterns in domain packages
bad=$(rg -n 'modernc.org/sqlite|github.com/mattn/go-sqlite3|connectrpc.com/connect|github.com/jackc/pgx|electron|"database/sql"' \
  "$DOMAINS" --glob '*.go' || true)
# Allow database/sql only if we find it — domains must not use it
if echo "$bad" | rg -q .; then
  echo "FAIL: domains import concrete infrastructure:"
  echo "$bad"
  exit 1
fi
echo "OK: domain import boundaries"
