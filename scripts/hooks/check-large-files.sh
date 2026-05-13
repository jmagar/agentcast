#!/usr/bin/env bash
set -euo pipefail

max_bytes="${AGENTCAST_HOOK_MAX_FILE_BYTES:-5242880}"
failed=0

while IFS= read -r -d '' file; do
  if ! git show ":$file" >/dev/null 2>&1; then
    continue
  fi

  size="$(git cat-file -s ":$file")"
  if (( size > max_bytes )); then
    printf 'Staged file is larger than %s bytes: %s (%s bytes)\n' "$max_bytes" "$file" "$size" >&2
    printf 'If this is intentional, commit with AGENTCAST_HOOK_MAX_FILE_BYTES adjusted for this command.\n' >&2
    failed=1
  fi
done < <(git diff --cached --name-only -z --diff-filter=ACMR)

exit "$failed"
