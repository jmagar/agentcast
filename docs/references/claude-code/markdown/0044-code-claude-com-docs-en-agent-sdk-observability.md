Observability with OpenTelemetry - Claude Code Docs
## > Documentation Index
> Fetch the complete documentation index at:
[> https://code.claude.com/docs/llms.txt
](https://code.claude.com/docs/llms.txt)
> Use this file to discover all available pages before exploring further.
When you run agents in production, you need visibility into what they did:
* which tools they called
* how long each model request took
* how many tokens were spent
* where failures occurred
The Agent SDK can export this data as OpenTelemetry traces, metrics, and log events to any backend that accepts the OpenTelemetry Protocol (OTLP), such as Honeycomb, Datadog, Grafana, Langfuse, or a self-hosted collector.
This guide explains how the SDK emits telemetry, how to configure the export, and how to tag and filter the data once it reaches your backend. To read token usage and cost directly from the SDK response stream instead of exporting to a backend, see [Track cost and usage](/docs/en/agent-sdk/cost-tracking).
##
[​
](#how-telemetry-flows-from-the-sdk)
How telemetry flows from the SDK
The Agent SDK runs the Claude Code CLI as a child process and communicates with it over a local pipe. The CLI has OpenTelemetry instrumentation built in: it records spans around each model request and tool execution, emits metrics for token and cost counters, and emits structured log events for prompts and tool results. The SDK does not produce telemetry of its own. Instead, it passes configuration through to the CLI process, and the CLI exports directly to your collector.
Configuration is passed as environment variables. By default, the child process inherits your application’s environment, so you can configure telemetry in either of two places:
* **Process environment:** set the variables in your shell, container, or orchestrator before your application starts. Every `query()` call picks them up automatically with no code change. This is the recommended approach for production deployments.
* **Per-call options:** set the variables in `ClaudeAgentOptions.env` (Python) or `options.env` (TypeScript). Use this when different agents in the same process need different telemetry settings. In Python, `env` is merged on top of the inherited environment. In TypeScript, `env` replaces the inherited environment entirely, so include `...process.env` in the object you pass.
The CLI exports three independent OpenTelemetry signals. Each has its own enable switch and its own exporter, so you can turn on only the ones you need.
|Signal|What it contains|Enable with|
|Metrics|Counters for tokens, cost, sessions, lines of code, and tool decisions|`OTEL\_METRICS\_EXPORTER`|
|Log events|Structured records for each prompt, API request, API error, and tool result|`OTEL\_LOGS\_EXPORTER`|
|Traces|Spans for each interaction, model request, tool call, and hook (beta)|`OTEL\_TRACES\_EXPORTER` plus `CLAUDE\_CODE\_ENHANCED\_TELEMETRY\_BETA=1`|
For the complete list of metric names, event names, and attributes, see the Claude Code [Monitoring](/docs/en/monitoring-usage) reference. The Agent SDK emits the same data because it runs the same CLI. Span names are listed in [Read agent traces](#read-agent-traces) below.
##
[​
](#enable-telemetry-export)
Enable telemetry export
Telemetry is off until you set `CLAUDE\_CODE\_ENABLE\_TELEMETRY=1` and choose at least one exporter. The most common configuration sends all three signals over OTLP HTTP to a collector.
The following example sets the variables in a dictionary and passes them through `options.env`. The agent runs a single task, and the CLI exports spans, metrics, and events to the collector at `collector.example.com` while the loop consumes the response stream:
Python
TypeScript
```
`import asyncio
from claude\_agent\_sdk import query, ClaudeAgentOptions
OTEL\_ENV = {
"CLAUDE\_CODE\_ENABLE\_TELEMETRY": "1",
# Required for traces, which are in beta. Metrics and log events do not need this.
"CLAUDE\_CODE\_ENHANCED\_TELEMETRY\_BETA": "1",
# Choose an exporter per signal. Use otlp for the SDK; see the Note below.
"OTEL\_TRACES\_EXPORTER": "otlp",
"OTEL\_METRICS\_EXPORTER": "otlp",
"OTEL\_LOGS\_EXPORTER": "otlp",
# Standard OTLP transport configuration.
"OTEL\_EXPORTER\_OTLP\_PROTOCOL": "http/protobuf",
"OTEL\_EXPORTER\_OTLP\_ENDPOINT": "http://collector.example.com:4318",
"OTEL\_EXPORTER\_OTLP\_HEADERS": "Authorization=Bearer your-token",
}
async def main():
options = ClaudeAgentOptions(env=OTEL\_ENV)
async for message in query(
prompt="List the files in this directory", options=options
):
print(message)
asyncio.run(main())
`
```
Because the child process inherits your application’s environment by default, you can achieve the same result by exporting these variables in a Dockerfile, Kubernetes manifest, or shell profile and omitting `options.env` entirely.
The `console` exporter writes telemetry to standard output, which the SDK uses
as its message channel. Do not set `console` as an exporter value when running
through the SDK. To inspect telemetry locally, point
`OTEL\_EXPORTER\_OTLP\_ENDPOINT` at a local collector or an all-in-one Jaeger
container instead.
###
[​
](#flush-telemetry-from-short-lived-calls)
Flush telemetry from short-lived calls
The CLI batches telemetry and exports on an interval. On a clean process exit it attempts to flush pending data, but the flush is bounded by a short timeout, so spans can still be dropped if the collector is slow to respond. If your process is killed before the CLI shuts down, anything still in the batch buffer is lost. Lowering the export intervals reduces both windows.
By default, metrics export every 60 seconds and traces and logs export every 5 seconds. The following example shortens all three intervals so that data reaches the collector while a short task is still running:
Python
TypeScript
```
`OTEL\_ENV = {
# ... exporter configuration from the previous example ...
"OTEL\_METRIC\_EXPORT\_INTERVAL": "1000",
"OTEL\_LOGS\_EXPORT\_INTERVAL": "1000",
"OTEL\_TRACES\_EXPORT\_INTERVAL": "1000",
}
`
```
##
[​
](#read-agent-traces)
Read agent traces
Traces give you the most detailed view of an agent run. With `CLAUDE\_CODE\_ENHANCED\_TELEMETRY\_BETA=1` set, each step of the agent loop becomes a span you can inspect in your tracing backend:
* **`claude\_code.interaction`:** wraps a single turn of the agent loop, from receiving a prompt to producing a response.
* **`claude\_code.llm\_request`:** wraps each call to the Claude API, with model name, latency, and token counts as attributes.
* **`claude\_code.tool`:** wraps each tool invocation, with child spans for the permission wait (`claude\_code.tool.blocked\_on\_user`) and the execution itself (`claude\_code.tool.execution`).
* **`claude\_code.hook`:** wraps each [hook](/docs/en/agent-sdk/hooks) execution. Requires detailed beta tracing (`ENABLE\_BETA\_TRACING\_DETAILED=1` and `BETA\_TRACING\_ENDPOINT`) in addition to the variables above.
The `llm\_request`, `tool`, and `hook` spans are children of the enclosing `claude\_code.interaction` span. When the agent spawns a subagent through the Task tool, the subagent’s `llm\_request` and `tool` spans nest under the parent agent’s `claude\_code.tool` span, so the full delegation chain appears as one trace.
Spans carry a `session.id` attribute by default. When you make several `query()` calls against the same [session](/docs/en/agent-sdk/sessions), filter on `session.id` in your backend to see them as one timeline. The attribute is omitted if `OTEL\_METRICS\_INCLUDE\_SESSION\_ID` is set to a falsy value.
Tracing is in beta. Span names and attributes may change between releases. See
[Traces (beta)](/docs/en/monitoring-usage#traces-beta) in the Monitoring reference
for the trace exporter configuration variables.
##
[​
](#link-traces-to-your-application)
Link traces to your application
The SDK automatically propagates W3C trace context into the CLI subprocess. When you call `query()` while an OpenTelemetry span is active in your application, the SDK injects `TRACEPARENT` and `TRACESTATE` into the child process environment, and the CLI reads them so its `claude\_code.interaction` span becomes a child of your span. The agent run then appears inside your application’s trace instead of as a disconnected root.
The CLI also forwards `TRACEPARENT` to every Bash and PowerShell command it runs. If a command launched through the Bash tool emits its own OpenTelemetry spans, those spans nest under the `claude\_code.tool.execution` span that wraps the command.
Auto-injection is skipped when you set `TRACEPARENT` explicitly in `options.env`, so you can pin a specific parent context if needed. Interactive CLI sessions ignore inbound `TRACEPARENT` entirely; only Agent SDK and `claude -p` runs honor it. See [Traces (beta)](/docs/en/monitoring-usage#traces-beta) in the Monitoring reference for the full span and attribute reference.
##
[​
](#tag-telemetry-from-your-agent)
Tag telemetry from your agent
By default, the CLI reports `service.name` as `claude-code`. If you run several agents, or run the SDK alongside other services that export to the same collector, override the service name and add resource attributes so you can filter by agent in your backend.
The following example renames the service and attaches deployment metadata. These values are applied as OpenTelemetry resource attributes on every span, metric, and event the agent emits:
Python
TypeScript
```
`options = ClaudeAgentOptions(
env={
# ... exporter configuration ...
"OTEL\_SERVICE\_NAME": "support-triage-agent",
"OTEL\_RESOURCE\_ATTRIBUTES": "service.version=1.4.0,deployment.environment=production",
},
)
`
```
##
[​
](#attribute-actions-to-your-end-users)
Attribute actions to your end users
The CLI attaches [identity attributes](/docs/en/monitoring-usage#standard-attributes) to every event based on the credential it uses to call Anthropic. When you build an application that serves many end users from one deployment, these attributes identify your service’s credential, not the end user on whose behalf the agent acted.
To make tool calls and MCP activity attributable to your application’s end users, inject end-user identity as resource attributes on each `query()` call. Percent-encode values before interpolating them, since `OTEL\_RESOURCE\_ATTRIBUTES` [reserves commas, spaces, and equals signs](/docs/en/monitoring-usage#multi-team-organization-support). The following example attaches the requesting user and tenant to every span and event from one request:
Python
TypeScript
```
`from urllib.parse import quote
options = ClaudeAgentOptions(
env={
# ... exporter configuration ...
"OTEL\_RESOURCE\_ATTRIBUTES": f"enduser.id={quote(request.user\_id)},tenant.id={quote(request.tenant\_id)}",
},
)
`
```
With end-user identity attached, the `tool\_decision`, `tool\_result`, `mcp\_server\_connection`, and `permission\_mode\_changed` events become a per-user audit trail you can forward to a Security Information and Event Management (SIEM) platform. See [Audit security events](/docs/en/monitoring-usage#audit-security-events) in the Monitoring reference for the full list of security-relevant events and the attributes each one carries.
##
[​
](#control-sensitive-data-in-exports)
Control sensitive data in exports
Telemetry is structural by default. Durations, model names, and tool names are recorded on every span; token counts are recorded when the underlying API request returns usage data, so spans for failed or aborted requests may omit them. The content your agent reads and writes is not recorded by default. These opt-in variables add content to the exported data:
|Variable|Adds|
|`OTEL\_LOG\_USER\_PROMPTS=1`|Prompt text on `claude\_code.user\_prompt` events and on the `claude\_code.interaction` span|
|`OTEL\_LOG\_TOOL\_DETAILS=1`|Tool input arguments (file paths, shell commands, search patterns) on `claude\_code.tool\_result` events|
|`OTEL\_LOG\_TOOL\_CONTENT=1`|Full tool input and output bodies as span events on `claude\_code.tool`, truncated at 60 KB. Requires [tracing](#read-agent-traces) to be enabled|
|`OTEL\_LOG\_RAW\_API\_BODIES`|Full Anthropic Messages API request and response JSON as `claude\_code.api\_request\_body` and `claude\_code.api\_response\_body` log events. Set to `1` for inline bodies truncated at 60 KB, or `file:\<dir\>` for untruncated bodies on disk with a `body\_ref` path in the event. Bodies include the entire conversation history and have extended-thinking content redacted. Enabling this implies consent to everything the three variables above would reveal|
Leave these unset unless your observability pipeline is approved to store the data your agent handles. See [Security and privacy](/docs/en/monitoring-usage#security-and-privacy) in the Monitoring reference for the full list of attributes and redaction behavior.
##
[​
](#related-documentation)
Related documentation
These guides cover adjacent topics for monitoring and deploying agents:
* [Track cost and usage](/docs/en/agent-sdk/cost-tracking): read token and cost data from the message stream without an external backend.
* [Hosting the Agent SDK](/docs/en/agent-sdk/hosting): deploy agents in containers where you can set OpenTelemetry variables at the environment level.
* [Monitoring](/docs/en/monitoring-usage): the complete reference for every environment variable, metric, and event the CLI emits.
⌘I