#!/usr/bin/env bash
set -euo pipefail

failed=0

while IFS= read -r -d '' file; do
  if ! git show ":$file" >/dev/null 2>&1; then
    continue
  fi

  if git show ":$file" | grep -q $'\r$'; then
    printf 'CRLF line endings found in staged file: %s\n' "$file" >&2
    failed=1
  fi

  mode="$(git ls-files -s -- "$file" | awk '{print $1}' | tail -n 1)"
  if [[ "$mode" == "100755" ]]; then
    first_line="$(git show ":$file" | sed -n '1p')"
    case "$file" in
      scripts/*|*.sh|*.bash|*.zsh) ;;
      *)
        if [[ "$first_line" != '#!'* ]]; then
          printf 'Executable bit set on non-script staged file: %s\n' "$file" >&2
          failed=1
        fi
        ;;
    esac
  fi
done < <(git diff --cached --name-only -z --diff-filter=ACMR)

exit "$failed"
