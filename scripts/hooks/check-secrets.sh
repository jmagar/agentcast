#!/usr/bin/env bash
set -euo pipefail

if ! command -v gitleaks >/dev/null 2>&1; then
  printf 'gitleaks is required for the approved staged secret scan hook.\n' >&2
  printf 'Install it or remove the hook only after updating docs/DECISIONS.md.\n' >&2
  exit 1
fi

gitleaks protect --staged --no-banner
