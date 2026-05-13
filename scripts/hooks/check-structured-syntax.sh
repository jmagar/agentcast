#!/usr/bin/env bash
set -euo pipefail

failed=0

check_json() {
  local file="$1"
  local tmp="$2"
  if ! jq empty "$tmp" >/dev/null; then
    printf 'Invalid JSON: %s\n' "$file" >&2
    failed=1
  fi
}

check_toml() {
  local file="$1"
  local tmp="$2"
  if ! taplo check "$tmp" >/dev/null; then
    printf 'Invalid TOML: %s\n' "$file" >&2
    failed=1
  fi
}

check_yaml() {
  local file="$1"
  local tmp="$2"
  if ! python3 - "$tmp" >/dev/null <<'PY'
import sys
import yaml

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    yaml.safe_load(handle)
PY
  then
    printf 'Invalid YAML: %s\n' "$file" >&2
    failed=1
  fi
}

while IFS= read -r -d '' file; do
  if ! git show ":$file" >/dev/null 2>&1; then
    continue
  fi

  case "$file" in
    *.json|*.toml|*.yaml|*.yml) ;;
    *) continue ;;
  esac

  case "$file" in
    *.json) tmp="$(mktemp --suffix=.json)" ;;
    *.toml) tmp="$(mktemp --suffix=.toml)" ;;
    *.yaml) tmp="$(mktemp --suffix=.yaml)" ;;
    *.yml) tmp="$(mktemp --suffix=.yml)" ;;
  esac
  trap 'rm -f "$tmp"' RETURN
  git show ":$file" > "$tmp"

  case "$file" in
    *.json) check_json "$file" "$tmp" ;;
    *.toml) check_toml "$file" "$tmp" ;;
    *.yaml|*.yml) check_yaml "$file" "$tmp" ;;
  esac

  rm -f "$tmp"
  trap - RETURN
done < <(git diff --cached --name-only -z --diff-filter=ACMR)

exit "$failed"
