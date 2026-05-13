#!/usr/bin/env bash
set -Eeuo pipefail

IFS=$'\n\t'

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd -P)"

MODEL="${CLAUDE_MODEL:-}"
EFFORT="${CLAUDE_EFFORT:-high}"
MAX_TURNS="${CLAUDE_MAX_TURNS:-}"
OUTPUT_FORMAT="${CLAUDE_OUTPUT_FORMAT:-text}"
NAME="${CLAUDE_SESSION_NAME:-refresh-docs-review}"
EXTRA_PROMPT=""
STREAM=true

usage() {
  cat <<'EOF'
Usage: scripts/review-changes.sh [OPTIONS] [EXTRA_PROMPT...]

Run Claude Code headless mode to invoke the local refresh-docs skill. The skill
refreshes docs/references, appends docs/references/CHANGES.md, reviews the
latest reference changes against the repo, archives any existing report, and
writes docs/references/CHANGES-REPORT.md.

Options:
  --model MODEL          Claude model alias/name to pass through.
  --effort LEVEL         Reasoning effort. Default: high.
  --max-turns N          Limit agentic turns.
  --output-format FORMAT Output format for claude -p when --text is used. Default: text.
  --stream              Stream Claude Code events live. Default.
  --text                Disable streaming and print the final text response only.
  --name NAME            Claude session display name. Default: refresh-docs-review.
  -h, --help             Show this help.

Environment:
  CLAUDE_MODEL           Default --model value.
  CLAUDE_EFFORT          Default --effort value.
  CLAUDE_MAX_TURNS       Default --max-turns value.
  CLAUDE_OUTPUT_FORMAT   Default --output-format value.
  CLAUDE_SESSION_NAME    Default --name value.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --model)
      MODEL="${2:?--model requires a value}"
      shift 2
      ;;
    --effort)
      EFFORT="${2:?--effort requires a value}"
      shift 2
      ;;
    --max-turns)
      MAX_TURNS="${2:?--max-turns requires a value}"
      shift 2
      ;;
    --output-format)
      OUTPUT_FORMAT="${2:?--output-format requires a value}"
      shift 2
      ;;
    --stream)
      STREAM=true
      shift
      ;;
    --text)
      STREAM=false
      shift
      ;;
    --name)
      NAME="${2:?--name requires a value}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --)
      shift
      EXTRA_PROMPT="$*"
      break
      ;;
    *)
      EXTRA_PROMPT="$*"
      break
      ;;
  esac
done

require_cmd() {
  local -r cmd="$1"
  command -v "$cmd" >/dev/null 2>&1 || {
    echo "ERROR: required command not found: $cmd" >&2
    exit 1
  }
}

require_cmd claude

if [[ ! -f "$ROOT_DIR/.agents/skills/refresh-docs/SKILL.md" ]]; then
  echo "ERROR: missing local refresh-docs skill at .agents/skills/refresh-docs/SKILL.md" >&2
  exit 1
fi

if [[ ! -e "$ROOT_DIR/.claude/skills" ]]; then
  echo "ERROR: missing .claude/skills symlink or directory; Claude may not discover local skills" >&2
  exit 1
fi

prompt="$(
  cat <<'EOF'
Use the local refresh-docs skill.

Run the full refresh-docs workflow exactly as defined by the skill:
- refresh docs/references,
- inspect the latest docs/references/CHANGES.md entry,
- review every changed reference document from that latest entry,
- cross-reference those reference changes against the current codebase,
- archive any existing docs/references/CHANGES-REPORT.md with the bundled helper,
- write a new docs/references/CHANGES-REPORT.md with required updates, verification, and possible new additions.

Do not make application code changes. Only update the reference refresh outputs and the CHANGES-REPORT.md requested by the skill.
EOF
)"

if [[ -n "$EXTRA_PROMPT" ]]; then
  prompt="$prompt

Additional user instructions:
$EXTRA_PROMPT"
fi

args=(-p "$prompt" --name "$NAME" --effort "$EFFORT")

if [[ "$STREAM" == true ]]; then
  args+=(--output-format stream-json --verbose --include-partial-messages --include-hook-events)
else
  args+=(--output-format "$OUTPUT_FORMAT")
fi

if [[ -n "$MODEL" ]]; then
  args+=(--model "$MODEL")
fi

if [[ -n "$MAX_TURNS" ]]; then
  args+=(--max-turns "$MAX_TURNS")
fi

cd "$ROOT_DIR"
exec claude "${args[@]}"
