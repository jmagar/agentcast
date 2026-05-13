#!/usr/bin/env bash
set -Eeuo pipefail

IFS=$'\n\t'

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd -P)"
REF_DIR="$ROOT_DIR/docs/references"
CHANGES_FILE="$REF_DIR/CHANGES.md"
AXON_OUTPUT_DIR="${AXON_OUTPUT_DIR:-$HOME/.axon/output}"

DRY_RUN=false
SKIP_CRAWL=false
SKIP_REPOMIX=false

usage() {
  cat <<'EOF'
Usage: scripts/refresh-docs.sh [OPTIONS]

Refresh all local reference docs while preserving docs/references layout.
Real refresh runs append a full file-level summary to docs/references/CHANGES.md.

Options:
  --dry-run        Print planned crawl and Repomix refreshes without writing.
  --skip-crawl     Refresh Repomix XML packs only.
  --skip-repomix   Refresh Axon-crawled markdown docs only.
  -h, --help       Show this help.

Environment:
  AXON_OUTPUT_DIR  Axon host output directory. Default: ~/.axon/output
  REPOMIX_BIN      Repomix executable. Default: repomix if present, otherwise npx --yes repomix
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --skip-crawl)
      SKIP_CRAWL=true
      shift
      ;;
    --skip-repomix)
      SKIP_REPOMIX=true
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ "$SKIP_CRAWL" == true && "$SKIP_REPOMIX" == true ]]; then
  echo "ERROR: --skip-crawl and --skip-repomix cannot both be set" >&2
  exit 2
fi

log() {
  printf '[refresh-docs] %s\n' "$*"
}

refresh_scope() {
  if [[ "$SKIP_CRAWL" == true ]]; then
    printf 'repomix-only'
  elif [[ "$SKIP_REPOMIX" == true ]]; then
    printf 'crawl-only'
  else
    printf 'full'
  fi
}

require_cmd() {
  local -r cmd="$1"
  command -v "$cmd" >/dev/null 2>&1 || {
    echo "ERROR: required command not found: $cmd" >&2
    exit 1
  }
}

make_tmpdir() {
  mktemp -d "${TMPDIR:-/tmp}/agentcast-refresh-docs.XXXXXX"
}

atomic_replace_dir() {
  local -r src="$1"
  local -r dst="$2"
  local -r parent="$(dirname -- "$dst")"
  local -r base="$(basename -- "$dst")"
  local backup=""

  mkdir -p "$parent"
  backup="$(mktemp -d "$parent/.${base}.backup.XXXXXX")"
  rmdir "$backup"

  if [[ -e "$dst" ]]; then
    mv -- "$dst" "$backup"
  fi

  if mv -- "$src" "$dst"; then
    rm -rf -- "$backup"
  else
    if [[ -e "$backup" ]]; then
      mv -- "$backup" "$dst"
    fi
    return 1
  fi
}

copy_job_output_to_layout() {
  local -r source_dir="$1"
  local -r target_dir="$2"
  local tmp_target=""

  [[ -f "$source_dir/manifest.jsonl" ]] || {
    echo "ERROR: missing Axon manifest: $source_dir/manifest.jsonl" >&2
    return 1
  }
  [[ -d "$source_dir/markdown" ]] || {
    echo "ERROR: missing Axon markdown directory: $source_dir/markdown" >&2
    return 1
  }

  tmp_target="$(make_tmpdir)"
  cp -a "$source_dir/." "$tmp_target/"
  atomic_replace_dir "$tmp_target" "$target_dir"
}

sparse_clone_path() {
  local -r remote="$1"
  local -r sparse_path="$2"
  local -r target_rel="$3"
  local -r mode="${4:-recursive}"
  local -r target_dir="$REF_DIR/$target_rel"
  local tmp_dir=""
  local clone_dir=""
  local tmp_target=""

  log "sparse clone $remote/$sparse_path -> docs/references/$target_rel"
  if [[ "$mode" == "flat-mdx" ]]; then
    log "  mode: flat .mdx files only"
  fi
  if [[ "$DRY_RUN" == true ]]; then
    return 0
  fi

  require_cmd git
  tmp_dir="$(make_tmpdir)"
  clone_dir="$tmp_dir/repo"
  tmp_target="$tmp_dir/output"

  git clone --filter=blob:none --sparse --depth=1 "$remote" "$clone_dir" >/dev/null
  git -C "$clone_dir" sparse-checkout set "$sparse_path" >/dev/null

  mkdir -p "$tmp_target"
  case "$mode" in
    flat-mdx)
      find "$clone_dir/$sparse_path" -maxdepth 1 -type f -name '*.mdx' -exec cp -a {} "$tmp_target/" \;
      ;;
    recursive)
      cp -a "$clone_dir/$sparse_path/." "$tmp_target/"
      ;;
    *)
      echo "ERROR: unknown sparse clone mode: $mode" >&2
      rm -rf -- "$tmp_dir"
      return 1
      ;;
  esac

  atomic_replace_dir "$tmp_target" "$target_dir"
  rm -rf -- "$tmp_dir"
}

newest_domain_run() {
  local -r domain="$1"
  local -r domain_dir="$AXON_OUTPUT_DIR/domains/$domain"

  [[ -d "$domain_dir" ]] || return 1
  find "$domain_dir" -mindepth 1 -maxdepth 1 -type d -printf '%T@ %p\n' \
    | sort -nr \
    | awk 'NR == 1 { $1=""; sub(/^ /, ""); print }'
}

crawl_docs() {
  local -r url="$1"
  local -r domain="$2"
  local -r target_rel="$3"
  local -r target_dir="$REF_DIR/$target_rel"
  local output=""
  local job_id=""
  local source_dir=""

  log "crawl $url -> docs/references/$target_rel"
  if [[ "$DRY_RUN" == true ]]; then
    return 0
  fi

  require_cmd axon
  output="$(axon crawl "$url" --wait true --yes 2>&1)"
  printf '%s\n' "$output"

  job_id="$(awk '/^Job ID:/ { print $3 }' <<<"$output" | tail -1)"
  if [[ -n "$job_id" && -d "$AXON_OUTPUT_DIR/domains/$domain/$job_id" ]]; then
    source_dir="$AXON_OUTPUT_DIR/domains/$domain/$job_id"
  else
    source_dir="$(newest_domain_run "$domain")"
  fi

  [[ -n "$source_dir" && -d "$source_dir" ]] || {
    echo "ERROR: could not locate Axon output for $domain under $AXON_OUTPUT_DIR/domains" >&2
    return 1
  }

  copy_job_output_to_layout "$source_dir" "$target_dir"
}

repomix_command() {
  if [[ -n "${REPOMIX_BIN:-}" ]]; then
    "$REPOMIX_BIN" "$@"
  elif command -v repomix >/dev/null 2>&1; then
    repomix "$@"
  else
    require_cmd npx
    npx --yes repomix "$@"
  fi
}

pack_repo() {
  local -r remote="$1"
  local -r target_rel="$2"
  local -r include_patterns="${3:-}"
  local -r ignore_patterns="${4:-}"
  local -r target_file="$REF_DIR/$target_rel"
  local tmp_dir=""
  local tmp_file=""

  log "pack $remote -> docs/references/$target_rel"
  if [[ -n "$include_patterns" ]]; then
    log "  include: $include_patterns"
  fi
  if [[ -n "$ignore_patterns" ]]; then
    log "  ignore: $ignore_patterns"
  fi
  if [[ "$DRY_RUN" == true ]]; then
    return 0
  fi

  tmp_dir="$(make_tmpdir)"
  tmp_file="$tmp_dir/repomix-output.xml"

  local args=(--remote "$remote" --style xml --output "$tmp_file" --top-files-len 10)
  if [[ -n "$include_patterns" ]]; then
    args+=(--include "$include_patterns")
  fi
  if [[ -n "$ignore_patterns" ]]; then
    args+=(--ignore "$ignore_patterns")
  fi

  repomix_command "${args[@]}"
  [[ -s "$tmp_file" ]] || {
    echo "ERROR: Repomix produced no output for $remote" >&2
    rm -rf -- "$tmp_dir"
    return 1
  }

  mkdir -p "$(dirname -- "$target_file")"
  mv -- "$tmp_file" "$target_file"
  rm -rf -- "$tmp_dir"
}

write_index() {
  local acp_count=0
  local mcp_count=0
  local claude_count=0
  local fastmcp_docs_count=0
  local mcporter_docs_count=0

  [[ -d "$REF_DIR/acp/docs" ]] && acp_count="$(find "$REF_DIR/acp/docs" -type f | wc -l | tr -d ' ')"
  [[ -d "$REF_DIR/mcp/docs" ]] && mcp_count="$(find "$REF_DIR/mcp/docs" -type f | wc -l | tr -d ' ')"
  [[ -d "$REF_DIR/claude-code" ]] && claude_count="$(find "$REF_DIR/claude-code" -type f | wc -l | tr -d ' ')"
  [[ -d "$REF_DIR/fastmcp/docs" ]] && fastmcp_docs_count="$(find "$REF_DIR/fastmcp/docs" -type f | wc -l | tr -d ' ')"
  [[ -d "$REF_DIR/mcporter/docs" ]] && mcporter_docs_count="$(find "$REF_DIR/mcporter/docs" -type f | wc -l | tr -d ' ')"

  cat > "$REF_DIR/INDEX.md" <<EOF
# Reference Index

This directory stores local reference material for protocol, SDK, CLI, and related project research.

## Layout

| Path | Contents | Source |
| --- | --- | --- |
| \`acp/docs/\` | Axon-crawled markdown docs and \`manifest.jsonl\` | \`https://agentclientprotocol.com\` |
| \`acp/repos/\` | Repomix XML packs for ACP repos | \`agentclientprotocol/*\` |
| \`mcp/docs/\` | Axon-crawled markdown docs and \`manifest.jsonl\` | \`https://modelcontextprotocol.io\` |
| \`mcp/repos/\` | Repomix XML packs for MCP repos | \`modelcontextprotocol/*\` |
| \`claude-code/\` | Axon-crawled markdown docs and \`manifest.jsonl\` | \`https://code.claude.com/\` |
| \`fastmcp/docs/\` | Sparse checkout of top-level FastMCP CLI \`.mdx\` docs | \`PrefectHQ/fastmcp/docs/cli\` |
| \`fastmcp/repos/\` | Repomix XML pack for the full FastMCP repo | \`PrefectHQ/fastmcp\` |
| \`jmagar/\` | Repomix XML packs for local/user-owned reference repos | \`jmagar/*\` |
| \`mcporter/docs/\` | Sparse checkout of mcporter docs | \`openclaw/mcporter/docs\` |
| \`mcporter/repos/\` | Repomix XML pack for mcporter | \`openclaw/mcporter\` |

## Crawled Docs

Crawled docs are copied from Axon's host output under \`~/.axon/output/domains/\`. Each crawled-doc directory contains:

- \`manifest.jsonl\` with crawl metadata.
- \`markdown/\` with one markdown file per crawled page.

Current crawl copies:

| Path | Files |
| --- | ---: |
| \`acp/docs/\` | $acp_count |
| \`mcp/docs/\` | $mcp_count |
| \`claude-code/\` | $claude_count |
| \`fastmcp/docs/\` | $fastmcp_docs_count |
| \`mcporter/docs/\` | $mcporter_docs_count |

## Repomix Packs

Repomix packs are consolidated XML snapshots intended for codebase-level reference and search.

### ACP Repos

- \`acp/repos/agentclientprotocol-agent-client-protocol.xml\`
- \`acp/repos/agentclientprotocol-claude-agent-acp.xml\`
- \`acp/repos/agentclientprotocol-codex-acp.xml\`
- \`acp/repos/agentclientprotocol-registry.xml\`
- \`acp/repos/agentclientprotocol-rust-sdk.xml\`
- \`acp/repos/agentclientprotocol-typescript-sdk.xml\`

### MCP Repos

- \`mcp/repos/modelcontextprotocol-ext-auth.xml\`
- \`mcp/repos/modelcontextprotocol-modelcontextprotocol.xml\`
- \`mcp/repos/modelcontextprotocol-registry.xml\`
- \`mcp/repos/modelcontextprotocol-rust-sdk.xml\`

### Other Packs

- \`fastmcp/repos/prefecthq-fastmcp.xml\`
- \`jmagar/jmagar-aurora-design-system.xml\`
- \`jmagar/jmagar-lab.xml\`
- \`mcporter/repos/openclaw-mcporter.xml\`
EOF
}

snapshot_references() {
  local -r output_file="$1"

  if [[ ! -d "$REF_DIR" ]]; then
    : > "$output_file"
    return 0
  fi

  (
    cd "$REF_DIR"
    find . -type f \
      ! -path './CHANGES.md' \
      ! -path './CHANGES-REPORT.md' \
      ! -path './archive/changes-reports/*' \
      -print0 \
      | sort -z \
      | xargs -0 -r sha256sum \
      | sed 's#  \./#  #'
  ) > "$output_file"
}

snapshot_paths() {
  local -r snapshot_file="$1"
  awk '{ $1=""; sub(/^  /, ""); print }' "$snapshot_file"
}

change_count() {
  wc -l | tr -d ' '
}

print_limited_list() {
  local -r title="$1"
  local -r file="$2"
  local -r limit="${3:-40}"
  local count=0

  count="$(wc -l < "$file" | tr -d ' ')"
  if [[ "$count" == 0 ]]; then
    return 0
  fi

  log "$title ($count)"
  sed -n "1,${limit}p" "$file" | sed 's/^/  - /'
  if (( count > limit )); then
    log "  ... $((count - limit)) more"
  fi
}

ensure_changes_file() {
  mkdir -p "$REF_DIR"
  if [[ -f "$CHANGES_FILE" ]]; then
    return 0
  fi

  cat > "$CHANGES_FILE" <<EOF
---
title: Reference Refresh Change Log
generated_by: scripts/refresh-docs.sh
created_at: $(date -u +%Y-%m-%dT%H:%M:%SZ)
timezone: UTC
purpose: Append-only log of docs/references refresh changes
---

# Reference Refresh Change Log

Each entry records file-level changes detected after a real \`scripts/refresh-docs.sh\` run. Generated log/report files are excluded from the detected file-change set.
EOF
}

append_file_section() {
  local -r title="$1"
  local -r file="$2"
  local count=0

  count="$(wc -l < "$file" | tr -d ' ')"
  {
    printf '\n### %s (%s)\n\n' "$title" "$count"
    if [[ "$count" == 0 ]]; then
      printf '_None_\n'
    else
      sed 's/^/- `/' "$file" | sed 's/$/`/'
    fi
  } >> "$CHANGES_FILE"
}

append_changes_log() {
  local -r added_file="$1"
  local -r modified_file="$2"
  local -r removed_file="$3"
  local -r added_count="$4"
  local -r modified_count="$5"
  local -r removed_count="$6"
  local -r timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

  ensure_changes_file

  {
    printf '\n## %s\n\n' "$timestamp"
    printf '%s\n' '- script: `scripts/refresh-docs.sh`'
    printf '%s\n' "- scope: \`$(refresh_scope)\`"
    printf '%s\n' "- axon_output_dir: \`$AXON_OUTPUT_DIR\`"
    printf '%s\n' "- summary: \`$added_count added, $modified_count modified, $removed_count removed\`"
  } >> "$CHANGES_FILE"

  append_file_section "Added" "$added_file"
  append_file_section "Modified" "$modified_file"
  append_file_section "Removed" "$removed_file"
}

summarize_reference_changes() {
  local -r before_snapshot="$1"
  local -r after_snapshot="$2"
  local tmp_dir=""
  local before_paths=""
  local after_paths=""
  local added=""
  local removed=""
  local common=""
  local modified=""
  local added_count=0
  local removed_count=0
  local modified_count=0

  tmp_dir="$(make_tmpdir)"
  before_paths="$tmp_dir/before.paths"
  after_paths="$tmp_dir/after.paths"
  added="$tmp_dir/added"
  removed="$tmp_dir/removed"
  common="$tmp_dir/common"
  modified="$tmp_dir/modified"

  snapshot_paths "$before_snapshot" | sort > "$before_paths"
  snapshot_paths "$after_snapshot" | sort > "$after_paths"

  comm -13 "$before_paths" "$after_paths" > "$added"
  comm -23 "$before_paths" "$after_paths" > "$removed"
  comm -12 "$before_paths" "$after_paths" > "$common"

  : > "$modified"
  while IFS= read -r path; do
    local before_line=""
    local after_line=""
    before_line="$(grep -F "  $path" "$before_snapshot" || true)"
    after_line="$(grep -F "  $path" "$after_snapshot" || true)"
    if [[ "${before_line%%  *}" != "${after_line%%  *}" ]]; then
      printf '%s\n' "$path" >> "$modified"
    fi
  done < "$common"

  added_count="$(change_count < "$added")"
  removed_count="$(change_count < "$removed")"
  modified_count="$(change_count < "$modified")"

  if [[ "$added_count" == 0 && "$removed_count" == 0 && "$modified_count" == 0 ]]; then
    log "change summary: no reference changes"
    append_changes_log "$added" "$modified" "$removed" "$added_count" "$modified_count" "$removed_count"
    rm -rf -- "$tmp_dir"
    return 0
  fi

  log "change summary: $added_count added, $modified_count modified, $removed_count removed"
  print_limited_list "added" "$added"
  print_limited_list "modified" "$modified"
  print_limited_list "removed" "$removed"
  append_changes_log "$added" "$modified" "$removed" "$added_count" "$modified_count" "$removed_count"

  rm -rf -- "$tmp_dir"
}

main() {
  local snapshot_dir=""
  local before_snapshot=""
  local after_snapshot=""

  if [[ "$DRY_RUN" != true ]]; then
    snapshot_dir="$(make_tmpdir)"
    before_snapshot="$snapshot_dir/before.sha256"
    after_snapshot="$snapshot_dir/after.sha256"
    snapshot_references "$before_snapshot"
  fi

  mkdir -p \
    "$REF_DIR/acp/docs" \
    "$REF_DIR/acp/repos" \
    "$REF_DIR/mcp/docs" \
    "$REF_DIR/mcp/repos" \
    "$REF_DIR/claude-code" \
    "$REF_DIR/fastmcp/docs" \
    "$REF_DIR/fastmcp/repos" \
    "$REF_DIR/fastmcp" \
    "$REF_DIR/jmagar" \
    "$REF_DIR/mcporter/docs" \
    "$REF_DIR/mcporter/repos" \
    "$REF_DIR/mcporter"

  if [[ "$SKIP_CRAWL" != true ]]; then
    crawl_docs "https://agentclientprotocol.com" "agentclientprotocol.com" "acp/docs"
    crawl_docs "https://modelcontextprotocol.io" "modelcontextprotocol.io" "mcp/docs"
    crawl_docs "https://code.claude.com/" "code.claude.com" "claude-code"
  fi

  if [[ "$SKIP_REPOMIX" != true ]]; then
    pack_repo "agentclientprotocol/agent-client-protocol" "acp/repos/agentclientprotocol-agent-client-protocol.xml"
    pack_repo "agentclientprotocol/claude-agent-acp" "acp/repos/agentclientprotocol-claude-agent-acp.xml"
    pack_repo "agentclientprotocol/codex-acp" "acp/repos/agentclientprotocol-codex-acp.xml"
    pack_repo "agentclientprotocol/registry" "acp/repos/agentclientprotocol-registry.xml"
    pack_repo "agentclientprotocol/rust-sdk" "acp/repos/agentclientprotocol-rust-sdk.xml"
    pack_repo "agentclientprotocol/typescript-sdk" "acp/repos/agentclientprotocol-typescript-sdk.xml"

    pack_repo "modelcontextprotocol/ext-auth" "mcp/repos/modelcontextprotocol-ext-auth.xml"
    pack_repo "modelcontextprotocol/modelcontextprotocol" "mcp/repos/modelcontextprotocol-modelcontextprotocol.xml"
    pack_repo "modelcontextprotocol/registry" "mcp/repos/modelcontextprotocol-registry.xml"
    pack_repo "modelcontextprotocol/rust-sdk" "mcp/repos/modelcontextprotocol-rust-sdk.xml"

    pack_repo "PrefectHQ/fastmcp" "fastmcp/repos/prefecthq-fastmcp.xml"
    pack_repo "jmagar/aurora-design-system" "jmagar/jmagar-aurora-design-system.xml"
    pack_repo "jmagar/lab" "jmagar/jmagar-lab.xml" "" "docs/references/**"
    pack_repo "openclaw/mcporter" "mcporter/repos/openclaw-mcporter.xml"
  fi

  if [[ "$SKIP_REPOMIX" != true ]]; then
    sparse_clone_path "https://github.com/PrefectHQ/fastmcp" "docs/cli" "fastmcp/docs" "flat-mdx"
    sparse_clone_path "https://github.com/openclaw/mcporter" "docs" "mcporter/docs" "recursive"
  fi

  if [[ "$DRY_RUN" != true ]]; then
    write_index
    snapshot_references "$after_snapshot"
    summarize_reference_changes "$before_snapshot" "$after_snapshot"
    rm -rf -- "$snapshot_dir"
  fi

  log "done"
}

main "$@"
