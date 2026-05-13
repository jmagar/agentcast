Monitoring - Claude Code Docs
## > Documentation Index
> Fetch the complete documentation index at:
[> https://code.claude.com/docs/llms.txt
](https://code.claude.com/docs/llms.txt)
> Use this file to discover all available pages before exploring further.
Track Claude Code usage, costs, and tool activity across your organization by exporting telemetry data through OpenTelemetry (OTel). Claude Code exports metrics as time series data via the standard metrics protocol, events via the logs/events protocol, and optionally distributed traces via the [traces protocol](#traces-beta). Configure your metrics, logs, and traces backends to match your monitoring requirements.
##
[​
](#quick-start)
Quick start
Configure OpenTelemetry using environment variables:
```
`# 1. Enable telemetry
export CLAUDE\_CODE\_ENABLE\_TELEMETRY=1
# 2. Choose exporters (both are optional - configure only what you need)
export OTEL\_METRICS\_EXPORTER=otlp # Options: otlp, prometheus, console, none
export OTEL\_LOGS\_EXPORTER=otlp # Options: otlp, console, none
# 3. Configure OTLP endpoint (for OTLP exporter)
export OTEL\_EXPORTER\_OTLP\_PROTOCOL=grpc
export OTEL\_EXPORTER\_OTLP\_ENDPOINT=http://localhost:4317
# 4. Set authentication (if required)
export OTEL\_EXPORTER\_OTLP\_HEADERS="Authorization=Bearer your-token"
# 5. For debugging: reduce export intervals
export OTEL\_METRIC\_EXPORT\_INTERVAL=10000 # 10 seconds (default: 60000ms)
export OTEL\_LOGS\_EXPORT\_INTERVAL=5000 # 5 seconds (default: 5000ms)
# 6. Run Claude Code
claude
`
```
The default export intervals are 60 seconds for metrics and 5 seconds for logs. During setup, you may want to use shorter intervals for debugging purposes. Remember to reset these for production use.
For full configuration options, see the [OpenTelemetry specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#configuration-options).
##
[​
](#administrator-configuration)
Administrator configuration
Administrators can configure OpenTelemetry settings for all users through the [managed settings file](/docs/en/settings#settings-files). This allows for centralized control of telemetry settings across an organization. See the [settings precedence](/docs/en/settings#settings-precedence) for more information about how settings are applied.
Example managed settings configuration:
```
`{
"env": {
"CLAUDE\_CODE\_ENABLE\_TELEMETRY": "1",
"OTEL\_METRICS\_EXPORTER": "otlp",
"OTEL\_LOGS\_EXPORTER": "otlp",
"OTEL\_EXPORTER\_OTLP\_PROTOCOL": "grpc",
"OTEL\_EXPORTER\_OTLP\_ENDPOINT": "http://collector.example.com:4317",
"OTEL\_EXPORTER\_OTLP\_HEADERS": "Authorization=Bearer example-token"
}
}
`
```
Managed settings can be distributed via MDM (Mobile Device Management) or other device management solutions. Environment variables defined in the managed settings file have high precedence and cannot be overridden by users.
Claude Code does not pass `OTEL\_\*` environment variables to the subprocesses it spawns, including the Bash tool, hooks, MCP servers, and language servers. An OpenTelemetry-instrumented application that you run through the Bash tool does not inherit Claude Code’s exporter endpoint or headers, so set those variables directly in the command if that application needs to export its own telemetry.
##
[​
](#configuration-details)
Configuration details
###
[​
](#common-configuration-variables)
Common configuration variables
|Environment Variable|Description|Example Values|
|`CLAUDE\_CODE\_ENABLE\_TELEMETRY`|Enables telemetry collection (required)|`1`|
|`OTEL\_METRICS\_EXPORTER`|Metrics exporter types, comma-separated. Use `none` to disable|`console`, `otlp`, `prometheus`, `none`|
|`OTEL\_LOGS\_EXPORTER`|Logs/events exporter types, comma-separated. Use `none` to disable|`console`, `otlp`, `none`|
|`OTEL\_EXPORTER\_OTLP\_PROTOCOL`|Protocol for OTLP exporter, applies to all signals|`grpc`, `http/json`, `http/protobuf`|
|`OTEL\_EXPORTER\_OTLP\_ENDPOINT`|OTLP collector endpoint for all signals|`http://localhost:4317`|
|`OTEL\_EXPORTER\_OTLP\_METRICS\_PROTOCOL`|Protocol for metrics, overrides general setting|`grpc`, `http/json`, `http/protobuf`|
|`OTEL\_EXPORTER\_OTLP\_METRICS\_ENDPOINT`|OTLP metrics endpoint, overrides general setting|`http://localhost:4318/v1/metrics`|
|`OTEL\_EXPORTER\_OTLP\_LOGS\_PROTOCOL`|Protocol for logs, overrides general setting|`grpc`, `http/json`, `http/protobuf`|
|`OTEL\_EXPORTER\_OTLP\_LOGS\_ENDPOINT`|OTLP logs endpoint, overrides general setting|`http://localhost:4318/v1/logs`|
|`OTEL\_EXPORTER\_OTLP\_HEADERS`|Authentication headers for OTLP|`Authorization=Bearer token`|
|`OTEL\_METRIC\_EXPORT\_INTERVAL`|Export interval in milliseconds (default: 60000)|`5000`, `60000`|
|`OTEL\_LOGS\_EXPORT\_INTERVAL`|Logs export interval in milliseconds (default: 5000)|`1000`, `10000`|
|`OTEL\_LOG\_USER\_PROMPTS`|Enable logging of user prompt content (default: disabled)|`1` to enable|
|`OTEL\_LOG\_TOOL\_DETAILS`|Enable logging of tool parameters and input arguments in tool events and trace span attributes: Bash commands, MCP server and tool names, skill names, and tool input. Also enables custom, plugin, and MCP command names on `user\_prompt` events (default: disabled)|`1` to enable|
|`OTEL\_LOG\_TOOL\_CONTENT`|Enable logging of tool input and output content in span events (default: disabled). Requires [tracing](#traces-beta). Content is truncated at 60 KB|`1` to enable|
|`OTEL\_LOG\_RAW\_API\_BODIES`|Emit the full Anthropic Messages API request and response JSON as `api\_request\_body` / `api\_response\_body` log events (default: disabled). Bodies include the entire conversation history. Enabling this implies consent to everything `OTEL\_LOG\_USER\_PROMPTS`, `OTEL\_LOG\_TOOL\_DETAILS`, and `OTEL\_LOG\_TOOL\_CONTENT` would reveal|`1` for inline bodies truncated at 60 KB, or `file:\<dir\>` for untruncated bodies on disk with a `body\_ref` pointer in the event|
|`OTEL\_EXPORTER\_OTLP\_METRICS\_TEMPORALITY\_PREFERENCE`|Metrics temporality preference (default: `delta`). Set to `cumulative` if your backend expects cumulative temporality|`delta`, `cumulative`|
|`CLAUDE\_CODE\_OTEL\_HEADERS\_HELPER\_DEBOUNCE\_MS`|Interval for refreshing dynamic headers (default: 1740000ms / 29 minutes)|`900000`|
###
[​
](#mtls-authentication)
mTLS authentication
How you configure client certificates for the OTLP exporter depends on the OTLP protocol in use for that signal, set via `OTEL\_EXPORTER\_OTLP\_PROTOCOL` or the per-signal override. The same configuration applies to metrics, logs, and traces.
|Protocol|Client certificate variables|Trust the collector’s CA with|
|`http/protobuf`, `http/json`|`CLAUDE\_CODE\_CLIENT\_CERT`, `CLAUDE\_CODE\_CLIENT\_KEY`, and optionally `CLAUDE\_CODE\_CLIENT\_KEY\_PASSPHRASE`. See [Network configuration](/docs/en/network-config#mtls-authentication)|`NODE\_EXTRA\_CA\_CERTS`|
|`grpc`|`OTEL\_EXPORTER\_OTLP\_CLIENT\_KEY` and `OTEL\_EXPORTER\_OTLP\_CLIENT\_CERTIFICATE`, or the per-signal variants such as `OTEL\_EXPORTER\_OTLP\_METRICS\_CLIENT\_KEY` to use a different certificate per signal|`OTEL\_EXPORTER\_OTLP\_CERTIFICATE`|
For `grpc`, the OpenTelemetry SDK reads the standard OTLP variables directly, so existing configurations that set the per-signal metrics variables continue to work.
###
[​
](#metrics-cardinality-control)
Metrics cardinality control
The following environment variables control which attributes are included in metrics to manage cardinality:
|Environment Variable|Description|Default Value|Example to Disable|
|`OTEL\_METRICS\_INCLUDE\_SESSION\_ID`|Include session.id attribute in metrics|`true`|`false`|
|`OTEL\_METRICS\_INCLUDE\_VERSION`|Include app.version attribute in metrics|`false`|`true`|
|`OTEL\_METRICS\_INCLUDE\_ACCOUNT\_UUID`|Include user.account\_uuid and user.account\_id attributes in metrics|`true`|`false`|
These variables help control the cardinality of metrics, which affects storage requirements and query performance in your metrics backend. Lower cardinality generally means better performance and lower storage costs but less granular data for analysis.
###
[​
](#traces-beta)
Traces (beta)
Distributed tracing exports spans that link each user prompt to the API requests and tool executions it triggers, so you can view a full request as a single trace in your tracing backend.
Tracing is off by default. To enable it, set both `CLAUDE\_CODE\_ENABLE\_TELEMETRY=1` and `CLAUDE\_CODE\_ENHANCED\_TELEMETRY\_BETA=1`, then set `OTEL\_TRACES\_EXPORTER` to choose where spans are sent. Traces reuse the [common OTLP configuration](#common-configuration-variables) for endpoint, protocol, headers, and [mTLS](#mtls-authentication).
|Environment Variable|Description|Example Values|
|`CLAUDE\_CODE\_ENHANCED\_TELEMETRY\_BETA`|Enable span tracing (required). `ENABLE\_ENHANCED\_TELEMETRY\_BETA` is also accepted|`1`|
|`OTEL\_TRACES\_EXPORTER`|Traces exporter types, comma-separated. Use `none` to disable|`console`, `otlp`, `none`|
|`OTEL\_EXPORTER\_OTLP\_TRACES\_PROTOCOL`|Protocol for traces, overrides `OTEL\_EXPORTER\_OTLP\_PROTOCOL`|`grpc`, `http/json`, `http/protobuf`|
|`OTEL\_EXPORTER\_OTLP\_TRACES\_ENDPOINT`|OTLP traces endpoint, overrides `OTEL\_EXPORTER\_OTLP\_ENDPOINT`|`http://localhost:4318/v1/traces`|
|`OTEL\_TRACES\_EXPORT\_INTERVAL`|Span batch export interval in milliseconds (default: 5000)|`1000`, `10000`|
Spans redact user prompt text, tool input details, and tool content by default. Set `OTEL\_LOG\_USER\_PROMPTS=1`, `OTEL\_LOG\_TOOL\_DETAILS=1`, and `OTEL\_LOG\_TOOL\_CONTENT=1` to include them.
When tracing is active, Bash and PowerShell subprocesses automatically inherit a `TRACEPARENT` environment variable containing the W3C trace context of the active tool execution span. This lets any subprocess that reads `TRACEPARENT` parent its own spans under the same trace, enabling end-to-end distributed tracing through scripts and commands that Claude runs.
In Agent SDK and non-interactive sessions started with `-p`, Claude Code also reads `TRACEPARENT` and `TRACESTATE` from its own environment when starting each interaction span. This lets an embedding process pass its active W3C trace context into the subprocess so Claude Code’s spans appear as children of the caller’s distributed trace. Interactive sessions ignore inbound `TRACEPARENT` to avoid accidentally inheriting ambient values from CI or container environments.
####
[​
](#span-hierarchy)
Span hierarchy
Each user prompt starts a `claude\_code.interaction` root span. API calls, tool calls, and hook executions are recorded as its children. Tool spans have two child spans of their own: one for the time spent waiting on a permission decision and one for the execution itself. When the Task tool spawns a subagent, the subagent’s API and tool spans nest under the parent’s `claude\_code.tool` span.
```
`claude\_code.interaction
├── claude\_code.llm\_request
├── claude\_code.hook (requires detailed beta tracing)
└── claude\_code.tool
├── claude\_code.tool.blocked\_on\_user
├── claude\_code.tool.execution
└── (Task tool) subagent claude\_code.llm\_request / claude\_code.tool spans
`
```
In Agent SDK and `claude -p` sessions, `claude\_code.interaction` itself becomes a child of the caller’s span when `TRACEPARENT` is set in the environment.
####
[​
](#span-attributes)
Span attributes
Every span carries the [standard attributes](#standard-attributes) plus a `span.type` attribute matching its name. The tables below list the additional attributes set on each span. The `llm\_request`, `tool.execution`, and `hook` spans set OpenTelemetry status `ERROR` when they record a failure; the other spans always end with status `UNSET`.
**`claude\_code.interaction`**
|Attribute|Description|Gated by|
|`user\_prompt`|Prompt text. Value is `\<REDACTED\>` unless the gate is set|`OTEL\_LOG\_USER\_PROMPTS`|
|`user\_prompt\_length`|Prompt length in characters||
|`interaction.sequence`|1-based counter of interactions in this session||
|`interaction.duration\_ms`|Wall-clock duration of the turn||
**`claude\_code.llm\_request`**
|Attribute|Description|Gated by|
|`model`|Model identifier||
|`gen\_ai.system`|Always `anthropic`. OpenTelemetry GenAI semantic convention||
|`gen\_ai.request.model`|Same value as `model`. OpenTelemetry GenAI semantic convention||
|`query\_source`|Subsystem that issued the request, such as `repl\_main\_thread` or a subagent name||
|`agent\_id`|Identifier of the subagent or teammate that issued the request. Absent on the main session||
|`parent\_agent\_id`|Identifier of the agent that spawned this one. Absent for the main session and for agents spawned directly from it||
|`speed`|`fast` or `normal`||
|`llm\_request.context`|`interaction`, `tool`, or `standalone` depending on the parent span||
|`duration\_ms`|Wall-clock duration including retries||
|`ttft\_ms`|Time to first token in milliseconds||
|`input\_tokens`|Input token count from the API usage block||
|`output\_tokens`|Output token count||
|`cache\_read\_tokens`|Tokens read from prompt cache||
|`cache\_creation\_tokens`|Tokens written to prompt cache||
|`request\_id`|Anthropic API request ID from the `request-id` response header||
|`gen\_ai.response.id`|Same value as `request\_id`. OpenTelemetry GenAI semantic convention||
|`client\_request\_id`|Client-generated `x-client-request-id` of the final attempt||
|`attempt`|Total attempts made for this request||
|`success`|`true` or `false`||
|`status\_code`|HTTP status code when the request failed||
|`error`|Error message when the request failed||
|`response.has\_tool\_call`|`true` when the response contained tool-use blocks||
|`stop\_reason`|API response `stop\_reason`, such as `end\_turn`, `tool\_use`, `max\_tokens`, `stop\_sequence`, `pause\_turn`, or `refusal`||
|`gen\_ai.response.finish\_reasons`|Same value as `stop\_reason`, wrapped in a string array. OpenTelemetry GenAI semantic convention||
Each retry attempt is also recorded as a `gen\_ai.request.attempt` span event with `attempt` and `client\_request\_id` attributes.
**`claude\_code.tool`**
|Attribute|Description|Gated by|
|`tool\_name`|Tool name||
|`duration\_ms`|Wall-clock duration including permission wait and execution||
|`result\_tokens`|Approximate token size of the tool result||
|`file\_path`|Target file path for Read, Edit, and Write tools|`OTEL\_LOG\_TOOL\_DETAILS`|
|`full\_command`|Command string for the Bash tool|`OTEL\_LOG\_TOOL\_DETAILS`|
|`skill\_name`|Skill name for the Skill tool|`OTEL\_LOG\_TOOL\_DETAILS`|
|`subagent\_type`|Subagent type for the Task tool|`OTEL\_LOG\_TOOL\_DETAILS`|
When `OTEL\_LOG\_TOOL\_CONTENT=1`, this span also records a `tool.output` span event whose attributes contain the tool’s input and output bodies, truncated at 60 KB per attribute.
**`claude\_code.tool.blocked\_on\_user`**
|Attribute|Description|Gated by|
|`duration\_ms`|Time spent waiting for the permission decision||
|`decision`|`accept` or `reject`||
|`source`|Decision source, matching the [Tool decision event](#tool-decision-event)||
**`claude\_code.tool.execution`**
|Attribute|Description|Gated by|
|`duration\_ms`|Time spent running the tool body||
|`success`|`true` or `false`||
|`error`|Error category string when execution failed, such as `Error:ENOENT` or `ShellError`. Contains the full error message instead when the gate is set|`OTEL\_LOG\_TOOL\_DETAILS`|
**`claude\_code.hook`**
This span is emitted only when detailed beta tracing is active, which requires `ENABLE\_BETA\_TRACING\_DETAILED=1` and `BETA\_TRACING\_ENDPOINT` in addition to the trace exporter configuration above. In interactive CLI sessions, this also requires your organization to be allowlisted for the feature. Agent SDK and non-interactive `-p` sessions are not gated. It is not emitted when only `CLAUDE\_CODE\_ENHANCED\_TELEMETRY\_BETA` is set.
|Attribute|Description|Gated by|
|`hook\_event`|Hook event type, such as `PreToolUse`||
|`hook\_name`|Full hook name, such as `PreToolUse:Write`||
|`num\_hooks`|Number of matching hook commands executed||
|`hook\_definitions`|JSON-serialized hook configuration|`OTEL\_LOG\_TOOL\_DETAILS`|
|`duration\_ms`|Wall-clock duration of all matching hooks||
|`num\_success`|Count of hooks that completed successfully||
|`num\_blocking`|Count of hooks that returned a blocking decision||
|`num\_non\_blocking\_error`|Count of hooks that failed without blocking||
|`num\_cancelled`|Count of hooks cancelled before completion||
Additional content-bearing attributes such as `new\_context`, `system\_prompt\_preview`, `user\_system\_prompt`, `tool\_input`, and `response.model\_output` are emitted only when detailed beta tracing is active. They are not part of the stable span schema. `user\_system\_prompt` additionally requires `OTEL\_LOG\_USER\_PROMPTS=1`. It carries only the system prompt text you provide via the `systemPrompt` SDK option or `--system-prompt` and `--append-system-prompt` flags, truncated at 60 KB, and is emitted once per session rather than per request.
###
[​
](#dynamic-headers)
Dynamic headers
For enterprise environments that require dynamic authentication, you can configure a script to generate headers dynamically. Dynamic headers apply only to the `http/protobuf` and `http/json` protocols. The `grpc` exporter uses only the static `OTEL\_EXPORTER\_OTLP\_HEADERS` value.
####
[​
](#settings-configuration)
Settings configuration
Add to your `.claude/settings.json`:
```
`{
"otelHeadersHelper": "/bin/generate\_opentelemetry\_headers.sh"
}
`
```
####
[​
](#script-requirements)
Script requirements
The script must output valid JSON with string key-value pairs representing HTTP headers:
```
`#!/bin/bash
# Example: Multiple headers
echo "{\\"Authorization\\": \\"Bearer $(get-token.sh)\\", \\"X-API-Key\\": \\"$(get-api-key.sh)\\"}"
`
```
####
[​
](#refresh-behavior)
Refresh behavior
The headers helper script runs at startup and periodically thereafter to support token refresh. By default, the script runs every 29 minutes. Customize the interval with the `CLAUDE\_CODE\_OTEL\_HEADERS\_HELPER\_DEBOUNCE\_MS` environment variable.
###
[​
](#multi-team-organization-support)
Multi-team organization support
Organizations with multiple teams or departments can add custom attributes to distinguish between different groups using the `OTEL\_RESOURCE\_ATTRIBUTES` environment variable:
```
`# Add custom attributes for team identification
export OTEL\_RESOURCE\_ATTRIBUTES="department=engineering,team.id=platform,cost\_center=eng-123"
`
```
These custom attributes will be included in all metrics and events, allowing you to:
* Filter metrics by team or department
* Track costs per cost center
* Create team-specific dashboards
* Set up alerts for specific teams
**Important formatting requirements for OTEL\_RESOURCE\_ATTRIBUTES:**The `OTEL\_RESOURCE\_ATTRIBUTES` environment variable uses comma-separated key=value pairs with strict formatting requirements:
* **No spaces allowed**: Values cannot contain spaces. For example, `user.organizationName=My Company` is invalid
* **Format**: Must be comma-separated key=value pairs: `key1=value1,key2=value2`
* **Allowed characters**: Only US-ASCII characters excluding control characters, whitespace, double quotes, commas, semicolons, and backslashes
* **Special characters**: Characters outside the allowed range must be percent-encoded
**Examples:**
```
`# ❌ Invalid - contains spaces
export OTEL\_RESOURCE\_ATTRIBUTES="org.name=John's Organization"
# ✅ Valid - use underscores or camelCase instead
export OTEL\_RESOURCE\_ATTRIBUTES="org.name=Johns\_Organization"
export OTEL\_RESOURCE\_ATTRIBUTES="org.name=JohnsOrganization"
# ✅ Valid - percent-encode special characters if needed
export OTEL\_RESOURCE\_ATTRIBUTES="org.name=John%27s%20Organization"
`
```
Note: wrapping values in quotes doesn’t escape spaces. For example, `org.name="My Company"` results in the literal value `"My Company"` (with quotes included), not `My Company`.
###
[​
](#example-configurations)
Example configurations
Set these environment variables before running `claude`. Each block shows a complete configuration for a different exporter or deployment scenario:
```
`# Console debugging (1-second intervals)
export CLAUDE\_CODE\_ENABLE\_TELEMETRY=1
export OTEL\_METRICS\_EXPORTER=console
export OTEL\_METRIC\_EXPORT\_INTERVAL=1000
# OTLP/gRPC
export CLAUDE\_CODE\_ENABLE\_TELEMETRY=1
export OTEL\_METRICS\_EXPORTER=otlp
export OTEL\_EXPORTER\_OTLP\_PROTOCOL=grpc
export OTEL\_EXPORTER\_OTLP\_ENDPOINT=http://localhost:4317
# Prometheus
export CLAUDE\_CODE\_ENABLE\_TELEMETRY=1
export OTEL\_METRICS\_EXPORTER=prometheus
# Multiple exporters
export CLAUDE\_CODE\_ENABLE\_TELEMETRY=1
export OTEL\_METRICS\_EXPORTER=console,otlp
export OTEL\_EXPORTER\_OTLP\_PROTOCOL=http/json
# Different endpoints/backends for metrics and logs
export CLAUDE\_CODE\_ENABLE\_TELEMETRY=1
export OTEL\_METRICS\_EXPORTER=otlp
export OTEL\_LOGS\_EXPORTER=otlp
export OTEL\_EXPORTER\_OTLP\_METRICS\_PROTOCOL=http/protobuf
export OTEL\_EXPORTER\_OTLP\_METRICS\_ENDPOINT=http://metrics.example.com:4318
export OTEL\_EXPORTER\_OTLP\_LOGS\_PROTOCOL=grpc
export OTEL\_EXPORTER\_OTLP\_LOGS\_ENDPOINT=http://logs.example.com:4317
# Metrics only (no events/logs)
export CLAUDE\_CODE\_ENABLE\_TELEMETRY=1
export OTEL\_METRICS\_EXPORTER=otlp
export OTEL\_EXPORTER\_OTLP\_PROTOCOL=grpc
export OTEL\_EXPORTER\_OTLP\_ENDPOINT=http://localhost:4317
# Events/logs only (no metrics)
export CLAUDE\_CODE\_ENABLE\_TELEMETRY=1
export OTEL\_LOGS\_EXPORTER=otlp
export OTEL\_EXPORTER\_OTLP\_PROTOCOL=grpc
export OTEL\_EXPORTER\_OTLP\_ENDPOINT=http://localhost:4317
`
```
##
[​
](#available-metrics-and-events)
Available metrics and events
###
[​
](#standard-attributes)
Standard attributes
All metrics and events share these standard attributes:
|Attribute|Description|Controlled By|
|`session.id`|Unique session identifier|`OTEL\_METRICS\_INCLUDE\_SESSION\_ID` (default: true)|
|`app.version`|Current Claude Code version|`OTEL\_METRICS\_INCLUDE\_VERSION` (default: false)|
|`organization.id`|Organization UUID (when authenticated)|Always included when available|
|`user.account\_uuid`|Account UUID (when authenticated)|`OTEL\_METRICS\_INCLUDE\_ACCOUNT\_UUID` (default: true)|
|`user.account\_id`|Account ID in tagged format matching Anthropic admin APIs (when authenticated), such as `user\_01BWBeN28...`|`OTEL\_METRICS\_INCLUDE\_ACCOUNT\_UUID` (default: true)|
|`user.id`|Anonymous device/installation identifier, generated per Claude Code installation|Always included|
|`user.email`|User email address (when authenticated via OAuth)|Always included when available|
|`terminal.type`|Terminal type, such as `iTerm.app`, `vscode`, `cursor`, or `tmux`|Always included when detected|
Events additionally include the following attributes. These are never attached to metrics because they would cause unbounded cardinality:
* `prompt.id`: UUID correlating a user prompt with all subsequent events until the next prompt. See [Event correlation attributes](#event-correlation-attributes).
* `workspace.host\_paths`: host workspace directories selected in the desktop app, as a string array
###
[​
](#metrics)
Metrics
Claude Code exports the following metrics:
|Metric Name|Description|Unit|
|`claude\_code.session.count`|Count of CLI sessions started|count|
|`claude\_code.lines\_of\_code.count`|Count of lines of code modified|count|
|`claude\_code.pull\_request.count`|Number of pull requests created|count|
|`claude\_code.commit.count`|Number of git commits created|count|
|`claude\_code.cost.usage`|Cost of the Claude Code session|USD|
|`claude\_code.token.usage`|Number of tokens used|tokens|
|`claude\_code.code\_edit\_tool.decision`|Count of code editing tool permission decisions|count|
|`claude\_code.active\_time.total`|Total active time in seconds|s|
###
[​
](#metric-details)
Metric details
Each metric includes the standard attributes listed above. Metrics with additional context-specific attributes are noted below.
####
[​
](#session-counter)
Session counter
Incremented at the start of each session.
**Attributes**:
* All [standard attributes](#standard-attributes)
* `start\_type`: How the session was started. One of `"fresh"`, `"resume"`, or `"continue"`
####
[​
](#lines-of-code-counter)
Lines of code counter
Incremented when code is added or removed.
**Attributes**:
* All [standard attributes](#standard-attributes)
* `type`: (`"added"`, `"removed"`)
####
[​
](#pull-request-counter)
Pull request counter
Incremented when Claude Code creates a pull request or merge request through a shell command or an MCP tool.
**Attributes**:
* All [standard attributes](#standard-attributes)
####
[​
](#commit-counter)
Commit counter
Incremented when creating git commits via Claude Code.
**Attributes**:
* All [standard attributes](#standard-attributes)
####
[​
](#cost-counter)
Cost counter
Incremented after each API request.
**Attributes**:
* All [standard attributes](#standard-attributes)
* `model`: Model identifier (for example, “claude-sonnet-4-6”)
* `query\_source`: Category of the subsystem that issued the request. One of `"main"`, `"subagent"`, or `"auxiliary"`
* `speed`: `"fast"` when the request used fast mode. Absent otherwise
* `effort`: [Effort level](/docs/en/model-config#adjust-effort-level) applied to the request: `"low"`, `"medium"`, `"high"`, `"xhigh"`, or `"max"`. Absent when the model does not support effort.
* `agent.name`: Subagent type that issued the request. Built-in agent names and agents from official-marketplace plugins appear verbatim. Other user-defined agent names are replaced with `"custom"`. Absent when the request was not issued by a named subagent type.
* `skill.name`: Skill active for the request, set by the Skill tool, a `/` command, or inherited by a spawned subagent. Built-in, bundled, user-defined, and official-marketplace plugin skill names appear verbatim. Third-party plugin skill names are replaced with `"third-party"`. Absent when no skill is active.
* `plugin.name`: Owning plugin when the active skill or subagent is provided by a plugin. Official-marketplace plugin names appear verbatim. Third-party plugin names are replaced with `"third-party"`. Absent when neither the skill nor the subagent has an owning plugin.
* `marketplace.name`: Marketplace the owning plugin was installed from. Only emitted for official-marketplace plugins. Absent otherwise.
####
[​
](#token-counter)
Token counter
Incremented after each API request.
**Attributes**:
* All [standard attributes](#standard-attributes)
* `type`: (`"input"`, `"output"`, `"cacheRead"`, `"cacheCreation"`)
* `model`: Model identifier (for example, “claude-sonnet-4-6”)
* `query\_source`: Category of the subsystem that issued the request. One of `"main"`, `"subagent"`, or `"auxiliary"`
* `speed`: `"fast"` when the request used fast mode. Absent otherwise
* `effort`: [Effort level](/docs/en/model-config#adjust-effort-level) applied to the request. See [Cost counter](#cost-counter) for details.
* `agent.name`, `skill.name`, `plugin.name`, `marketplace.name`: Skill, plugin, and agent attribution for the request. See [Cost counter](#cost-counter) for definitions and redaction behavior.
####
[​
](#code-edit-tool-decision-counter)
Code edit tool decision counter
Incremented when user accepts or rejects Edit, Write, or NotebookEdit tool usage.
**Attributes**:
* All [standard attributes](#standard-attributes)
* `tool\_name`: Tool name (`"Edit"`, `"Write"`, `"NotebookEdit"`)
* `decision`: User decision (`"accept"`, `"reject"`)
* `source`: Where the decision came from. One of `"config"`, `"hook"`, `"user\_permanent"`, `"user\_temporary"`, `"user\_abort"`, or `"user\_reject"`. See the [Tool decision event](#tool-decision-event) for what each value means.
* `language`: Programming language of the edited file, such as `"TypeScript"`, `"Python"`, `"JavaScript"`, or `"Markdown"`. Returns `"unknown"` for unrecognized file extensions.
####
[​
](#active-time-counter)
Active time counter
Tracks actual time spent actively using Claude Code, excluding idle time. This metric is incremented during user interactions (typing, reading responses) and during CLI processing (tool execution, AI response generation).
**Attributes**:
* All [standard attributes](#standard-attributes)
* `type`: `"user"` for keyboard interactions, `"cli"` for tool execution and AI responses
###
[​
](#events)
Events
Claude Code exports the following events via OpenTelemetry logs/events (when `OTEL\_LOGS\_EXPORTER` is configured):
####
[​
](#event-correlation-attributes)
Event correlation attributes
When a user submits a prompt, Claude Code may make multiple API calls and run several tools. The `prompt.id` attribute lets you tie all of those events back to the single prompt that triggered them.
|Attribute|Description|
|`prompt.id`|UUID v4 identifier linking all events produced while processing a single user prompt|
To trace all activity triggered by a single prompt, filter your events by a specific `prompt.id` value. This returns the user\_prompt event, any api\_request events, and any tool\_result events that occurred while processing that prompt.
`prompt.id` is intentionally excluded from metrics because each prompt generates a unique ID, which would create an ever-growing number of time series. Use it for event-level analysis and audit trails only.
####
[​
](#user-prompt-event)
User prompt event
Logged when a user submits a prompt.
**Event Name**: `claude\_code.user\_prompt`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"user\_prompt"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `prompt\_length`: Length of the prompt
* `prompt`: Prompt content (redacted by default, enable with `OTEL\_LOG\_USER\_PROMPTS=1`)
* `command\_name`: Command name when the prompt invokes one. Built-in and bundled command names such as `compact` or `debug` are emitted as-is; aliases such as `reset` emit as typed rather than the canonical name. Custom, plugin, and MCP command names collapse to `custom` or `mcp` unless `OTEL\_LOG\_TOOL\_DETAILS=1` is set
* `command\_source`: Origin of the command when present: `builtin`, `custom`, or `mcp`. Plugin-provided commands report as `custom`
####
[​
](#tool-result-event)
Tool result event
Logged when a tool completes execution.
**Event Name**: `claude\_code.tool\_result`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"tool\_result"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `tool\_name`: Name of the tool
* `tool\_use\_id`: Unique identifier for this tool invocation. Matches the `tool\_use\_id` passed to hooks, allowing correlation between OTel events and hook-captured data.
* `success`: `"true"` or `"false"`
* `duration\_ms`: Execution time in milliseconds
* `error\_type`: Error category string when the tool failed, such as `"Error:ENOENT"` or `"ShellError"`
* `error` (when `OTEL\_LOG\_TOOL\_DETAILS=1`): Full error message when the tool failed
* `decision\_type`: Either `"accept"` or `"reject"`
* `decision\_source`: Where the decision came from. One of `"config"`, `"hook"`, `"user\_permanent"`, `"user\_temporary"`, `"user\_abort"`, or `"user\_reject"`. See the [Tool decision event](#tool-decision-event) for what each value means.
* `tool\_input\_size\_bytes`: Size of the JSON-serialized tool input in bytes
* `tool\_result\_size\_bytes`: Size of the tool result in bytes
* `mcp\_server\_scope`: MCP server scope identifier (for MCP tools)
* `tool\_parameters` (when `OTEL\_LOG\_TOOL\_DETAILS=1`): JSON string containing tool-specific parameters:
* For Bash tool: includes `bash\_command`, `full\_command`, `timeout`, `description`, `dangerouslyDisableSandbox`, and `git\_commit\_id` (the commit SHA, when a `git commit` command succeeds)
* For MCP tools: includes `mcp\_server\_name`, `mcp\_tool\_name`
* For Skill tool: includes `skill\_name`
* For Task tool: includes `subagent\_type`
* `tool\_input` (when `OTEL\_LOG\_TOOL\_DETAILS=1`): JSON-serialized tool arguments. Individual values over 512 characters are truncated, and the full payload is bounded to \~4 K characters. Applies to all tools including MCP tools.
####
[​
](#api-request-event)
API request event
Logged for each API request to Claude.
**Event Name**: `claude\_code.api\_request`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"api\_request"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `model`: Model used (for example, “claude-sonnet-4-6”)
* `cost\_usd`: Estimated cost in USD
* `duration\_ms`: Request duration in milliseconds
* `input\_tokens`: Number of input tokens
* `output\_tokens`: Number of output tokens
* `cache\_read\_tokens`: Number of tokens read from cache
* `cache\_creation\_tokens`: Number of tokens used for cache creation
* `request\_id`: Anthropic API request ID from the response’s `request-id` header, such as `"req\_011..."`. Present only when the API returns one.
* `speed`: `"fast"` or `"normal"`, indicating whether fast mode was active
* `query\_source`: Subsystem that issued the request, such as `"repl\_main\_thread"`, `"compact"`, or a subagent name
* `effort`: [Effort level](/docs/en/model-config#adjust-effort-level) applied to the request: `"low"`, `"medium"`, `"high"`, `"xhigh"`, or `"max"`. Absent when the model does not support effort.
####
[​
](#api-error-event)
API error event
Logged when an API request to Claude fails.
**Event Name**: `claude\_code.api\_error`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"api\_error"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `model`: Model used (for example, “claude-sonnet-4-6”)
* `error`: Error message
* `status\_code`: HTTP status code as a number. Absent for non-HTTP errors such as connection failures.
* `duration\_ms`: Request duration in milliseconds
* `attempt`: Total number of attempts made, including the initial request (`1` means no retries occurred)
* `request\_id`: Anthropic API request ID from the response’s `request-id` header, such as `"req\_011..."`. Present only when the API returns one.
* `speed`: `"fast"` or `"normal"`, indicating whether fast mode was active
* `query\_source`: Subsystem that issued the request, such as `"repl\_main\_thread"`, `"compact"`, or a subagent name
* `effort`: [Effort level](/docs/en/model-config#adjust-effort-level) applied to the request. Absent when the model does not support effort.
####
[​
](#api-request-body-event)
API request body event
Logged for each API request attempt when `OTEL\_LOG\_RAW\_API\_BODIES` is set. One event is emitted per attempt, so retries with adjusted parameters each produce their own event.
**Event Name**: `claude\_code.api\_request\_body`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"api\_request\_body"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `body`: JSON-serialized Messages API request parameters (system prompt, messages, tools, etc.), truncated at 60 KB. Extended-thinking content in prior assistant turns is redacted. Emitted only in inline mode (`OTEL\_LOG\_RAW\_API\_BODIES=1`).
* `body\_ref`: Absolute path to a `\<dir\>/\<uuid\>.request.json` file containing the untruncated body. Emitted only in file mode (`OTEL\_LOG\_RAW\_API\_BODIES=file:\<dir\>`).
* `body\_length`: Untruncated body length. UTF-8 bytes when `OTEL\_LOG\_RAW\_API\_BODIES=file:\<dir\>`, or UTF-16 code units when `=1`
* `body\_truncated`: `"true"` when inline truncation occurred. Absent in file mode and when no truncation occurred.
* `model`: Model identifier from the request parameters
* `query\_source`: Subsystem that issued the request (for example, `"compact"`)
####
[​
](#api-response-body-event)
API response body event
Logged for each successful API response when `OTEL\_LOG\_RAW\_API\_BODIES` is set.
**Event Name**: `claude\_code.api\_response\_body`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"api\_response\_body"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `body`: JSON-serialized Messages API response (id, content blocks, usage, stop reason), truncated at 60 KB. Extended-thinking content is redacted. Emitted only in inline mode (`OTEL\_LOG\_RAW\_API\_BODIES=1`).
* `body\_ref`: Absolute path to a `\<dir\>/\<request\_id\>.response.json` file containing the untruncated body. Emitted only in file mode (`OTEL\_LOG\_RAW\_API\_BODIES=file:\<dir\>`).
* `body\_length`: Untruncated body length. UTF-8 bytes when `OTEL\_LOG\_RAW\_API\_BODIES=file:\<dir\>`, or UTF-16 code units when `=1`
* `body\_truncated`: `"true"` when inline truncation occurred. Absent in file mode and when no truncation occurred.
* `model`: Model identifier
* `query\_source`: Subsystem that issued the request
* `request\_id`: Anthropic API request ID from the response’s `request-id` header, such as `"req\_011..."`. Present only when the API returns one.
####
[​
](#tool-decision-event)
Tool decision event
Logged when a tool permission decision is made (accept/reject).
**Event Name**: `claude\_code.tool\_decision`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"tool\_decision"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `tool\_name`: Name of the tool (for example, “Read”, “Edit”, “Write”, “NotebookEdit”)
* `tool\_use\_id`: Unique identifier for this tool invocation. Matches the `tool\_use\_id` passed to hooks, allowing correlation between OTel events and hook-captured data.
* `decision`: Either `"accept"` or `"reject"`
* `source`: Where the decision came from:
* `"config"`: Decided automatically without prompting, based on project settings, allow rules in the user’s personal settings, enterprise managed policy, `--allowedTools` or `--disallowedTools` flags, the active permission mode, a session-scoped grant from an earlier prompt in the same interactive CLI session, or because the tool is inherently safe. The event does not indicate which of these sources matched.
* `"hook"`: A `PreToolUse` or `PermissionRequest` hook returned the decision.
* `"user\_permanent"`: Emitted when the user chose “Yes, and don’t ask again for …” at a permission prompt, which saves an allow rule to their personal settings. In the interactive CLI this is emitted only for that choice itself; later calls that match the saved rule emit `"config"` instead. In Agent SDK or non-interactive `-p` sessions, both the initial choice and later rule matches emit `"user\_permanent"`. Treated as an accept.
* `"user\_temporary"`: Emitted when the user chose “Yes” at a permission prompt for a one-time approval, or chose one of the ”… during this session” options on a file edit or read prompt. In the interactive CLI this is emitted only for the choice itself; later calls allowed by that session-scoped grant emit `"config"` instead. In Agent SDK or non-interactive `-p` sessions, both the choice and later matches emit `"user\_temporary"`. Treated as an accept.
* `"user\_abort"`: Emitted when the user dismissed the permission prompt without answering. Treated as a reject.
* `"user\_reject"`: Emitted when the user chose “No” when prompted, or a call matched a deny rule in their personal settings. Treated as a reject.
####
[​
](#permission-mode-changed-event)
Permission mode changed event
Logged when the permission mode changes, for example from `Shift+Tab` cycling, exiting plan mode, or an auto mode gate check.
**Event Name**: `claude\_code.permission\_mode\_changed`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"permission\_mode\_changed"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `from\_mode`: The previous permission mode, for example `"default"`, `"plan"`, `"acceptEdits"`, `"auto"`, or `"bypassPermissions"`
* `to\_mode`: The new permission mode
* `trigger`: What caused the change. One of `"shift\_tab"`, `"exit\_plan\_mode"`, `"auto\_gate\_denied"`, or `"auto\_opt\_in"`. Absent when the transition originates from the SDK or bridge
####
[​
](#auth-event)
Auth event
Logged when `/login` or `/logout` completes.
**Event Name**: `claude\_code.auth`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"auth"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `action`: `"login"` or `"logout"`
* `success`: `"true"` or `"false"`
* `auth\_method`: Authentication method, such as `"oauth"`
* `error\_category`: Categorical error kind when the action failed. The raw error message is never included
* `status\_code`: HTTP status code as a string when the action failed with an HTTP error
####
[​
](#mcp-server-connection-event)
MCP server connection event
Logged when an MCP server connects, disconnects, or fails to connect.
**Event Name**: `claude\_code.mcp\_server\_connection`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"mcp\_server\_connection"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `status`: `"connected"`, `"failed"`, or `"disconnected"`
* `transport\_type`: Server transport, such as `"stdio"`, `"sse"`, or `"http"`
* `server\_scope`: Scope the server is configured at, such as `"user"`, `"project"`, or `"local"`
* `duration\_ms`: Connection attempt duration in milliseconds
* `error\_code`: Error code when the connection failed
* `server\_name` (when `OTEL\_LOG\_TOOL\_DETAILS=1`): Configured server name
* `error` (when `OTEL\_LOG\_TOOL\_DETAILS=1`): Full error message when the connection failed
####
[​
](#internal-error-event)
Internal error event
Logged when Claude Code catches an unexpected internal error. Only the error class name and an errno-style code are recorded. The error message and stack trace are never included. This event is not emitted when running against Bedrock, Vertex, or Foundry, or when `DISABLE\_ERROR\_REPORTING` is set.
**Event Name**: `claude\_code.internal\_error`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"internal\_error"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `error\_name`: Error class name, such as `"TypeError"` or `"SyntaxError"`
* `error\_code`: Node.js errno code such as `"ENOENT"` when present on the error
####
[​
](#plugin-installed-event)
Plugin installed event
Logged when a plugin finishes installing, from both the `claude plugin install` CLI command and the interactive `/plugin` UI.
**Event Name**: `claude\_code.plugin\_installed`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"plugin\_installed"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `marketplace.is\_official`: `"true"` if the marketplace is an official Anthropic marketplace, `"false"` otherwise
* `install.trigger`: `"cli"` or `"ui"`
* `plugin.name`: Name of the installed plugin. For third-party marketplaces this is included only when `OTEL\_LOG\_TOOL\_DETAILS=1`
* `plugin.version`: Plugin version when declared in the marketplace entry. For third-party marketplaces this is included only when `OTEL\_LOG\_TOOL\_DETAILS=1`
* `marketplace.name`: Marketplace the plugin was installed from. For third-party marketplaces this is included only when `OTEL\_LOG\_TOOL\_DETAILS=1`
####
[​
](#plugin-loaded-event)
Plugin loaded event
Logged once per enabled plugin at session start. Use this event to inventory which plugins are active across your fleet, as a complement to `plugin\_installed` which records the install action itself.
**Event Name**: `claude\_code.plugin\_loaded`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"plugin\_loaded"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `plugin.name`: name of the plugin. For plugins outside the official marketplace and built-in bundle the value is `"third-party"` unless `OTEL\_LOG\_TOOL\_DETAILS=1`
* `marketplace.name`: marketplace the plugin was installed from, when known. Redacted to `"third-party"` under the same condition as `plugin.name`
* `plugin.version`: version from the plugin manifest. Included only when the name is not redacted and the manifest declares a version
* `plugin.scope`: provenance category for the plugin: `"official"`, `"org"`, `"user-local"`, or `"default-bundle"`
* `enabled\_via`: how the plugin came to be enabled: `"default-enable"`, `"org-policy"`, `"seed-mount"`, or `"user-install"`
* `plugin\_id\_hash`: deterministic hash of the plugin name and marketplace, sent only to your configured exporter. Lets you count how many distinct third-party plugins are loaded across your fleet without recording their names
* `has\_hooks`: whether the plugin contributes hooks
* `has\_mcp`: whether the plugin contributes MCP servers
* `skill\_path\_count`: number of skill directories the plugin declares
* `command\_path\_count`: number of command directories the plugin declares
* `agent\_path\_count`: number of agent directories the plugin declares
####
[​
](#skill-activated-event)
Skill activated event
Logged when a skill is invoked, whether Claude calls it through the Skill tool or you run it as a `/` command.
**Event Name**: `claude\_code.skill\_activated`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"skill\_activated"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `skill.name`: Name of the skill. For user-defined and third-party plugin skills the value is the placeholder `"custom\_skill"` unless `OTEL\_LOG\_TOOL\_DETAILS=1`
* `invocation\_trigger`: How the skill was triggered (`"user-slash"`, `"claude-proactive"`, or `"nested-skill"`)
* `skill.source`: Where the skill was loaded from (for example, `"bundled"`, `"userSettings"`, `"projectSettings"`, `"plugin"`)
* `plugin.name` (when `OTEL\_LOG\_TOOL\_DETAILS=1` or the plugin is from an official marketplace): Name of the owning plugin when the skill is provided by a plugin
* `marketplace.name` (when `OTEL\_LOG\_TOOL\_DETAILS=1` or the plugin is from an official marketplace): Marketplace the owning plugin was installed from, when the skill is provided by a plugin
####
[​
](#at-mention-event)
At mention event
Logged when Claude Code resolves an `@`-mention in a prompt. Not every mention emits an event: early-exit paths such as permission denials, oversized files, PDF reference attachments, and directory listing failures return without logging.
**Event Name**: `claude\_code.at\_mention`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"at\_mention"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `mention\_type`: Type of mention (`"file"`, `"directory"`, `"agent"`, `"mcp\_resource"`)
* `success`: Whether the mention resolved successfully (`"true"` or `"false"`)
####
[​
](#api-retries-exhausted-event)
API retries exhausted event
Logged once when an API request fails after more than one attempt. Emitted alongside the final `api\_error` event.
**Event Name**: `claude\_code.api\_retries\_exhausted`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"api\_retries\_exhausted"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `model`: Model used
* `error`: Final error message
* `status\_code`: HTTP status code as a number. Absent for non-HTTP errors.
* `total\_attempts`: Total number of attempts made
* `total\_retry\_duration\_ms`: Total wall-clock time across all attempts
* `speed`: `"fast"` or `"normal"`
####
[​
](#hook-registered-event)
Hook registered event
Logged once per configured hook at session start. Use this event to inventory which hooks are active across your fleet, as a complement to the per-execution `hook\_execution\_start` and `hook\_execution\_complete` events.
**Event Name**: `claude\_code.hook\_registered`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"hook\_registered"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `hook\_event`: hook event type, such as `"PreToolUse"` or `"PostToolUse"`
* `hook\_type`: hook implementation type: `"command"`, `"prompt"`, `"mcp\_tool"`, `"http"`, or `"agent"`
* `hook\_source`: where the hook is defined: `"userSettings"`, `"projectSettings"`, `"localSettings"`, `"flagSettings"`, `"policySettings"`, or `"pluginHook"`
* `hook\_matcher` (when `OTEL\_LOG\_TOOL\_DETAILS=1`): the matcher string from the hook configuration, when one is set
* `plugin.name` (when `hook\_source` is `"pluginHook"`): name of the contributing plugin. For plugins outside the official marketplace and built-in bundle the value is `"third-party"` unless `OTEL\_LOG\_TOOL\_DETAILS=1`
* `plugin\_id\_hash` (when `hook\_source` is `"pluginHook"`): deterministic hash of the plugin name and marketplace, sent only to your configured exporter. Lets you count distinct contributing plugins without recording their names
####
[​
](#hook-execution-start-event)
Hook execution start event
Logged when one or more hooks begin executing for a hook event.
**Event Name**: `claude\_code.hook\_execution\_start`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"hook\_execution\_start"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `hook\_event`: Hook event type, such as `"PreToolUse"` or `"PostToolUse"`
* `hook\_name`: Full hook name including matcher, such as `"PreToolUse:Write"`
* `num\_hooks`: Number of matching hook commands
* `managed\_only`: `"true"` when only managed-policy hooks are permitted
* `hook\_source`: `"policySettings"` or `"merged"`
* `hook\_definitions`: JSON-serialized hook configuration. Included only when both detailed beta tracing and `OTEL\_LOG\_TOOL\_DETAILS=1` are enabled
####
[​
](#hook-execution-complete-event)
Hook execution complete event
Logged when all hooks for a hook event have finished.
**Event Name**: `claude\_code.hook\_execution\_complete`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"hook\_execution\_complete"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `hook\_event`: Hook event type
* `hook\_name`: Full hook name including matcher
* `num\_hooks`: Number of matching hook commands
* `num\_success`: Count that completed successfully
* `num\_blocking`: Count that returned a blocking decision
* `num\_non\_blocking\_error`: Count that failed without blocking
* `num\_cancelled`: Count cancelled before completion
* `total\_duration\_ms`: Wall-clock duration of all matching hooks
* `managed\_only`: `"true"` when only managed-policy hooks are permitted
* `hook\_source`: `"policySettings"` or `"merged"`
* `hook\_definitions`: JSON-serialized hook configuration. Included only when both detailed beta tracing and `OTEL\_LOG\_TOOL\_DETAILS=1` are enabled
####
[​
](#compaction-event)
Compaction event
Logged when conversation compaction completes.
**Event Name**: `claude\_code.compaction`
**Attributes**:
* All [standard attributes](#standard-attributes)
* `event.name`: `"compaction"`
* `event.timestamp`: ISO 8601 timestamp
* `event.sequence`: monotonically increasing counter for ordering events within a session
* `trigger`: `"auto"` or `"manual"`
* `success`: `"true"` or `"false"`
* `duration\_ms`: Compaction duration
* `pre\_tokens`: Approximate token count before compaction
* `post\_tokens`: Approximate token count after compaction
* `error`: Error message when compaction failed
##
[​
](#interpret-metrics-and-events-data)
Interpret metrics and events data
The exported metrics and events support a range of analyses:
###
[​
](#usage-monitoring)
Usage monitoring
|Metric|Analysis Opportunity|
|`claude\_code.token.usage`|Break down by `type` (input/output), user, team, model, `skill.name`, `plugin.name`, or `agent.name`|
|`claude\_code.session.count`|Track adoption and engagement over time|
|`claude\_code.lines\_of\_code.count`|Measure productivity by tracking code additions/removals|
|`claude\_code.commit.count` & `claude\_code.pull\_request.count`|Understand impact on development workflows|
###
[​
](#cost-monitoring)
Cost monitoring
The `claude\_code.cost.usage` metric helps with:
* Tracking usage trends across teams or individuals
* Identifying high-usage sessions for optimization
* Attributing spend to specific skills, plugins, or subagent types via the `skill.name`, `plugin.name`, and `agent.name` attributes
Cost metrics are approximations. For official billing data, refer to your API provider (Claude Console, Amazon Bedrock, or Google Cloud Vertex).
###
[​
](#alerting-and-segmentation)
Alerting and segmentation
Common alerts to consider:
* Cost spikes
* Unusual token consumption
* High session volume from specific users
All metrics can be segmented by `user.account\_uuid`, `user.account\_id`, `organization.id`, `session.id`, `model`, and `app.version`.
###
[​
](#detect-retry-exhaustion)
Detect retry exhaustion
Claude Code retries failed API requests internally and emits a single `claude\_code.api\_error` event only after it gives up, so the event itself is the terminal signal for that request. Intermediate retry attempts are not logged as separate events.
The `attempt` attribute on the event records how many attempts were made in total. A value greater than `CLAUDE\_CODE\_MAX\_RETRIES` (default `10`) indicates the request exhausted all retries on a transient error. A lower value indicates a non-retryable error such as a `400` response.
To distinguish a session that recovered from one that stalled, group events by `session.id` and check whether a later `api\_request` event exists after the error.
###
[​
](#event-analysis)
Event analysis
The event data provides detailed insights into Claude Code interactions:
**Tool Usage Patterns**: analyze tool result events to identify:
* Most frequently used tools
* Tool success rates
* Average tool execution times
* Error patterns by tool type
**Performance Monitoring**: track API request durations and tool execution times to identify performance bottlenecks.
##
[​
](#audit-security-events)
Audit security events
OpenTelemetry events are the audit data source for Claude Code activity. Every event carries identity attributes that tie tool calls, MCP activity, and permission decisions back to the user who triggered them, and the OTLP logs exporter can deliver these events to any Security Information and Event Management (SIEM) platform with an OTLP receiver or to an OpenTelemetry Collector that forwards to your SIEM.
###
[​
](#attribute-actions-to-users)
Attribute actions to users
The [standard attributes](#standard-attributes) on each event include the authenticated user’s identity: `user.email`, `user.account\_uuid`, `user.account\_id`, and `organization.id` when signed in with a Claude account, plus the installation-scoped `user.id` and the per-session `session.id`.
MCP tool calls, Bash commands, and file edits are therefore attributed to the developer who started the session. Claude Code does not act under a separate service account; the identity recorded on each event is the developer’s own Claude account.
When Claude Code authenticates with a direct API key, or against Bedrock, Vertex AI, or Microsoft Foundry, there is no Claude account in the session and only `user.id` and `session.id` are populated. In these deployments, attach user identity yourself with `OTEL\_RESOURCE\_ATTRIBUTES`, set per user through the [managed settings](#administrator-configuration) file or a launch wrapper:
```
`export OTEL\_RESOURCE\_ATTRIBUTES="enduser.id=jdoe@example.com,enduser.directory\_id=S-1-5-21-..."
`
```
###
[​
](#audit-mcp-activity)
Audit MCP activity
To capture MCP server activity with full call detail, enable the logs exporter and set `OTEL\_LOG\_TOOL\_DETAILS=1`. Each MCP operation then produces structured events that carry the server name, tool name, and call arguments alongside the standard identity attributes:
|Event|What it records for MCP|
|`mcp\_server\_connection`|Server connect, disconnect, and connection failure with `server\_name`, `transport\_type`, `server\_scope`, and error detail|
|`tool\_result`|Each MCP tool call with `tool\_name` and `mcp\_server\_scope`, a `tool\_parameters` payload containing `mcp\_server\_name` and `mcp\_tool\_name`, and a `tool\_input` payload containing the call arguments|
|`tool\_decision`|Whether the call was allowed or denied, and whether the decision came from config, a hook, or the user|
Without `OTEL\_LOG\_TOOL\_DETAILS`, `tool\_result` events still carry `tool\_name` and `mcp\_server\_scope` but omit the `mcp\_server\_name`/`mcp\_tool\_name` breakdown and the arguments, and `mcp\_server\_connection` events omit `server\_name` and the error message.
###
[​
](#map-security-questions-to-events)
Map security questions to events
When building detection rules, look up the signal you want to monitor and query your backend for the corresponding event and attributes:
|Signal|Event|Key attributes|
|Tool call allowed or denied, and by what|`tool\_decision`|`decision`, `source`, `tool\_name`|
|Permission mode escalation|`permission\_mode\_changed`|`from\_mode`, `to\_mode`, `trigger`|
|Policy hook blocked an action|`hook\_execution\_complete`|`hook\_event`, `num\_blocking`|
|Login, logout, and authentication failure|`auth`|`action`, `success`, `error\_category`|
|MCP server connect or failure|`mcp\_server\_connection`|`status`, `server\_name`, `error\_code`|
|Plugin installed and its source|`plugin\_installed`|`plugin.name`, `marketplace.name`, `marketplace.is\_official`|
|Commands run and files touched|`tool\_result` with `OTEL\_LOG\_TOOL\_DETAILS=1`|`tool\_parameters`, `tool\_input`|
Claude Code emits the raw event stream only. Anomaly detection, baselining, correlation across sessions, and alerting are the responsibility of your SIEM or observability backend.
###
[​
](#send-events-to-a-siem)
Send events to a SIEM
Point `OTEL\_EXPORTER\_OTLP\_LOGS\_ENDPOINT` at your SIEM’s OTLP receiver, or at an OpenTelemetry Collector that forwards to your SIEM’s native ingest API. The following managed-settings example exports events only, with full tool detail enabled for MCP and Bash auditing:
```
`{
"env": {
"CLAUDE\_CODE\_ENABLE\_TELEMETRY": "1",
"OTEL\_LOGS\_EXPORTER": "otlp",
"OTEL\_LOG\_TOOL\_DETAILS": "1",
"OTEL\_EXPORTER\_OTLP\_LOGS\_PROTOCOL": "http/protobuf",
"OTEL\_EXPORTER\_OTLP\_LOGS\_ENDPOINT": "https://siem.example.com:4318/v1/logs",
"OTEL\_EXPORTER\_OTLP\_HEADERS": "Authorization=Bearer your-siem-token"
}
}
`
```
##
[​
](#backend-considerations)
Backend considerations
Your choice of metrics, logs, and traces backends determines the types of analyses you can perform:
###
[​
](#for-metrics)
For metrics
* **Time series databases (for example, Prometheus)**: Rate calculations, aggregated metrics
* **Columnar stores (for example, ClickHouse)**: Complex queries, unique user analysis
* **Full-featured observability platforms (for example, Honeycomb, Datadog)**: Advanced querying, visualization, alerting
###
[​
](#for-events/logs)
For events/logs
* **Log aggregation systems (for example, Elasticsearch, Loki)**: Full-text search, log analysis
* **Columnar stores (for example, ClickHouse)**: Structured event analysis
* **Full-featured observability platforms (for example, Honeycomb, Datadog)**: Correlation between metrics and events
###
[​
](#for-traces)
For traces
Choose a backend that supports distributed trace storage and span correlation:
* **Distributed tracing systems (for example, Jaeger, Zipkin, Grafana Tempo)**: Span visualization, request waterfalls, latency analysis
* **Full-featured observability platforms (for example, Honeycomb, Datadog)**: Trace search and correlation with metrics and logs
For organizations requiring Daily/Weekly/Monthly Active User (DAU/WAU/MAU) metrics, consider backends that support efficient unique value queries.
##
[​
](#service-information)
Service information
All metrics and events are exported with the following resource attributes:
* `service.name`: `claude-code`
* `service.version`: Current Claude Code version
* `os.type`: Operating system type (for example, `linux`, `darwin`, `windows`)
* `os.version`: Operating system version string
* `host.arch`: Host architecture (for example, `amd64`, `arm64`)
* `wsl.version`: WSL version number (only present when running on Windows Subsystem for Linux)
* Meter Name: `com.anthropic.claude\_code`
##
[​
](#roi-measurement-resources)
ROI measurement resources
For a comprehensive guide on measuring return on investment for Claude Code, including telemetry setup, cost analysis, productivity metrics, and automated reporting, see the [Claude Code ROI Measurement Guide](https://github.com/anthropics/claude-code-monitoring-guide). This repository provides ready-to-use Docker Compose configurations, Prometheus and OpenTelemetry setups, and templates for generating productivity reports integrated with tools like Linear.
##
[​
](#security-and-privacy)
Security and privacy
* OpenTelemetry export to your backend is opt-in and requires explicit configuration. For Anthropic’s separate operational telemetry and how to disable it, see [Data usage](/docs/en/data-usage#telemetry-services)
* Raw file contents and code snippets are not included in metrics or events. Trace spans are a separate data path: see the `OTEL\_LOG\_TOOL\_CONTENT` bullet below
* When authenticated via OAuth, `user.email` is included in telemetry attributes. If this is a concern for your organization, work with your telemetry backend to filter or redact this field
* User prompt content is not collected by default. Only prompt length is recorded. To include prompt content, set `OTEL\_LOG\_USER\_PROMPTS=1`
* Tool input arguments and parameters are not logged by default. To include them, set `OTEL\_LOG\_TOOL\_DETAILS=1`. When enabled, `tool\_result` events include a `tool\_parameters` attribute with Bash commands, MCP server and tool names, and skill names, plus a `tool\_input` attribute with file paths, URLs, search patterns, and other arguments. `user\_prompt` events include the verbatim `command\_name` for custom, plugin, and MCP commands. Trace spans include the same `tool\_input` attribute and input-derived attributes such as `file\_path`. Individual values over 512 characters are truncated and the total is bounded to \~4 K characters, but the arguments may still contain sensitive values. Configure your telemetry backend to filter or redact these attributes as needed
* Tool input and output content is not logged in trace spans by default. To include it, set `OTEL\_LOG\_TOOL\_CONTENT=1`. When enabled, span events include full tool input and output content truncated at 60 KB per span. This can include raw file contents from Read tool results and Bash command output. Configure your telemetry backend to filter or redact these attributes as needed
* Raw Anthropic Messages API request and response bodies are not logged by default. To include them, set `OTEL\_LOG\_RAW\_API\_BODIES`. With `=1`, each API call emits `api\_request\_body` and `api\_response\_body` log events whose `body` attribute is the JSON-serialized payload, truncated at 60 KB. With `=file:\<dir\>`, untruncated bodies are written to `.request.json` and `.response.json` files under that directory and the events carry a `body\_ref` path instead of the inline body. Ship the directory with a log collector or sidecar rather than through the telemetry stream. In both modes, bodies contain the full conversation history (system prompt, every prior user and assistant turn, tool results), so enabling this implies consent to everything the other `OTEL\_LOG\_\*` content flags would reveal. Claude’s extended-thinking content is always redacted from these bodies regardless of other settings
##
[​
](#monitor-claude-code-on-amazon-bedrock)
Monitor Claude Code on Amazon Bedrock
For detailed Claude Code usage monitoring guidance for Amazon Bedrock, see [Claude Code Monitoring Implementation (Bedrock)](https://github.com/aws-solutions-library-samples/guidance-for-claude-code-with-amazon-bedrock/blob/main/assets/docs/MONITORING.md).
⌘I