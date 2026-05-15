#!/usr/bin/env bash
set -euo pipefail

MAX_RS="${AGENTCAST_MAX_RS_LINES:-400}"
MAX_TS="${AGENTCAST_MAX_TS_LINES:-350}"

is_test_file() {
  local file="$1"
  [[ "$file" =~ (/tests?/|_test\.rs$|/tests\.rs$) ]] && return 0
  [[ "$file" =~ (\.(test|spec)\.(ts|tsx)$|/__tests__/) ]] && return 0
  return 1
}

count_effective_loc() {
  local file="$1"
  local end_line="${2:-0}"
  awk -v end_line="$end_line" '
    BEGIN { count = 0; in_block = 0 }
    end_line > 0 && NR > end_line { exit }
    {
      line = $0
      sub(/^[[:space:]]+/, "", line)
      if (line == "") next

      if (in_block) {
        if (line ~ /\*\//) {
          sub(/^.*\*\//, "", line)
          sub(/^[[:space:]]+/, "", line)
          in_block = 0
          if (line == "") next
        } else {
          next
        }
      }

      if (line ~ /^\/\//) next

      if (line ~ /^\/\*/) {
        if (line ~ /\*\//) {
          sub(/^\/\*.*\*\//, "", line)
          sub(/^[[:space:]]+/, "", line)
          if (line == "") next
        } else {
          in_block = 1
          next
        }
      }

      count++
    }
    END { print count }
  ' "$file"
}

rs_production_lines() {
  local file="$1"
  local end_line=0
  local test_mod_line

  test_mod_line="$(awk '
    /#\[cfg\(test\)\]/ { cfg_line = NR; next }
    cfg_line && /^[[:space:]]*(pub[[:space:]]+)?mod [a-z_]+[[:space:]]*;/ { print cfg_line; exit }
    cfg_line && /^[[:space:]]*(pub[[:space:]]+)?mod [a-z_]+[[:space:]]*\{/ { print cfg_line; exit }
    { cfg_line = 0 }
  ' "$file" || true)"

  if [[ -n "$test_mod_line" ]]; then
    end_line=$((test_mod_line - 1))
  fi

  count_effective_loc "$file" "$end_line"
}

files=()
if [[ "$#" -gt 0 ]]; then
  files=("$@")
else
  while IFS= read -r file; do
    files+=("$file")
  done < <(git ls-files '*.rs' '*.ts' '*.tsx' ':!:docs/references/**')
fi

violations=()
for file in "${files[@]}"; do
  [[ -f "$file" ]] || continue
  is_test_file "$file" && continue

  case "$file" in
    *.rs)
      lines="$(rs_production_lines "$file")"
      limit="$MAX_RS"
      ;;
    *.ts|*.tsx)
      lines="$(count_effective_loc "$file")"
      limit="$MAX_TS"
      ;;
    *)
      continue
      ;;
  esac

  if (( lines > limit )); then
    violations+=("${file}: ${lines} effective production lines (limit: ${limit})")
  fi
done

if (( ${#violations[@]} > 0 )); then
  echo "File-size check failed; split large production modules:" >&2
  printf '  - %s\n' "${violations[@]}" >&2
  echo "Limits: .rs=${MAX_RS}, .ts/.tsx=${MAX_TS}. Override with AGENTCAST_MAX_RS_LINES or AGENTCAST_MAX_TS_LINES." >&2
  exit 1
fi

printf 'File-size check passed (%s file(s) checked).\n' "${#files[@]}"
