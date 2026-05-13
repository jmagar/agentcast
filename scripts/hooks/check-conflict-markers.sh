#!/usr/bin/env bash
set -euo pipefail

failed=0

while IFS= read -r -d '' file; do
  if ! git show ":$file" >/dev/null 2>&1; then
    continue
  fi

  matches="$(git show ":$file" | grep -nE '^(<<<<<<<|=======|>>>>>>>)($|[[:space:]])' || true)"
  if [[ -n "$matches" ]]; then
    printf 'Conflict markers found in %s:\n%s\n' "$file" "$matches" >&2
    failed=1
  fi
done < <(git diff --cached --name-only -z --diff-filter=ACMR)

exit "$failed"
