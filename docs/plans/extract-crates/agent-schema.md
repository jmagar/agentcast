---
title: "agent-schema Extraction Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md"
  - "docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-schema Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `agent-schema` as the JSON Schema normalization and validation crate for launcher actions, MCP tools, CLI arguments, API validation, and future forms.

**Architecture:** Schema handling is shared infrastructure. It should convert MCP/registry/tool schemas into AgentCast-owned normalized metadata, but it must not render CLI or UI controls directly.

**Tech Stack:** Rust 2024, serde/serde_json, thiserror.

---

## MVP Position

`agent-schema` supports the MCP launcher MVP by defining how MCP tool input schemas become validated invocation inputs and future schema-driven UX metadata.

## Lab Evidence Read

- `../lab/crates/lab-apis/src/core.rs`
- `../lab/crates/lab/src/mcp/server.rs`
- `../lab/crates/lab/src/api/openapi.rs`
- `../lab/crates/lab/src/audit/checks/ui_schema.rs`
- `../lab/apps/gateway-admin/components/ai/schema-display.tsx`
- `../lab/apps/gateway-admin/lib/stdio-command.ts`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab schema source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`.
- MCP JSON Schema and tool input/output schema claims are cross-checked against `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md` and `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`.
- ACP content/schema claims are cross-checked against `docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md` and `docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md`.

Live source discovery command:

```bash
rg -n "schema|json schema|parameters|params|validation|enum|required|tool input" ../lab
```

## Live Lab Findings

- `lab-apis/src/core.rs` is the source for action parameter metadata.
- `mcp/server.rs::action_schema` turns action metadata into MCP JSON Schema.
- `api/openapi.rs` has useful parameter-to-OpenAPI type mapping and fallback tests.
- `schema-display.tsx` is UI evidence only; Rust should output normalized metadata, not components.

## Extraction Boundary

Extract into `agent-schema`:

- JSON Schema normalization for object, string, number, integer, boolean, array, enum, and nested object fields.
- required/default/description/title handling.
- validation helpers for invocation payloads.
- metadata used by CLI args and future forms.
- unsupported-schema fallback classification.

Keep out of `agent-schema`:

- CLI parser definitions.
- React/Tauri UI rendering.
- MCP client calls.
- registry fetch behavior.
- install-plan policy.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-schema/src/lib.rs` - public exports for normalized schema, fields, validation, and errors.
- Create: `crates/agent-schema/src/error.rs` - schema error type and stable kind strings.
- Create: `crates/agent-schema/src/field.rs` - normalized field DTOs.
- Create: `crates/agent-schema/src/normalize.rs` - JSON Schema normalization.
- Create: `crates/agent-schema/src/validate.rs` - invocation payload validation.
- Add sidecar tests in: `crates/agent-schema/src/normalize.rs` (`#[cfg(test)] mod tests`) - required/default/enum/array/object tests.
- Add sidecar tests in: `crates/agent-schema/src/validate.rs` (`#[cfg(test)] mod tests`) - validation tests.

## Implementation Tasks

### Task 1: Normalize Action Parameter Metadata

**Files:**
- Create: `crates/agent-schema/src/normalize.rs`
- Test sidecar: `crates/agent-schema/src/normalize.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Read Lab schema generation and OpenAPI mapping.**

Run:

```bash
rg -n "action_schema|param_type|enum_literals|required|nullable" ../lab/crates/lab/src/mcp/server.rs ../lab/crates/lab/src/api/openapi.rs ../lab/crates/lab-apis/src/core.rs
```

Expected: AgentCast supports required fields, defaults, enums, arrays, nested object fallback, and unsupported schema classification.

- [ ] **Step 2: Write failing normalization tests.**

Add this `#[cfg(test)] mod tests` sidecar to `crates/agent-schema/src/normalize.rs`:

```rust
use super::*;
use serde_json::json;

#[test]
fn normalizes_required_string_field() {
    let schema = json!({
        "type": "object",
        "properties": {
            "path": {
                "type": "string",
                "description": "Path to read"
            }
        },
        "required": ["path"]
    });

    let normalized = normalize_schema(&schema).unwrap();
    assert_eq!(normalized.fields[0].name, "path");
    assert_eq!(normalized.fields[0].kind, FieldKind::String);
    assert!(normalized.fields[0].required);
}
```

- [ ] **Step 3: Export schema modules.**

Update `crates/agent-schema/src/lib.rs`:

```rust
mod error;
mod field;
mod normalize;
mod validate;

pub use error::{SchemaError, SchemaResult};
pub use field::{FieldKind, NormalizedField, NormalizedSchema};
pub use normalize::normalize_schema;
pub use validate::validate_payload;
```

- [ ] **Step 4: Implement field DTOs and error type.**

Create `crates/agent-schema/src/error.rs`:

```rust
use thiserror::Error;

pub type SchemaResult<T> = Result<T, SchemaError>;

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("unsupported schema: {0}")]
    Unsupported(String),
    #[error("validation failed: {0}")]
    Validation(String),
}
```

Create `crates/agent-schema/src/field.rs`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedSchema {
    pub fields: Vec<NormalizedField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedField {
    pub name: String,
    pub kind: FieldKind,
    pub required: bool,
    pub description: Option<String>,
    pub default: Option<serde_json::Value>,
    pub enum_values: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Unsupported,
}
```

- [ ] **Step 5: Implement normalizer.**

Create `crates/agent-schema/src/normalize.rs`:

```rust
use std::collections::BTreeSet;

use serde_json::Value;

use crate::{FieldKind, NormalizedField, NormalizedSchema, SchemaError, SchemaResult};

pub fn normalize_schema(schema: &Value) -> SchemaResult<NormalizedSchema> {
    let properties = schema
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| SchemaError::Unsupported("schema must be an object with properties".into()))?;
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();

    let fields = properties
        .iter()
        .map(|(name, property)| NormalizedField {
            name: name.clone(),
            kind: match property.get("type").and_then(Value::as_str) {
                Some("string") => FieldKind::String,
                Some("number") => FieldKind::Number,
                Some("integer") => FieldKind::Integer,
                Some("boolean") => FieldKind::Boolean,
                Some("array") => FieldKind::Array,
                Some("object") => FieldKind::Object,
                _ => FieldKind::Unsupported,
            },
            required: required.contains(name),
            description: property.get("description").and_then(Value::as_str).map(str::to_string),
            default: property.get("default").cloned(),
            enum_values: property
                .get("enum")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect(),
        })
        .collect();

    Ok(NormalizedSchema { fields })
}
```

### Task 2: Add Payload Validation

**Files:**
- Create: `crates/agent-schema/src/validate.rs`
- Test sidecar: `crates/agent-schema/src/validate.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Write failing validation test.**

Add this `#[cfg(test)] mod tests` sidecar to `crates/agent-schema/src/validate.rs`:

```rust
use super::*;
use serde_json::json;

#[test]
fn validation_rejects_missing_required_field() {
    let schema = NormalizedSchema {
        fields: vec![NormalizedField {
            name: "path".into(),
            kind: FieldKind::String,
            required: true,
            description: None,
            default: None,
            enum_values: vec![],
        }],
    };

    let err = validate_payload(&schema, &json!({})).unwrap_err();
    assert!(err.to_string().contains("path"));
}
```

- [ ] **Step 2: Implement validation helper.**

Create `crates/agent-schema/src/validate.rs`:

```rust
use serde_json::Value;

use crate::{NormalizedSchema, SchemaError, SchemaResult};

pub fn validate_payload(schema: &NormalizedSchema, payload: &Value) -> SchemaResult<()> {
    let object = payload
        .as_object()
        .ok_or_else(|| SchemaError::Validation("payload must be an object".into()))?;
    for field in &schema.fields {
        if field.required && !object.contains_key(&field.name) {
            return Err(SchemaError::Validation(format!(
                "missing required field `{}`",
                field.name
            )));
        }
    }
    Ok(())
}
```

### Task 3: Verify Full Schema Extraction

**Files:**
- Test sidecar: `crates/agent-schema/src/*.rs` (`#[cfg(test)] mod tests`)
- Read: `docs/plans/extract-crates/agent-schema.md`

- [ ] **Step 1: Run focused schema tests.**

Run:

```bash
cargo test -p agent-schema
```

Expected: schema normalization and validation tests pass without launching MCP servers.

- [ ] **Step 2: Scan for rendering/runtime leakage.**

Run:

```bash
rg -n "clap|axum|react|component|rmcp|call_tool|html" crates/agent-schema
```

Expected: no output.
