Agent SDK reference - TypeScript - Claude Code Docs
## > Documentation Index
> Fetch the complete documentation index at:
[> https://code.claude.com/docs/llms.txt
](https://code.claude.com/docs/llms.txt)
> Use this file to discover all available pages before exploring further.
##
[​
](#installation)
Installation
```
`npm install @anthropic-ai/claude-agent-sdk
`
```
The SDK bundles a native Claude Code binary for your platform as an optional dependency such as `@anthropic-ai/claude-agent-sdk-darwin-arm64`. You don’t need to install Claude Code separately. If your package manager skips optional dependencies, the SDK throws `Native CLI binary for \<platform\> not found`; set [`pathToClaudeCodeExecutable`](#options) to a separately installed `claude` binary instead.
##
[​
](#functions)
Functions
###
[​
](#query)
`query()`
The primary function for interacting with Claude Code. Creates an async generator that streams messages as they arrive.
```
`function query({
prompt,
options
}: {
prompt: string | AsyncIterable\<SDKUserMessage\>;
options?: Options;
}): Query;
`
```
####
[​
](#parameters)
Parameters
|Parameter|Type|Description|
|`prompt`|`string | AsyncIterable\<`[`SDKUserMessage`](#sdkusermessage)`\>`|The input prompt as a string or async iterable for streaming mode|
|`options`|[`Options`](#options)|Optional configuration object (see Options type below)|
####
[​
](#returns)
Returns
Returns a [`Query`](#query-object) object that extends `AsyncGenerator\<`[`SDKMessage`](#sdkmessage)`, void\>` with additional methods.
###
[​
](#startup)
`startup()`
Pre-warms the CLI subprocess by spawning it and completing the initialize handshake before a prompt is available. The returned [`WarmQuery`](#warmquery) handle accepts a prompt later and writes it to an already-ready process, so the first `query()` call resolves without paying subprocess spawn and initialization cost inline.
```
`function startup(params?: {
options?: Options;
initializeTimeoutMs?: number;
}): Promise\<WarmQuery\>;
`
```
####
[​
](#parameters-2)
Parameters
|Parameter|Type|Description|
|`options`|[`Options`](#options)|Optional configuration object. Same as the `options` parameter to `query()`|
|`initializeTimeoutMs`|`number`|Maximum time in milliseconds to wait for subprocess initialization. Defaults to `60000`. If initialization does not complete in time, the promise rejects with a timeout error|
####
[​
](#returns-2)
Returns
Returns a `Promise\<`[`WarmQuery`](#warmquery)`\>` that resolves once the subprocess has spawned and completed its initialize handshake.
####
[​
](#example)
Example
Call `startup()` early, for example on application boot, then call `.query()` on the returned handle once a prompt is ready. This moves subprocess spawn and initialization out of the critical path.
```
`import { startup } from "@anthropic-ai/claude-agent-sdk";
// Pay startup cost upfront
const warm = await startup({ options: { maxTurns: 3 } });
// Later, when a prompt is ready, this is immediate
for await (const message of warm.query("What files are here?")) {
console.log(message);
}
`
```
###
[​
](#tool)
`tool()`
Creates a type-safe MCP tool definition for use with SDK MCP servers.
```
`function tool\<Schema extends AnyZodRawShape\>(
name: string,
description: string,
inputSchema: Schema,
handler: (args: InferShape\<Schema\>, extra: unknown) =\> Promise\<CallToolResult\>,
extras?: { annotations?: ToolAnnotations }
): SdkMcpToolDefinition\<Schema\>;
`
```
####
[​
](#parameters-3)
Parameters
|Parameter|Type|Description|
|`name`|`string`|The name of the tool|
|`description`|`string`|A description of what the tool does|
|`inputSchema`|`Schema extends AnyZodRawShape`|Zod schema defining the tool’s input parameters (supports both Zod 3 and Zod 4)|
|`handler`|`(args, extra) =\> Promise\<`[`CallToolResult`](#calltoolresult)`\>`|Async function that executes the tool logic|
|`extras`|`{ annotations?: `[`ToolAnnotations`](#toolannotations)` }`|Optional MCP tool annotations providing behavioral hints to clients|
####
[​
](#toolannotations)
`ToolAnnotations`
Re-exported from `@modelcontextprotocol/sdk/types.js`. All fields are optional hints; clients should not rely on them for security decisions.
|Field|Type|Default|Description|
|`title`|`string`|`undefined`|Human-readable title for the tool|
|`readOnlyHint`|`boolean`|`false`|If `true`, the tool does not modify its environment|
|`destructiveHint`|`boolean`|`true`|If `true`, the tool may perform destructive updates (only meaningful when `readOnlyHint` is `false`)|
|`idempotentHint`|`boolean`|`false`|If `true`, repeated calls with the same arguments have no additional effect (only meaningful when `readOnlyHint` is `false`)|
|`openWorldHint`|`boolean`|`true`|If `true`, the tool interacts with external entities (for example, web search). If `false`, the tool’s domain is closed (for example, a memory tool)|
```
`import { tool } from "@anthropic-ai/claude-agent-sdk";
import { z } from "zod";
const searchTool = tool(
"search",
"Search the web",
{ query: z.string() },
async ({ query }) =\> {
return { content: [{ type: "text", text: `Results for: ${query}` }] };
},
{ annotations: { readOnlyHint: true, openWorldHint: true } }
);
`
```
###
[​
](#createsdkmcpserver)
`createSdkMcpServer()`
Creates an MCP server instance that runs in the same process as your application.
```
`function createSdkMcpServer(options: {
name: string;
version?: string;
tools?: Array\<SdkMcpToolDefinition\<any\>\>;
}): McpSdkServerConfigWithInstance;
`
```
####
[​
](#parameters-4)
Parameters
|Parameter|Type|Description|
|`options.name`|`string`|The name of the MCP server|
|`options.version`|`string`|Optional version string|
|`options.tools`|`Array\<SdkMcpToolDefinition\>`|Array of tool definitions created with [`tool()`](#tool)|
###
[​
](#listsessions)
`listSessions()`
Discovers and lists past sessions with light metadata. Filter by project directory or list sessions across all projects.
```
`function listSessions(options?: ListSessionsOptions): Promise\<SDKSessionInfo[]\>;
`
```
####
[​
](#parameters-5)
Parameters
|Parameter|Type|Default|Description|
|`options.dir`|`string`|`undefined`|Directory to list sessions for. When omitted, returns sessions across all projects|
|`options.limit`|`number`|`undefined`|Maximum number of sessions to return|
|`options.includeWorktrees`|`boolean`|`true`|When `dir` is inside a git repository, include sessions from all worktree paths|
####
[​
](#return-type-sdksessioninfo)
Return type: `SDKSessionInfo`
|Property|Type|Description|
|`sessionId`|`string`|Unique session identifier (UUID)|
|`summary`|`string`|Display title: custom title, auto-generated summary, or first prompt|
|`lastModified`|`number`|Last modified time in milliseconds since epoch|
|`fileSize`|`number | undefined`|Session file size in bytes. Only populated for local JSONL storage|
|`customTitle`|`string | undefined`|User-set session title (via `/rename`)|
|`firstPrompt`|`string | undefined`|First meaningful user prompt in the session|
|`gitBranch`|`string | undefined`|Git branch at the end of the session|
|`cwd`|`string | undefined`|Working directory for the session|
|`tag`|`string | undefined`|User-set session tag (see [`tagSession()`](#tagsession))|
|`createdAt`|`number | undefined`|Creation time in milliseconds since epoch, from the first entry’s timestamp|
####
[​
](#example-2)
Example
Print the 10 most recent sessions for a project. Results are sorted by `lastModified` descending, so the first item is the newest. Omit `dir` to search across all projects.
```
`import { listSessions } from "@anthropic-ai/claude-agent-sdk";
const sessions = await listSessions({ dir: "/path/to/project", limit: 10 });
for (const session of sessions) {
console.log(`${session.summary} (${session.sessionId})`);
}
`
```
###
[​
](#getsessionmessages)
`getSessionMessages()`
Reads user and assistant messages from a past session transcript.
```
`function getSessionMessages(
sessionId: string,
options?: GetSessionMessagesOptions
): Promise\<SessionMessage[]\>;
`
```
####
[​
](#parameters-6)
Parameters
|Parameter|Type|Default|Description|
|`sessionId`|`string`|required|Session UUID to read (see `listSessions()`)|
|`options.dir`|`string`|`undefined`|Project directory to find the session in. When omitted, searches all projects|
|`options.limit`|`number`|`undefined`|Maximum number of messages to return|
|`options.offset`|`number`|`undefined`|Number of messages to skip from the start|
####
[​
](#return-type-sessionmessage)
Return type: `SessionMessage`
|Property|Type|Description|
|`type`|`"user" | "assistant"`|Message role|
|`uuid`|`string`|Unique message identifier|
|`session\_id`|`string`|Session this message belongs to|
|`message`|`unknown`|Raw message payload from the transcript|
|`parent\_tool\_use\_id`|`null`|Reserved|
####
[​
](#example-3)
Example
```
`import { listSessions, getSessionMessages } from "@anthropic-ai/claude-agent-sdk";
const [latest] = await listSessions({ dir: "/path/to/project", limit: 1 });
if (latest) {
const messages = await getSessionMessages(latest.sessionId, {
dir: "/path/to/project",
limit: 20
});
for (const msg of messages) {
console.log(`[${msg.type}] ${msg.uuid}`);
}
}
`
```
###
[​
](#getsessioninfo)
`getSessionInfo()`
Reads metadata for a single session by ID without scanning the full project directory.
```
`function getSessionInfo(
sessionId: string,
options?: GetSessionInfoOptions
): Promise\<SDKSessionInfo | undefined\>;
`
```
####
[​
](#parameters-7)
Parameters
|Parameter|Type|Default|Description|
|`sessionId`|`string`|required|UUID of the session to look up|
|`options.dir`|`string`|`undefined`|Project directory path. When omitted, searches all project directories|
Returns [`SDKSessionInfo`](#return-type-sdksessioninfo), or `undefined` if the session is not found.
###
[​
](#renamesession)
`renameSession()`
Renames a session by appending a custom-title entry. Repeated calls are safe; the most recent title wins.
```
`function renameSession(
sessionId: string,
title: string,
options?: SessionMutationOptions
): Promise\<void\>;
`
```
####
[​
](#parameters-8)
Parameters
|Parameter|Type|Default|Description|
|`sessionId`|`string`|required|UUID of the session to rename|
|`title`|`string`|required|New title. Must be non-empty after trimming whitespace|
|`options.dir`|`string`|`undefined`|Project directory path. When omitted, searches all project directories|
###
[​
](#tagsession)
`tagSession()`
Tags a session. Pass `null` to clear the tag. Repeated calls are safe; the most recent tag wins.
```
`function tagSession(
sessionId: string,
tag: string | null,
options?: SessionMutationOptions
): Promise\<void\>;
`
```
####
[​
](#parameters-9)
Parameters
|Parameter|Type|Default|Description|
|`sessionId`|`string`|required|UUID of the session to tag|
|`tag`|`string | null`|required|Tag string, or `null` to clear|
|`options.dir`|`string`|`undefined`|Project directory path. When omitted, searches all project directories|
###
[​
](#resolvesettings)
`resolveSettings()`
Resolves the effective Claude Code settings for a given directory using the same merge engine as the CLI, without spawning the Claude CLI. Use it to inspect what configuration a `query()` call would see before invoking one.
This function is alpha and its API may change before stabilization. It reads MDM sources, including macOS plist and Windows HKLM/HKCU, for parity with CLI startup, but does not execute the admin-configured `policyHelper` subprocess. The `permissions.defaultMode` field is returned as-is from all tiers including project settings. The trust filter the CLI applies before honoring escalating permission modes is not applied.
```
`function resolveSettings(
options?: ResolveSettingsOptions
): Promise\<ResolvedSettings\>;
`
```
####
[​
](#parameters-10)
Parameters
`resolveSettings()` accepts a single options object. All fields are optional.
|Parameter|Type|Default|Description|
|`options.cwd`|`string`|`process.cwd()`|Directory to resolve project and local settings relative to|
|`options.settingSources`|[`SettingSource`](#settingsource)`[]`|All sources|Which filesystem sources to load. Pass `[]` to skip user, project, and local settings. Managed policy settings load in all cases|
|`options.managedSettings`|`Settings`|`undefined`|Restrictive policy-tier settings merged at the managed-policy precedence level. Non-restrictive keys such as `model` are silently dropped|
|`options.serverManagedSettings`|`Settings`|`undefined`|Server-managed settings payload from `/api/claude\_code/settings`. Non-restrictive keys pass through unfiltered|
####
[​
](#return-type-resolvedsettings)
Return type: `ResolvedSettings`
`resolveSettings()` returns an object describing the merged settings and the source that contributed each key.
|Property|Type|Description|
|`effective`|`Settings`|Merged settings after applying all enabled sources in precedence order|
|`provenance`|`Partial\<Record\<keyof Settings, ProvenanceEntry\>\>`|For each top-level key in `effective`, which source supplied the value|
|`sources`|`Array\<{ source, settings, path?, policyOrigin? }\>`|Per-source raw settings, ordered from lowest to highest precedence|
####
[​
](#example-4)
Example
The example below resolves settings for a project directory and prints the source that controls the cleanup period.
```
`import { resolveSettings } from "@anthropic-ai/claude-agent-sdk";
const { effective, provenance } = await resolveSettings({
cwd: "/path/to/project",
settingSources: ["user", "project", "local"],
});
console.log(`Cleanup period: ${effective.cleanupPeriodDays} days`);
console.log(`Set by: ${provenance.cleanupPeriodDays?.source}`);
`
```
##
[​
](#types)
Types
###
[​
](#options)
`Options`
Configuration object for the `query()` function.
|Property|Type|Default|Description|
|`abortController`|`AbortController`|`new AbortController()`|Controller for cancelling operations|
|`additionalDirectories`|`string[]`|`[]`|Additional directories Claude can access|
|`agent`|`string`|`undefined`|Agent name for the main thread. The agent must be defined in the `agents` option or in settings|
|`agents`|`Record\<string, [`AgentDefinition`](#agentdefinition)\>`|`undefined`|Programmatically define subagents|
|`allowDangerouslySkipPermissions`|`boolean`|`false`|Enable bypassing permissions. Required when using `permissionMode: 'bypassPermissions'`|
|`allowedTools`|`string[]`|`[]`|Tools to auto-approve without prompting. This does not restrict Claude to only these tools; unlisted tools fall through to `permissionMode` and `canUseTool`. Use `disallowedTools` to block tools. See [Permissions](/docs/en/agent-sdk/permissions#allow-and-deny-rules)|
|`betas`|[`SdkBeta`](#sdkbeta)`[]`|`[]`|Enable beta features|
|`canUseTool`|[`CanUseTool`](#canusetool)|`undefined`|Custom permission function for tool usage|
|`continue`|`boolean`|`false`|Continue the most recent conversation|
|`cwd`|`string`|`process.cwd()`|Current working directory|
|`debug`|`boolean`|`false`|Enable debug mode for the Claude Code process|
|`debugFile`|`string`|`undefined`|Write debug logs to a specific file path. Implicitly enables debug mode|
|`disallowedTools`|`string[]`|`[]`|Tools to always deny. Deny rules are checked first and override `allowedTools` and `permissionMode` (including `bypassPermissions`)|
|`effort`|`'low' | 'medium' | 'high' | 'xhigh' | 'max'`|`'high'`|Controls how much effort Claude puts into its response. Works with adaptive thinking to guide thinking depth|
|`enableFileCheckpointing`|`boolean`|`false`|Enable file change tracking for rewinding. See [File checkpointing](/docs/en/agent-sdk/file-checkpointing)|
|`env`|`Record\<string, string | undefined\>`|`process.env`|Environment variables. See [Environment variables](/docs/en/env-vars) for variables the underlying CLI reads, and [Handle slow or stalled API responses](#handle-slow-or-stalled-api-responses) for timeout-related variables. Set `CLAUDE\_AGENT\_SDK\_CLIENT\_APP` to identify your app in the User-Agent header|
|`executable`|`'bun' | 'deno' | 'node'`|Auto-detected|JavaScript runtime to use|
|`executableArgs`|`string[]`|`[]`|Arguments to pass to the executable|
|`extraArgs`|`Record\<string, string | null\>`|`{}`|Additional arguments|
|`fallbackModel`|`string`|`undefined`|Model to use if primary fails|
|`forkSession`|`boolean`|`false`|When resuming with `resume`, fork to a new session ID instead of continuing the original session|
|`hooks`|`Partial\<Record\<`[`HookEvent`](#hookevent)`, `[`HookCallbackMatcher`](#hookcallbackmatcher)`[]\>\>`|`{}`|Hook callbacks for events|
|`includePartialMessages`|`boolean`|`false`|Include partial message events|
|`maxBudgetUsd`|`number`|`undefined`|Stop the query when the client-side cost estimate reaches this USD value. Compared against the same estimate as `total\_cost\_usd`; see [Track cost and usage](/docs/en/agent-sdk/cost-tracking) for accuracy caveats|
|`maxThinkingTokens`|`number`|`undefined`|*Deprecated:* Use `thinking` instead. Maximum tokens for thinking process|
|`maxTurns`|`number`|`undefined`|Maximum agentic turns (tool-use round trips)|
|`mcpServers`|`Record\<string, [`McpServerConfig`](#mcpserverconfig)\>`|`{}`|MCP server configurations|
|`model`|`string`|Default from CLI|Claude model to use|
|`outputFormat`|`{ type: 'json\_schema', schema: JSONSchema }`|`undefined`|Define output format for agent results. See [Structured outputs](/docs/en/agent-sdk/structured-outputs) for details|
|`outputStyle`|`string`|`undefined`|Name of an [output style](/docs/en/output-styles) to activate for the session. The style must exist in a loaded `settingSources` location, such as `.claude/output-styles/`. See [Activate an output style](/docs/en/agent-sdk/modifying-system-prompts#activate-an-output-style)|
|`pathToClaudeCodeExecutable`|`string`|Auto-resolved from bundled native binary|Path to Claude Code executable. Only needed if optional dependencies were skipped during install or your platform isn’t in the supported set|
|`permissionMode`|[`PermissionMode`](#permissionmode)|`'default'`|Permission mode for the session|
|`permissionPromptToolName`|`string`|`undefined`|MCP tool name for permission prompts|
|`persistSession`|`boolean`|`true`|When `false`, disables session persistence to disk. Sessions cannot be resumed later|
|`plugins`|[`SdkPluginConfig`](#sdkpluginconfig)`[]`|`[]`|Load custom plugins from local paths. See [Plugins](/docs/en/agent-sdk/plugins) for details|
|`promptSuggestions`|`boolean`|`false`|Enable prompt suggestions. Emits a `prompt\_suggestion` message after each turn with a predicted next user prompt|
|`resume`|`string`|`undefined`|Session ID to resume|
|`resumeSessionAt`|`string`|`undefined`|Resume session at a specific message UUID|
|`sandbox`|[`SandboxSettings`](#sandboxsettings)|`undefined`|Configure sandbox behavior programmatically. See [Sandbox settings](#sandboxsettings) for details|
|`sessionId`|`string`|Auto-generated|Use a specific UUID for the session instead of auto-generating one|
|`sessionStore`|[`SessionStore`](/docs/en/agent-sdk/session-storage#the-sessionstore-interface)|`undefined`|Mirror session transcripts to an external backend so any host can resume them. See [Persist sessions to external storage](/docs/en/agent-sdk/session-storage)|
|`settings`|`string | Settings`|`undefined`|Inline [settings](/docs/en/settings) object or path to a settings file. Populates the flag-settings layer in the [precedence order](/docs/en/settings#settings-precedence). Change at runtime with [`applyFlagSettings()`](#applyflagsettings)|
|`settingSources`|[`SettingSource`](#settingsource)`[]`|CLI defaults (all sources)|Control which filesystem settings to load. Pass `[]` to disable user, project, and local settings. Managed policy settings load regardless. See [Use Claude Code features](/docs/en/agent-sdk/claude-code-features#what-settingsources-does-not-control)|
|`skills`|`string[] | 'all'`|`undefined`|Skills available to the session. Pass `'all'` to enable every discovered skill, or a list of skill names. When set, the SDK enables the Skill tool automatically without listing it in `allowedTools`. See [Skills](/docs/en/agent-sdk/skills)|
|`spawnClaudeCodeProcess`|`(options: SpawnOptions) =\> SpawnedProcess`|`undefined`|Custom function to spawn the Claude Code process. Use to run Claude Code in VMs, containers, or remote environments|
|`stderr`|`(data: string) =\> void`|`undefined`|Callback for stderr output|
|`strictMcpConfig`|`boolean`|`false`|Enforce strict MCP validation|
|`systemPrompt`|`string | { type: 'preset'; preset: 'claude\_code'; append?: string; excludeDynamicSections?: boolean }`|`undefined` (minimal prompt)|System prompt configuration. Pass a string for custom prompt, or `{ type: 'preset', preset: 'claude\_code' }` to use Claude Code’s system prompt. When using the preset object form, add `append` to extend it with additional instructions, and set `excludeDynamicSections: true` to move per-session context into the first user message for [better prompt-cache reuse across machines](/docs/en/agent-sdk/modifying-system-prompts#improve-prompt-caching-across-users-and-machines)|
|`thinking`|[`ThinkingConfig`](#thinkingconfig)|`{ type: 'adaptive' }` for supported models|Controls Claude’s thinking/reasoning behavior. See [`ThinkingConfig`](#thinkingconfig) for options|
|`toolConfig`|[`ToolConfig`](#toolconfig)|`undefined`|Configuration for built-in tool behavior. See [`ToolConfig`](#toolconfig) for details|
|`tools`|`string[] | { type: 'preset'; preset: 'claude\_code' }`|`undefined`|Tool configuration. Pass an array of tool names or use the preset to get Claude Code’s default tools|
####
[​
](#handle-slow-or-stalled-api-responses)
Handle slow or stalled API responses
The CLI subprocess reads several environment variables that control API timeouts and stall detection. Pass them through the `env` option:
```
`const result = query({
prompt: "Analyze this code",
options: {
env: {
...process.env,
API\_TIMEOUT\_MS: "120000",
CLAUDE\_CODE\_MAX\_RETRIES: "2",
CLAUDE\_ASYNC\_AGENT\_STALL\_TIMEOUT\_MS: "120000",
},
},
});
`
```
* `API\_TIMEOUT\_MS`: per-request timeout on the Anthropic client, in milliseconds. Default `600000`. Applies to the main loop and all subagents.
* `CLAUDE\_CODE\_MAX\_RETRIES`: maximum API retries. Default `10`. Each retry gets its own `API\_TIMEOUT\_MS` window, so worst-case wall time is roughly `API\_TIMEOUT\_MS × (CLAUDE\_CODE\_MAX\_RETRIES + 1)` plus backoff.
* `CLAUDE\_ASYNC\_AGENT\_STALL\_TIMEOUT\_MS`: stall watchdog for subagents launched with `run\_in\_background`. Default `600000`. Resets on each stream event; on stall it aborts the subagent, marks the task failed, and surfaces the error to the parent with any partial result. Does not apply to synchronous subagents.
* `CLAUDE\_ENABLE\_STREAM\_WATCHDOG=1` with `CLAUDE\_STREAM\_IDLE\_TIMEOUT\_MS`: aborts the request when headers have arrived but the response body stops streaming. Off by default. `CLAUDE\_STREAM\_IDLE\_TIMEOUT\_MS` defaults to `300000` and is clamped to that minimum. The aborted request goes through the normal retry path.
###
[​
](#query-object)
`Query` object
Interface returned by the `query()` function.
```
`interface Query extends AsyncGenerator\<SDKMessage, void\> {
interrupt(): Promise\<void\>;
rewindFiles(
userMessageId: string,
options?: { dryRun?: boolean }
): Promise\<RewindFilesResult\>;
setPermissionMode(mode: PermissionMode): Promise\<void\>;
setModel(model?: string): Promise\<void\>;
setMaxThinkingTokens(maxThinkingTokens: number | null): Promise\<void\>;
applyFlagSettings(settings: { [K in keyof Settings]?: Settings[K] | null }): Promise\<void\>;
initializationResult(): Promise\<SDKControlInitializeResponse\>;
supportedCommands(): Promise\<SlashCommand[]\>;
supportedModels(): Promise\<ModelInfo[]\>;
supportedAgents(): Promise\<AgentInfo[]\>;
mcpServerStatus(): Promise\<McpServerStatus[]\>;
accountInfo(): Promise\<AccountInfo\>;
reconnectMcpServer(serverName: string): Promise\<void\>;
toggleMcpServer(serverName: string, enabled: boolean): Promise\<void\>;
setMcpServers(servers: Record\<string, McpServerConfig\>): Promise\<McpSetServersResult\>;
streamInput(stream: AsyncIterable\<SDKUserMessage\>): Promise\<void\>;
stopTask(taskId: string): Promise\<void\>;
close(): void;
}
`
```
####
[​
](#methods)
Methods
|Method|Description|
|`interrupt()`|Interrupts the query (only available in streaming input mode)|
|`rewindFiles(userMessageId, options?)`|Restores files to their state at the specified user message. Pass `{ dryRun: true }` to preview changes. Requires `enableFileCheckpointing: true`. See [File checkpointing](/docs/en/agent-sdk/file-checkpointing)|
|`setPermissionMode()`|Changes the permission mode (only available in streaming input mode)|
|`setModel()`|Changes the model (only available in streaming input mode)|
|`setMaxThinkingTokens()`|*Deprecated:* Use the `thinking` option instead. Changes the maximum thinking tokens|
|`applyFlagSettings(settings)`|Merges settings into the session’s flag settings layer at runtime (only available in streaming input mode). See [`applyFlagSettings()`](#applyflagsettings)|
|`initializationResult()`|Returns the full initialization result including supported commands, models, account info, and output style configuration|
|`supportedCommands()`|Returns available slash commands|
|`supportedModels()`|Returns available models with display info|
|`supportedAgents()`|Returns available subagents as [`AgentInfo`](#agentinfo)`[]`|
|`mcpServerStatus()`|Returns status of connected MCP servers|
|`accountInfo()`|Returns account information|
|`reconnectMcpServer(serverName)`|Reconnect an MCP server by name|
|`toggleMcpServer(serverName, enabled)`|Enable or disable an MCP server by name|
|`setMcpServers(servers)`|Dynamically replace the set of MCP servers for this session. Returns info about which servers were added, removed, and any errors|
|`streamInput(stream)`|Stream input messages to the query for multi-turn conversations|
|`stopTask(taskId)`|Stop a running background task by ID|
|`close()`|Close the query and terminate the underlying process. Forcefully ends the query and cleans up all resources|
####
[​
](#applyflagsettings)
`applyFlagSettings()`
Changes any [setting](/docs/en/settings) on a running session without restarting the query. Use it when a setting that has no dedicated setter needs to change mid-session, such as tightening `permissions` after the agent reads untrusted input. `setModel()` and `setPermissionMode()` are dedicated setters for those two keys; `applyFlagSettings()` is the general form that accepts any subset of the settings keys, and passing `model` here behaves the same as `setModel()`.
The values are written to the flag-settings layer, the same layer the inline `settings` option of `query()` populates at startup. Flag settings sit near the top of the [settings precedence order](/docs/en/settings#settings-precedence): they override user, project, and local settings, and only managed policy settings can override them. This is the same tier the [on-page precedence section](#settings-precedence) calls programmatic options.
Successive calls shallow-merge top-level keys. A second call with `{ permissions: {...} }` replaces the entire `permissions` object from the prior call rather than deep-merging into it. To clear a key from the flag layer and fall back to lower-precedence sources, pass `null` for that key. Passing `undefined` has no effect because JSON serialization drops it.
Only available in streaming input mode, the same constraint as `setModel()` and `setPermissionMode()`.
The example below switches the active model mid-session, then clears the override so the model falls back to whatever the user or project settings specify.
```
`const q = query({ prompt: messageStream });
// Override the model for the rest of the session
await q.applyFlagSettings({ model: "claude-opus-4-6" });
// Later: clear the override and fall back to lower-precedence settings
await q.applyFlagSettings({ model: null });
`
```
`applyFlagSettings()` is TypeScript-only. The Python SDK does not expose an equivalent method.
###
[​
](#warmquery)
`WarmQuery`
Handle returned by [`startup()`](#startup). The subprocess is already spawned and initialized, so calling `query()` on this handle writes the prompt directly to a ready process with no startup latency.
```
`interface WarmQuery extends AsyncDisposable {
query(prompt: string | AsyncIterable\<SDKUserMessage\>): Query;
close(): void;
}
`
```
####
[​
](#methods-2)
Methods
|Method|Description|
|`query(prompt)`|Send a prompt to the pre-warmed subprocess and return a [`Query`](#query-object). Can only be called once per `WarmQuery`|
|`close()`|Close the subprocess without sending a prompt. Use this to discard a warm query that is no longer needed|
`WarmQuery` implements `AsyncDisposable`, so it can be used with `await using` for automatic cleanup.
###
[​
](#sdkcontrolinitializeresponse)
`SDKControlInitializeResponse`
Return type of `initializationResult()`. Contains session initialization data.
```
`type SDKControlInitializeResponse = {
commands: SlashCommand[];
agents: AgentInfo[];
output\_style: string;
available\_output\_styles: string[];
models: ModelInfo[];
account: AccountInfo;
fast\_mode\_state?: "off" | "cooldown" | "on";
};
`
```
###
[​
](#agentdefinition)
`AgentDefinition`
Configuration for a subagent defined programmatically.
```
`type AgentDefinition = {
description: string;
tools?: string[];
disallowedTools?: string[];
prompt: string;
model?: string;
mcpServers?: AgentMcpServerSpec[];
skills?: string[];
initialPrompt?: string;
maxTurns?: number;
background?: boolean;
memory?: "user" | "project" | "local";
effort?: "low" | "medium" | "high" | "xhigh" | "max" | number;
permissionMode?: PermissionMode;
criticalSystemReminder\_EXPERIMENTAL?: string;
};
`
```
|Field|Required|Description|
|`description`|Yes|Natural language description of when to use this agent|
|`tools`|No|Array of allowed tool names. If omitted, inherits all tools from parent. To preload Skills into the agent’s context, use the `skills` field rather than listing `'Skill'` here|
|`disallowedTools`|No|Array of tool names to explicitly disallow for this agent|
|`prompt`|Yes|The agent’s system prompt|
|`model`|No|Model override for this agent. Accepts an alias such as `'sonnet'`, `'opus'`, `'haiku'`, `'inherit'`, or a full model ID. If omitted or `'inherit'`, uses the main model|
|`mcpServers`|No|MCP server specifications for this agent|
|`skills`|No|Array of skill names to preload into the agent context|
|`initialPrompt`|No|Auto-submitted as the first user turn when this agent runs as the main thread agent|
|`maxTurns`|No|Maximum number of agentic turns (API round-trips) before stopping|
|`background`|No|Run this agent as a non-blocking background task when invoked|
|`memory`|No|Memory source for this agent: `'user'`, `'project'`, or `'local'`|
|`effort`|No|Reasoning effort level for this agent. Accepts a named level or an integer|
|`permissionMode`|No|Permission mode for tool execution within this agent. See [`PermissionMode`](#permissionmode)|
|`criticalSystemReminder\_EXPERIMENTAL`|No|Experimental: Critical reminder added to the system prompt|
###
[​
](#agentmcpserverspec)
`AgentMcpServerSpec`
Specifies MCP servers available to a subagent. Can be a server name (string referencing a server from the parent’s `mcpServers` config) or an inline server configuration record mapping server names to configs.
```
`type AgentMcpServerSpec = string | Record\<string, McpServerConfigForProcessTransport\>;
`
```
Where `McpServerConfigForProcessTransport` is `McpStdioServerConfig | McpSSEServerConfig | McpHttpServerConfig | McpSdkServerConfig`.
###
[​
](#settingsource)
`SettingSource`
Controls which filesystem-based configuration sources the SDK loads settings from.
```
`type SettingSource = "user" | "project" | "local";
`
```
|Value|Description|Location|
|`'user'`|Global user settings|`\~/.claude/settings.json`|
|`'project'`|Shared project settings (version controlled)|`.claude/settings.json`|
|`'local'`|Local project settings (gitignored)|`.claude/settings.local.json`|
####
[​
](#default-behavior)
Default behavior
When `settingSources` is omitted or `undefined`, `query()` loads the same filesystem settings as the Claude Code CLI: user, project, and local. Managed policy settings are loaded in all cases. See [What settingSources does not control](/docs/en/agent-sdk/claude-code-features#what-settingsources-does-not-control) for inputs that are read regardless of this option, and how to disable them.
####
[​
](#why-use-settingsources)
Why use settingSources
**Disable filesystem settings:**
```
`// Do not load user, project, or local settings from disk
const result = query({
prompt: "Analyze this code",
options: { settingSources: [] }
});
`
```
**Load all filesystem settings explicitly:**
```
`const result = query({
prompt: "Analyze this code",
options: {
settingSources: ["user", "project", "local"] // Load all settings
}
});
`
```
**Load only specific setting sources:**
```
`// Load only project settings, ignore user and local
const result = query({
prompt: "Run CI checks",
options: {
settingSources: ["project"] // Only .claude/settings.json
}
});
`
```
**Testing and CI environments:**
```
`// Ensure consistent behavior in CI by excluding local settings
const result = query({
prompt: "Run tests",
options: {
settingSources: ["project"], // Only team-shared settings
permissionMode: "bypassPermissions"
}
});
`
```
**SDK-only applications:**
```
`// Define everything programmatically.
// Pass [] to opt out of filesystem setting sources.
const result = query({
prompt: "Review this PR",
options: {
settingSources: [],
agents: {
/\* ... \*/
},
mcpServers: {
/\* ... \*/
},
allowedTools: ["Read", "Grep", "Glob"]
}
});
`
```
**Loading CLAUDE.md project instructions:**
```
`// Load project settings to include CLAUDE.md files
const result = query({
prompt: "Add a new feature following project conventions",
options: {
systemPrompt: {
type: "preset",
preset: "claude\_code" // Use Claude Code's system prompt
},
settingSources: ["project"], // Loads CLAUDE.md from project directory
allowedTools: ["Read", "Write", "Edit"]
}
});
`
```
####
[​
](#settings-precedence)
Settings precedence
When multiple sources are loaded, settings are merged with this precedence (highest to lowest):
1. Local settings (`.claude/settings.local.json`)
2. Project settings (`.claude/settings.json`)
3. User settings (`\~/.claude/settings.json`)
Programmatic options such as `agents`, `allowedTools`, and `settings` override user, project, and local filesystem settings. Managed policy settings take precedence over programmatic options.
###
[​
](#permissionmode)
`PermissionMode`
```
`type PermissionMode =
| "default" // Standard permission behavior
| "acceptEdits" // Auto-accept file edits
| "bypassPermissions" // Bypass all permission checks
| "plan" // Planning mode - read-only tools only
| "dontAsk" // Don't prompt for permissions, deny if not pre-approved
| "auto"; // Use a model classifier to approve or deny each tool call
`
```
###
[​
](#canusetool)
`CanUseTool`
Custom permission function type for controlling tool usage.
```
`type CanUseTool = (
toolName: string,
input: Record\<string, unknown\>,
options: {
signal: AbortSignal;
suggestions?: PermissionUpdate[];
blockedPath?: string;
decisionReason?: string;
toolUseID: string;
agentID?: string;
}
) =\> Promise\<PermissionResult\>;
`
```
|Option|Type|Description|
|`signal`|`AbortSignal`|Signaled if the operation should be aborted|
|`suggestions`|[`PermissionUpdate`](#permissionupdate)`[]`|Suggested permission updates so the user is not prompted again for this tool. Bash prompts include a suggestion with the `localSettings`[destination](#permissionupdatedestination), so returning it in `updatedPermissions` writes the rule to `.claude/settings.local.json` and persists across sessions.|
|`blockedPath`|`string`|The file path that triggered the permission request, if applicable|
|`decisionReason`|`string`|Explains why this permission request was triggered|
|`toolUseID`|`string`|Unique identifier for this specific tool call within the assistant message|
|`agentID`|`string`|If running within a sub-agent, the sub-agent’s ID|
###
[​
](#permissionresult)
`PermissionResult`
Result of a permission check.
```
`type PermissionResult =
| {
behavior: "allow";
updatedInput?: Record\<string, unknown\>;
updatedPermissions?: PermissionUpdate[];
toolUseID?: string;
}
| {
behavior: "deny";
message: string;
interrupt?: boolean;
toolUseID?: string;
};
`
```
###
[​
](#toolconfig)
`ToolConfig`
Configuration for built-in tool behavior.
```
`type ToolConfig = {
askUserQuestion?: {
previewFormat?: "markdown" | "html";
};
};
`
```
|Field|Type|Description|
|`askUserQuestion.previewFormat`|`'markdown' | 'html'`|Opts into the `preview` field on [`AskUserQuestion`](/docs/en/agent-sdk/user-input#question-format) options and sets its content format. When unset, Claude does not emit previews|
###
[​
](#mcpserverconfig)
`McpServerConfig`
Configuration for MCP servers.
```
`type McpServerConfig =
| McpStdioServerConfig
| McpSSEServerConfig
| McpHttpServerConfig
| McpSdkServerConfigWithInstance;
`
```
####
[​
](#mcpstdioserverconfig)
`McpStdioServerConfig`
```
`type McpStdioServerConfig = {
type?: "stdio";
command: string;
args?: string[];
env?: Record\<string, string\>;
};
`
```
####
[​
](#mcpsseserverconfig)
`McpSSEServerConfig`
```
`type McpSSEServerConfig = {
type: "sse";
url: string;
headers?: Record\<string, string\>;
};
`
```
####
[​
](#mcphttpserverconfig)
`McpHttpServerConfig`
```
`type McpHttpServerConfig = {
type: "http";
url: string;
headers?: Record\<string, string\>;
};
`
```
####
[​
](#mcpsdkserverconfigwithinstance)
`McpSdkServerConfigWithInstance`
```
`type McpSdkServerConfigWithInstance = {
type: "sdk";
name: string;
instance: McpServer;
};
`
```
####
[​
](#mcpclaudeaiproxyserverconfig)
`McpClaudeAIProxyServerConfig`
```
`type McpClaudeAIProxyServerConfig = {
type: "claudeai-proxy";
url: string;
id: string;
};
`
```
###
[​
](#sdkpluginconfig)
`SdkPluginConfig`
Configuration for loading plugins in the SDK.
```
`type SdkPluginConfig = {
type: "local";
path: string;
};
`
```
|Field|Type|Description|
|`type`|`'local'`|Must be `'local'` (only local plugins currently supported)|
|`path`|`string`|Absolute or relative path to the plugin directory|
**Example:**
```
`plugins: [
{ type: "local", path: "./my-plugin" },
{ type: "local", path: "/absolute/path/to/plugin" }
];
`
```
For complete information on creating and using plugins, see [Plugins](/docs/en/agent-sdk/plugins).
##
[​
](#message-types)
Message Types
###
[​
](#sdkmessage)
`SDKMessage`
Union type of all possible messages returned by the query.
```
`type SDKMessage =
| SDKAssistantMessage
| SDKUserMessage
| SDKUserMessageReplay
| SDKResultMessage
| SDKSystemMessage
| SDKPartialAssistantMessage
| SDKCompactBoundaryMessage
| SDKStatusMessage
| SDKLocalCommandOutputMessage
| SDKHookStartedMessage
| SDKHookProgressMessage
| SDKHookResponseMessage
| SDKPluginInstallMessage
| SDKToolProgressMessage
| SDKAuthStatusMessage
| SDKTaskNotificationMessage
| SDKTaskStartedMessage
| SDKTaskProgressMessage
| SDKTaskUpdatedMessage
| SDKFilesPersistedEvent
| SDKToolUseSummaryMessage
| SDKRateLimitEvent
| SDKPermissionDeniedMessage
| SDKPromptSuggestionMessage;
`
```
###
[​
](#sdkassistantmessage)
`SDKAssistantMessage`
Assistant response message.
```
`type SDKAssistantMessage = {
type: "assistant";
uuid: UUID;
session\_id: string;
message: BetaMessage; // From Anthropic SDK
parent\_tool\_use\_id: string | null;
error?: SDKAssistantMessageError;
};
`
```
The `message` field is a [`BetaMessage`](https://platform.claude.com/docs/en/api/messages/create) from the Anthropic SDK. It includes fields like `id`, `content`, `model`, `stop\_reason`, and `usage`.
`SDKAssistantMessageError` is one of: `'authentication\_failed'`, `'oauth\_org\_not\_allowed'`, `'billing\_error'`, `'rate\_limit'`, `'invalid\_request'`, `'server\_error'`, `'max\_output\_tokens'`, or `'unknown'`.
###
[​
](#sdkusermessage)
`SDKUserMessage`
User input message.
```
`type SDKUserMessage = {
type: "user";
uuid?: UUID;
session\_id: string;
message: MessageParam; // From Anthropic SDK
parent\_tool\_use\_id: string | null;
isSynthetic?: boolean;
shouldQuery?: boolean;
tool\_use\_result?: unknown;
origin?: SDKMessageOrigin;
};
`
```
Set `shouldQuery` to `false` to append the message to the transcript without triggering an assistant turn. The message is held and merged into the next user message that does trigger a turn. Use this to inject context, such as the output of a command you ran out of band, without spending a model call on it.
###
[​
](#sdkusermessagereplay)
`SDKUserMessageReplay`
Replayed user message with required UUID.
```
`type SDKUserMessageReplay = {
type: "user";
uuid: UUID;
session\_id: string;
message: MessageParam;
parent\_tool\_use\_id: string | null;
isSynthetic?: boolean;
tool\_use\_result?: unknown;
origin?: SDKMessageOrigin;
isReplay: true;
};
`
```
###
[​
](#sdkresultmessage)
`SDKResultMessage`
Final result message.
```
`type SDKResultMessage =
| {
type: "result";
subtype: "success";
uuid: UUID;
session\_id: string;
duration\_ms: number;
duration\_api\_ms: number;
is\_error: boolean;
num\_turns: number;
result: string;
stop\_reason: string | null;
total\_cost\_usd: number;
usage: NonNullableUsage;
modelUsage: { [modelName: string]: ModelUsage };
permission\_denials: SDKPermissionDenial[];
structured\_output?: unknown;
deferred\_tool\_use?: { id: string; name: string; input: Record\<string, unknown\> };
origin?: SDKMessageOrigin;
}
| {
type: "result";
subtype:
| "error\_max\_turns"
| "error\_during\_execution"
| "error\_max\_budget\_usd"
| "error\_max\_structured\_output\_retries";
uuid: UUID;
session\_id: string;
duration\_ms: number;
duration\_api\_ms: number;
is\_error: boolean;
num\_turns: number;
stop\_reason: string | null;
total\_cost\_usd: number;
usage: NonNullableUsage;
modelUsage: { [modelName: string]: ModelUsage };
permission\_denials: SDKPermissionDenial[];
errors: string[];
origin?: SDKMessageOrigin;
};
`
```
The `origin` field forwards the [`SDKMessageOrigin`](#sdkmessageorigin) of the user message that triggered this result. When a background task finishes and the SDK injects a synthetic follow-up turn, the resulting `SDKResultMessage` carries `origin: { kind: "task-notification" }`. Check this field to distinguish results that answer your prompt from results emitted for background-task follow-ups, so you can route or suppress the latter. The field is absent for results emitted before any user turn, such as startup errors.
When a `PreToolUse` hook returns `permissionDecision: "defer"`, the result has `stop\_reason: "tool\_deferred"` and `deferred\_tool\_use` carries the pending tool’s `id`, `name`, and `input`. Read this field to surface the request in your own UI, then resume with the same `session\_id` to continue. See [Defer a tool call for later](/docs/en/hooks#defer-a-tool-call-for-later) for the full round trip.
###
[​
](#sdksystemmessage)
`SDKSystemMessage`
System initialization message.
```
`type SDKSystemMessage = {
type: "system";
subtype: "init";
uuid: UUID;
session\_id: string;
agents?: string[];
apiKeySource: ApiKeySource;
betas?: string[];
claude\_code\_version: string;
cwd: string;
tools: string[];
mcp\_servers: {
name: string;
status: string;
}[];
model: string;
permissionMode: PermissionMode;
slash\_commands: string[];
output\_style: string;
skills: string[];
plugins: { name: string; path: string }[];
};
`
```
###
[​
](#sdkpartialassistantmessage)
`SDKPartialAssistantMessage`
Streaming partial message (only when `includePartialMessages` is true).
```
`type SDKPartialAssistantMessage = {
type: "stream\_event";
event: BetaRawMessageStreamEvent; // From Anthropic SDK
parent\_tool\_use\_id: string | null;
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdkcompactboundarymessage)
`SDKCompactBoundaryMessage`
Message indicating a conversation compaction boundary.
```
`type SDKCompactBoundaryMessage = {
type: "system";
subtype: "compact\_boundary";
uuid: UUID;
session\_id: string;
compact\_metadata: {
trigger: "manual" | "auto";
pre\_tokens: number;
};
};
`
```
###
[​
](#sdkplugininstallmessage)
`SDKPluginInstallMessage`
Plugin installation progress event. Emitted when [`CLAUDE\_CODE\_SYNC\_PLUGIN\_INSTALL`](/docs/en/env-vars) is set, so your Agent SDK application can track marketplace plugin installation before the first turn. The `started` and `completed` statuses bracket the overall install. The `installed` and `failed` statuses report individual marketplaces and include `name`.
```
`type SDKPluginInstallMessage = {
type: "system";
subtype: "plugin\_install";
status: "started" | "installed" | "failed" | "completed";
name?: string;
error?: string;
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdkpermissiondeniedmessage)
`SDKPermissionDeniedMessage`
Stream event emitted when the permission system auto-denies a tool call without an interactive prompt. Use it to render the denial in your UI as it happens, rather than only observing the `is\_error` tool result that follows. The interactive ask path reaches your application separately through the [`canUseTool`](#canusetool) callback. Denials issued by a `PreToolUse` hook are not reported through this event.
This event requires Claude Code v2.1.136 or later.
```
`type SDKPermissionDeniedMessage = {
type: "system";
subtype: "permission\_denied";
tool\_name: string;
tool\_use\_id: string;
agent\_id?: string;
decision\_reason\_type?: string;
decision\_reason?: string;
message: string;
uuid: UUID;
session\_id: string;
};
`
```
|Field|Type|Description|
|`tool\_name`|`string`|Name of the tool that was denied|
|`tool\_use\_id`|`string`|ID of the `tool\_use` block this denial answers|
|`agent\_id`|`string`|Subagent ID when the denied call originated inside a subagent. Mirrors the field on `can\_use\_tool` for host-side routing|
|`decision\_reason\_type`|`string`|Discriminator for the component that decided, such as `"rule"`, `"mode"`, `"classifier"`, or `"asyncAgent"`|
|`decision\_reason`|`string`|Human-readable reason from the deciding component, when available|
|`message`|`string`|Rejection message returned to the model in the `tool\_result`|
###
[​
](#sdkpermissiondenial)
`SDKPermissionDenial`
Information about a denied tool use.
```
`type SDKPermissionDenial = {
tool\_name: string;
tool\_use\_id: string;
tool\_input: Record\<string, unknown\>;
};
`
```
###
[​
](#sdkmessageorigin)
`SDKMessageOrigin`
Provenance of a user-role message. This appears as `origin` on [`SDKUserMessage`](#sdkusermessage) and is forwarded onto the corresponding [`SDKResultMessage`](#sdkresultmessage) so you can tell what triggered a given turn.
```
`type SDKMessageOrigin =
| { kind: "human" }
| { kind: "channel"; server: string }
| { kind: "peer"; from: string; name?: string }
| { kind: "task-notification" }
| { kind: "coordinator" };
`
```
|`kind`|Meaning|
|`human`|Direct input from the end user. On user messages, an absent `origin` also means human input.|
|`channel`|Message arriving on a [channel](/docs/en/channels). `server` is the source MCP server name.|
|`peer`|Message from another agent session via `SendMessage`. `from` is the sender address; `name` is the sender’s display name when available.|
|`task-notification`|Synthetic turn injected after a background task finished. See [`SDKTaskNotificationMessage`](#sdktasknotificationmessage).|
|`coordinator`|Message from a team coordinator in an [agent team](/docs/en/agent-teams).|
##
[​
](#hook-types)
Hook Types
For a comprehensive guide on using hooks with examples and common patterns, see the [Hooks guide](/docs/en/agent-sdk/hooks).
###
[​
](#hookevent)
`HookEvent`
Available hook events.
```
`type HookEvent =
| "PreToolUse"
| "PostToolUse"
| "PostToolUseFailure"
| "PostToolBatch"
| "Notification"
| "UserPromptSubmit"
| "SessionStart"
| "SessionEnd"
| "Stop"
| "SubagentStart"
| "SubagentStop"
| "PreCompact"
| "PermissionRequest"
| "Setup"
| "TeammateIdle"
| "TaskCompleted"
| "ConfigChange"
| "WorktreeCreate"
| "WorktreeRemove";
`
```
###
[​
](#hookcallback)
`HookCallback`
Hook callback function type.
```
`type HookCallback = (
input: HookInput, // Union of all hook input types
toolUseID: string | undefined,
options: { signal: AbortSignal }
) =\> Promise\<HookJSONOutput\>;
`
```
###
[​
](#hookcallbackmatcher)
`HookCallbackMatcher`
Hook configuration with optional matcher.
```
`interface HookCallbackMatcher {
matcher?: string;
hooks: HookCallback[];
timeout?: number; // Timeout in seconds for all hooks in this matcher
}
`
```
###
[​
](#hookinput)
`HookInput`
Union type of all hook input types.
```
`type HookInput =
| PreToolUseHookInput
| PostToolUseHookInput
| PostToolUseFailureHookInput
| PostToolBatchHookInput
| NotificationHookInput
| UserPromptSubmitHookInput
| SessionStartHookInput
| SessionEndHookInput
| StopHookInput
| SubagentStartHookInput
| SubagentStopHookInput
| PreCompactHookInput
| PermissionRequestHookInput
| SetupHookInput
| TeammateIdleHookInput
| TaskCompletedHookInput
| ConfigChangeHookInput
| WorktreeCreateHookInput
| WorktreeRemoveHookInput;
`
```
###
[​
](#basehookinput)
`BaseHookInput`
Base interface that all hook input types extend.
```
`type BaseHookInput = {
session\_id: string;
transcript\_path: string;
cwd: string;
permission\_mode?: string;
effort?: { level: string };
agent\_id?: string;
agent\_type?: string;
};
`
```
####
[​
](#pretoolusehookinput)
`PreToolUseHookInput`
```
`type PreToolUseHookInput = BaseHookInput & {
hook\_event\_name: "PreToolUse";
tool\_name: string;
tool\_input: unknown;
tool\_use\_id: string;
};
`
```
####
[​
](#posttoolusehookinput)
`PostToolUseHookInput`
```
`type PostToolUseHookInput = BaseHookInput & {
hook\_event\_name: "PostToolUse";
tool\_name: string;
tool\_input: unknown;
tool\_response: unknown;
tool\_use\_id: string;
duration\_ms?: number;
};
`
```
####
[​
](#posttoolusefailurehookinput)
`PostToolUseFailureHookInput`
```
`type PostToolUseFailureHookInput = BaseHookInput & {
hook\_event\_name: "PostToolUseFailure";
tool\_name: string;
tool\_input: unknown;
tool\_use\_id: string;
error: string;
is\_interrupt?: boolean;
duration\_ms?: number;
};
`
```
####
[​
](#posttoolbatchhookinput)
`PostToolBatchHookInput`
Fires once after every tool call in a batch has resolved, before the next model request. `tool\_response` carries the serialized `tool\_result` content the model sees; the shape differs from `PostToolUseHookInput`’s structured `Output` object.
```
`type PostToolBatchHookInput = BaseHookInput & {
hook\_event\_name: "PostToolBatch";
tool\_calls: PostToolBatchToolCall[];
};
type PostToolBatchToolCall = {
tool\_name: string;
tool\_input: unknown;
tool\_use\_id: string;
tool\_response?: unknown;
};
`
```
####
[​
](#notificationhookinput)
`NotificationHookInput`
```
`type NotificationHookInput = BaseHookInput & {
hook\_event\_name: "Notification";
message: string;
title?: string;
notification\_type: string;
};
`
```
####
[​
](#userpromptsubmithookinput)
`UserPromptSubmitHookInput`
```
`type UserPromptSubmitHookInput = BaseHookInput & {
hook\_event\_name: "UserPromptSubmit";
prompt: string;
};
`
```
####
[​
](#sessionstarthookinput)
`SessionStartHookInput`
```
`type SessionStartHookInput = BaseHookInput & {
hook\_event\_name: "SessionStart";
source: "startup" | "resume" | "clear" | "compact";
agent\_type?: string;
model?: string;
};
`
```
####
[​
](#sessionendhookinput)
`SessionEndHookInput`
```
`type SessionEndHookInput = BaseHookInput & {
hook\_event\_name: "SessionEnd";
reason: ExitReason; // String from EXIT\_REASONS array
};
`
```
####
[​
](#stophookinput)
`StopHookInput`
```
`type StopHookInput = BaseHookInput & {
hook\_event\_name: "Stop";
stop\_hook\_active: boolean;
last\_assistant\_message?: string;
};
`
```
####
[​
](#subagentstarthookinput)
`SubagentStartHookInput`
```
`type SubagentStartHookInput = BaseHookInput & {
hook\_event\_name: "SubagentStart";
agent\_id: string;
agent\_type: string;
};
`
```
####
[​
](#subagentstophookinput)
`SubagentStopHookInput`
```
`type SubagentStopHookInput = BaseHookInput & {
hook\_event\_name: "SubagentStop";
stop\_hook\_active: boolean;
agent\_id: string;
agent\_transcript\_path: string;
agent\_type: string;
last\_assistant\_message?: string;
};
`
```
####
[​
](#precompacthookinput)
`PreCompactHookInput`
```
`type PreCompactHookInput = BaseHookInput & {
hook\_event\_name: "PreCompact";
trigger: "manual" | "auto";
custom\_instructions: string | null;
};
`
```
####
[​
](#permissionrequesthookinput)
`PermissionRequestHookInput`
```
`type PermissionRequestHookInput = BaseHookInput & {
hook\_event\_name: "PermissionRequest";
tool\_name: string;
tool\_input: unknown;
permission\_suggestions?: PermissionUpdate[];
};
`
```
####
[​
](#setuphookinput)
`SetupHookInput`
```
`type SetupHookInput = BaseHookInput & {
hook\_event\_name: "Setup";
trigger: "init" | "maintenance";
};
`
```
####
[​
](#teammateidlehookinput)
`TeammateIdleHookInput`
```
`type TeammateIdleHookInput = BaseHookInput & {
hook\_event\_name: "TeammateIdle";
teammate\_name: string;
team\_name: string;
};
`
```
####
[​
](#taskcompletedhookinput)
`TaskCompletedHookInput`
```
`type TaskCompletedHookInput = BaseHookInput & {
hook\_event\_name: "TaskCompleted";
task\_id: string;
task\_subject: string;
task\_description?: string;
teammate\_name?: string;
team\_name?: string;
};
`
```
####
[​
](#configchangehookinput)
`ConfigChangeHookInput`
```
`type ConfigChangeHookInput = BaseHookInput & {
hook\_event\_name: "ConfigChange";
source:
| "user\_settings"
| "project\_settings"
| "local\_settings"
| "policy\_settings"
| "skills";
file\_path?: string;
};
`
```
####
[​
](#worktreecreatehookinput)
`WorktreeCreateHookInput`
```
`type WorktreeCreateHookInput = BaseHookInput & {
hook\_event\_name: "WorktreeCreate";
name: string;
};
`
```
####
[​
](#worktreeremovehookinput)
`WorktreeRemoveHookInput`
```
`type WorktreeRemoveHookInput = BaseHookInput & {
hook\_event\_name: "WorktreeRemove";
worktree\_path: string;
};
`
```
###
[​
](#hookjsonoutput)
`HookJSONOutput`
Hook return value.
```
`type HookJSONOutput = AsyncHookJSONOutput | SyncHookJSONOutput;
`
```
####
[​
](#asynchookjsonoutput)
`AsyncHookJSONOutput`
```
`type AsyncHookJSONOutput = {
async: true;
asyncTimeout?: number;
};
`
```
####
[​
](#synchookjsonoutput)
`SyncHookJSONOutput`
```
`type SyncHookJSONOutput = {
continue?: boolean;
suppressOutput?: boolean;
stopReason?: string;
decision?: "approve" | "block";
systemMessage?: string;
reason?: string;
hookSpecificOutput?:
| {
hookEventName: "PreToolUse";
permissionDecision?: "allow" | "deny" | "ask" | "defer";
permissionDecisionReason?: string;
updatedInput?: Record\<string, unknown\>;
additionalContext?: string;
}
| {
hookEventName: "UserPromptSubmit";
additionalContext?: string;
}
| {
hookEventName: "SessionStart";
additionalContext?: string;
}
| {
hookEventName: "Setup";
additionalContext?: string;
}
| {
hookEventName: "SubagentStart";
additionalContext?: string;
}
| {
hookEventName: "PostToolUse";
additionalContext?: string;
updatedToolOutput?: unknown;
/\*\* @deprecated Use `updatedToolOutput`, which works for all tools. \*/
updatedMCPToolOutput?: unknown;
}
| {
hookEventName: "PostToolUseFailure";
additionalContext?: string;
}
| {
hookEventName: "PostToolBatch";
additionalContext?: string;
}
| {
hookEventName: "Notification";
additionalContext?: string;
}
| {
hookEventName: "PermissionRequest";
decision:
| {
behavior: "allow";
updatedInput?: Record\<string, unknown\>;
updatedPermissions?: PermissionUpdate[];
}
| {
behavior: "deny";
message?: string;
interrupt?: boolean;
};
};
};
`
```
##
[​
](#tool-input-types)
Tool Input Types
Documentation of input schemas for all built-in Claude Code tools. These types are exported from `@anthropic-ai/claude-agent-sdk` and can be used for type-safe tool interactions.
###
[​
](#toolinputschemas)
`ToolInputSchemas`
Union of all tool input types, exported from `@anthropic-ai/claude-agent-sdk`.
```
`type ToolInputSchemas =
| AgentInput
| AskUserQuestionInput
| BashInput
| TaskOutputInput
| EnterWorktreeInput
| ExitPlanModeInput
| FileEditInput
| FileReadInput
| FileWriteInput
| GlobInput
| GrepInput
| ListMcpResourcesInput
| McpInput
| MonitorInput
| NotebookEditInput
| ReadMcpResourceInput
| SubscribeMcpResourceInput
| SubscribePollingInput
| TaskStopInput
| TodoWriteInput
| UnsubscribeMcpResourceInput
| UnsubscribePollingInput
| WebFetchInput
| WebSearchInput;
`
```
###
[​
](#agent)
Agent
**Tool name:** `Agent` (previously `Task`, which is still accepted as an alias)
```
`type AgentInput = {
description: string;
prompt: string;
subagent\_type: string;
model?: "sonnet" | "opus" | "haiku";
resume?: string;
run\_in\_background?: boolean;
max\_turns?: number;
name?: string;
team\_name?: string;
mode?: "acceptEdits" | "bypassPermissions" | "default" | "dontAsk" | "plan";
isolation?: "worktree";
};
`
```
Launches a new agent to handle complex, multi-step tasks autonomously.
###
[​
](#askuserquestion)
AskUserQuestion
**Tool name:** `AskUserQuestion`
```
`type AskUserQuestionInput = {
questions: Array\<{
question: string;
header: string;
options: Array\<{ label: string; description: string; preview?: string }\>;
multiSelect: boolean;
}\>;
};
`
```
Asks the user clarifying questions during execution. See [Handle approvals and user input](/docs/en/agent-sdk/user-input#handle-clarifying-questions) for usage details.
###
[​
](#bash)
Bash
**Tool name:** `Bash`
```
`type BashInput = {
command: string;
timeout?: number;
description?: string;
run\_in\_background?: boolean;
dangerouslyDisableSandbox?: boolean;
};
`
```
Executes bash commands in a persistent shell session with optional timeout and background execution.
###
[​
](#monitor)
Monitor
**Tool name:** `Monitor`
```
`type MonitorInput = {
command: string;
description: string;
timeout\_ms?: number;
persistent?: boolean;
};
`
```
Runs a background script and delivers each stdout line to Claude as an event so it can react without polling. Set `persistent: true` for session-length watches such as log tails. Monitor follows the same permission rules as Bash. See the [Monitor tool reference](/docs/en/tools-reference#monitor-tool) for behavior and provider availability.
###
[​
](#taskoutput)
TaskOutput
**Tool name:** `TaskOutput`
```
`type TaskOutputInput = {
task\_id: string;
block: boolean;
timeout: number;
};
`
```
Retrieves output from a running or completed background task.
###
[​
](#edit)
Edit
**Tool name:** `Edit`
```
`type FileEditInput = {
file\_path: string;
old\_string: string;
new\_string: string;
replace\_all?: boolean;
};
`
```
Performs exact string replacements in files.
###
[​
](#read)
Read
**Tool name:** `Read`
```
`type FileReadInput = {
file\_path: string;
offset?: number;
limit?: number;
pages?: string;
};
`
```
Reads files from the local filesystem, including text, images, PDFs, and Jupyter notebooks. Use `pages` for PDF page ranges (for example, `"1-5"`).
###
[​
](#write)
Write
**Tool name:** `Write`
```
`type FileWriteInput = {
file\_path: string;
content: string;
};
`
```
Writes a file to the local filesystem, overwriting if it exists.
###
[​
](#glob)
Glob
**Tool name:** `Glob`
```
`type GlobInput = {
pattern: string;
path?: string;
};
`
```
Fast file pattern matching that works with any codebase size.
###
[​
](#grep)
Grep
**Tool name:** `Grep`
```
`type GrepInput = {
pattern: string;
path?: string;
glob?: string;
type?: string;
output\_mode?: "content" | "files\_with\_matches" | "count";
"-i"?: boolean;
"-n"?: boolean;
"-B"?: number;
"-A"?: number;
"-C"?: number;
context?: number;
head\_limit?: number;
offset?: number;
multiline?: boolean;
};
`
```
Powerful search tool built on ripgrep with regex support.
###
[​
](#taskstop)
TaskStop
**Tool name:** `TaskStop`
```
`type TaskStopInput = {
task\_id?: string;
shell\_id?: string; // Deprecated: use task\_id
};
`
```
Stops a running background task or shell by ID.
###
[​
](#notebookedit)
NotebookEdit
**Tool name:** `NotebookEdit`
```
`type NotebookEditInput = {
notebook\_path: string;
cell\_id?: string;
new\_source: string;
cell\_type?: "code" | "markdown";
edit\_mode?: "replace" | "insert" | "delete";
};
`
```
Edits cells in Jupyter notebook files.
###
[​
](#webfetch)
WebFetch
**Tool name:** `WebFetch`
```
`type WebFetchInput = {
url: string;
prompt: string;
};
`
```
Fetches content from a URL and processes it with an AI model.
###
[​
](#websearch)
WebSearch
**Tool name:** `WebSearch`
```
`type WebSearchInput = {
query: string;
allowed\_domains?: string[];
blocked\_domains?: string[];
};
`
```
Searches the web and returns formatted results.
###
[​
](#todowrite)
TodoWrite
**Tool name:** `TodoWrite`
```
`type TodoWriteInput = {
todos: Array\<{
content: string;
status: "pending" | "in\_progress" | "completed";
activeForm: string;
}\>;
};
`
```
Creates and manages a structured task list for tracking progress.
###
[​
](#exitplanmode)
ExitPlanMode
**Tool name:** `ExitPlanMode`
```
`type ExitPlanModeInput = {
allowedPrompts?: Array\<{
tool: "Bash";
prompt: string;
}\>;
};
`
```
Exits planning mode. Optionally specifies prompt-based permissions needed to implement the plan.
###
[​
](#listmcpresources)
ListMcpResources
**Tool name:** `ListMcpResources`
```
`type ListMcpResourcesInput = {
server?: string;
};
`
```
Lists available MCP resources from connected servers.
###
[​
](#readmcpresource)
ReadMcpResource
**Tool name:** `ReadMcpResource`
```
`type ReadMcpResourceInput = {
server: string;
uri: string;
};
`
```
Reads a specific MCP resource from a server.
###
[​
](#enterworktree)
EnterWorktree
**Tool name:** `EnterWorktree`
```
`type EnterWorktreeInput = {
name?: string;
path?: string;
};
`
```
Creates and enters a temporary git worktree for isolated work. Pass `path` to switch into an existing worktree of the current repository instead of creating a new one. `name` and `path` are mutually exclusive.
##
[​
](#tool-output-types)
Tool Output Types
Documentation of output schemas for all built-in Claude Code tools. These types are exported from `@anthropic-ai/claude-agent-sdk` and represent the actual response data returned by each tool.
###
[​
](#tooloutputschemas)
`ToolOutputSchemas`
Union of all tool output types.
```
`type ToolOutputSchemas =
| AgentOutput
| AskUserQuestionOutput
| BashOutput
| EnterWorktreeOutput
| ExitPlanModeOutput
| FileEditOutput
| FileReadOutput
| FileWriteOutput
| GlobOutput
| GrepOutput
| ListMcpResourcesOutput
| MonitorOutput
| NotebookEditOutput
| ReadMcpResourceOutput
| TaskStopOutput
| TodoWriteOutput
| WebFetchOutput
| WebSearchOutput;
`
```
###
[​
](#agent-2)
Agent
**Tool name:** `Agent` (previously `Task`, which is still accepted as an alias)
```
`type AgentOutput =
| {
status: "completed";
agentId: string;
content: Array\<{ type: "text"; text: string }\>;
totalToolUseCount: number;
totalDurationMs: number;
totalTokens: number;
usage: {
input\_tokens: number;
output\_tokens: number;
cache\_creation\_input\_tokens: number | null;
cache\_read\_input\_tokens: number | null;
server\_tool\_use: {
web\_search\_requests: number;
web\_fetch\_requests: number;
} | null;
service\_tier: ("standard" | "priority" | "batch") | null;
cache\_creation: {
ephemeral\_1h\_input\_tokens: number;
ephemeral\_5m\_input\_tokens: number;
} | null;
};
prompt: string;
}
| {
status: "async\_launched";
agentId: string;
description: string;
prompt: string;
outputFile: string;
canReadOutputFile?: boolean;
}
| {
status: "sub\_agent\_entered";
description: string;
message: string;
};
`
```
Returns the result from the subagent. Discriminated on the `status` field: `"completed"` for finished tasks, `"async\_launched"` for background tasks, and `"sub\_agent\_entered"` for interactive subagents.
###
[​
](#askuserquestion-2)
AskUserQuestion
**Tool name:** `AskUserQuestion`
```
`type AskUserQuestionOutput = {
questions: Array\<{
question: string;
header: string;
options: Array\<{ label: string; description: string; preview?: string }\>;
multiSelect: boolean;
}\>;
answers: Record\<string, string\>;
};
`
```
Returns the questions asked and the user’s answers.
###
[​
](#bash-2)
Bash
**Tool name:** `Bash`
```
`type BashOutput = {
stdout: string;
stderr: string;
rawOutputPath?: string;
interrupted: boolean;
isImage?: boolean;
backgroundTaskId?: string;
backgroundedByUser?: boolean;
dangerouslyDisableSandbox?: boolean;
returnCodeInterpretation?: string;
structuredContent?: unknown[];
persistedOutputPath?: string;
persistedOutputSize?: number;
};
`
```
Returns command output with stdout/stderr split. Background commands include a `backgroundTaskId`.
###
[​
](#monitor-2)
Monitor
**Tool name:** `Monitor`
```
`type MonitorOutput = {
taskId: string;
timeoutMs: number;
persistent?: boolean;
};
`
```
Returns the background task ID for the running monitor. Use this ID with `TaskStop` to cancel the watch early.
###
[​
](#edit-2)
Edit
**Tool name:** `Edit`
```
`type FileEditOutput = {
filePath: string;
oldString: string;
newString: string;
originalFile: string;
structuredPatch: Array\<{
oldStart: number;
oldLines: number;
newStart: number;
newLines: number;
lines: string[];
}\>;
userModified: boolean;
replaceAll: boolean;
gitDiff?: {
filename: string;
status: "modified" | "added";
additions: number;
deletions: number;
changes: number;
patch: string;
};
};
`
```
Returns the structured diff of the edit operation.
###
[​
](#read-2)
Read
**Tool name:** `Read`
```
`type FileReadOutput =
| {
type: "text";
file: {
filePath: string;
content: string;
numLines: number;
startLine: number;
totalLines: number;
};
}
| {
type: "image";
file: {
base64: string;
type: "image/jpeg" | "image/png" | "image/gif" | "image/webp";
originalSize: number;
dimensions?: {
originalWidth?: number;
originalHeight?: number;
displayWidth?: number;
displayHeight?: number;
};
};
}
| {
type: "notebook";
file: {
filePath: string;
cells: unknown[];
};
}
| {
type: "pdf";
file: {
filePath: string;
base64: string;
originalSize: number;
};
}
| {
type: "parts";
file: {
filePath: string;
originalSize: number;
count: number;
outputDir: string;
};
};
`
```
Returns file contents in a format appropriate to the file type. Discriminated on the `type` field.
###
[​
](#write-2)
Write
**Tool name:** `Write`
```
`type FileWriteOutput = {
type: "create" | "update";
filePath: string;
content: string;
structuredPatch: Array\<{
oldStart: number;
oldLines: number;
newStart: number;
newLines: number;
lines: string[];
}\>;
originalFile: string | null;
gitDiff?: {
filename: string;
status: "modified" | "added";
additions: number;
deletions: number;
changes: number;
patch: string;
};
};
`
```
Returns the write result with structured diff information.
###
[​
](#glob-2)
Glob
**Tool name:** `Glob`
```
`type GlobOutput = {
durationMs: number;
numFiles: number;
filenames: string[];
truncated: boolean;
};
`
```
Returns file paths matching the glob pattern, sorted by modification time.
###
[​
](#grep-2)
Grep
**Tool name:** `Grep`
```
`type GrepOutput = {
mode?: "content" | "files\_with\_matches" | "count";
numFiles: number;
filenames: string[];
content?: string;
numLines?: number;
numMatches?: number;
appliedLimit?: number;
appliedOffset?: number;
};
`
```
Returns search results. The shape varies by `mode`: file list, content with matches, or match counts.
###
[​
](#taskstop-2)
TaskStop
**Tool name:** `TaskStop`
```
`type TaskStopOutput = {
message: string;
task\_id: string;
task\_type: string;
command?: string;
};
`
```
Returns confirmation after stopping the background task.
###
[​
](#notebookedit-2)
NotebookEdit
**Tool name:** `NotebookEdit`
```
`type NotebookEditOutput = {
new\_source: string;
cell\_id?: string;
cell\_type: "code" | "markdown";
language: string;
edit\_mode: string;
error?: string;
notebook\_path: string;
original\_file: string;
updated\_file: string;
};
`
```
Returns the result of the notebook edit with original and updated file contents.
###
[​
](#webfetch-2)
WebFetch
**Tool name:** `WebFetch`
```
`type WebFetchOutput = {
bytes: number;
code: number;
codeText: string;
result: string;
durationMs: number;
url: string;
};
`
```
Returns the fetched content with HTTP status and metadata.
###
[​
](#websearch-2)
WebSearch
**Tool name:** `WebSearch`
```
`type WebSearchOutput = {
query: string;
results: Array\<
| {
tool\_use\_id: string;
content: Array\<{ title: string; url: string }\>;
}
| string
\>;
durationSeconds: number;
};
`
```
Returns search results from the web.
###
[​
](#todowrite-2)
TodoWrite
**Tool name:** `TodoWrite`
```
`type TodoWriteOutput = {
oldTodos: Array\<{
content: string;
status: "pending" | "in\_progress" | "completed";
activeForm: string;
}\>;
newTodos: Array\<{
content: string;
status: "pending" | "in\_progress" | "completed";
activeForm: string;
}\>;
};
`
```
Returns the previous and updated task lists.
###
[​
](#exitplanmode-2)
ExitPlanMode
**Tool name:** `ExitPlanMode`
```
`type ExitPlanModeOutput = {
plan: string | null;
isAgent: boolean;
filePath?: string;
hasTaskTool?: boolean;
awaitingLeaderApproval?: boolean;
requestId?: string;
};
`
```
Returns the plan state after exiting plan mode.
###
[​
](#listmcpresources-2)
ListMcpResources
**Tool name:** `ListMcpResources`
```
`type ListMcpResourcesOutput = Array\<{
uri: string;
name: string;
mimeType?: string;
description?: string;
server: string;
}\>;
`
```
Returns an array of available MCP resources.
###
[​
](#readmcpresource-2)
ReadMcpResource
**Tool name:** `ReadMcpResource`
```
`type ReadMcpResourceOutput = {
contents: Array\<{
uri: string;
mimeType?: string;
text?: string;
}\>;
};
`
```
Returns the contents of the requested MCP resource.
###
[​
](#enterworktree-2)
EnterWorktree
**Tool name:** `EnterWorktree`
```
`type EnterWorktreeOutput = {
worktreePath: string;
worktreeBranch?: string;
message: string;
};
`
```
Returns information about the git worktree.
##
[​
](#permission-types)
Permission Types
###
[​
](#permissionupdate)
`PermissionUpdate`
Operations for updating permissions.
```
`type PermissionUpdate =
| {
type: "addRules";
rules: PermissionRuleValue[];
behavior: PermissionBehavior;
destination: PermissionUpdateDestination;
}
| {
type: "replaceRules";
rules: PermissionRuleValue[];
behavior: PermissionBehavior;
destination: PermissionUpdateDestination;
}
| {
type: "removeRules";
rules: PermissionRuleValue[];
behavior: PermissionBehavior;
destination: PermissionUpdateDestination;
}
| {
type: "setMode";
mode: PermissionMode;
destination: PermissionUpdateDestination;
}
| {
type: "addDirectories";
directories: string[];
destination: PermissionUpdateDestination;
}
| {
type: "removeDirectories";
directories: string[];
destination: PermissionUpdateDestination;
};
`
```
###
[​
](#permissionbehavior)
`PermissionBehavior`
```
`type PermissionBehavior = "allow" | "deny" | "ask";
`
```
###
[​
](#permissionupdatedestination)
`PermissionUpdateDestination`
```
`type PermissionUpdateDestination =
| "userSettings" // Global user settings
| "projectSettings" // Per-directory project settings
| "localSettings" // Gitignored local settings
| "session" // Current session only
| "cliArg"; // CLI argument
`
```
###
[​
](#permissionrulevalue)
`PermissionRuleValue`
```
`type PermissionRuleValue = {
toolName: string;
ruleContent?: string;
};
`
```
##
[​
](#other-types)
Other Types
###
[​
](#apikeysource)
`ApiKeySource`
```
`type ApiKeySource = "user" | "project" | "org" | "temporary" | "oauth";
`
```
###
[​
](#sdkbeta)
`SdkBeta`
Available beta features that can be enabled via the `betas` option. See [Beta headers](https://platform.claude.com/docs/en/api/beta-headers) for more information.
```
`type SdkBeta = "context-1m-2025-08-07";
`
```
The `context-1m-2025-08-07` beta is retired as of April 30, 2026. Passing this value with Claude Sonnet 4.5 or Sonnet 4 has no effect, and requests that exceed the standard 200k-token context window return an error. To use a 1M-token context window, migrate to [Claude Sonnet 4.6, Claude Opus 4.6, or Claude Opus 4.7](https://platform.claude.com/docs/en/about-claude/models/overview), which include 1M context at standard pricing with no beta header required.
###
[​
](#slashcommand)
`SlashCommand`
Information about an available slash command.
```
`type SlashCommand = {
name: string;
description: string;
argumentHint: string;
aliases?: string[];
};
`
```
###
[​
](#modelinfo)
`ModelInfo`
Information about an available model.
```
`type ModelInfo = {
value: string;
displayName: string;
description: string;
supportsEffort?: boolean;
supportedEffortLevels?: ("low" | "medium" | "high" | "xhigh" | "max")[];
supportsAdaptiveThinking?: boolean;
supportsFastMode?: boolean;
};
`
```
###
[​
](#agentinfo)
`AgentInfo`
Information about an available subagent that can be invoked via the Agent tool.
```
`type AgentInfo = {
name: string;
description: string;
model?: string;
};
`
```
|Field|Type|Description|
|`name`|`string`|Agent type identifier (e.g., `"Explore"`, `"general-purpose"`)|
|`description`|`string`|Description of when to use this agent|
|`model`|`string | undefined`|Model alias this agent uses. If omitted, inherits the parent’s model|
###
[​
](#mcpserverstatus)
`McpServerStatus`
Status of a connected MCP server.
```
`type McpServerStatus = {
name: string;
status: "connected" | "failed" | "needs-auth" | "pending" | "disabled";
serverInfo?: {
name: string;
version: string;
};
error?: string;
config?: McpServerStatusConfig;
scope?: string;
tools?: {
name: string;
description?: string;
annotations?: {
readOnly?: boolean;
destructive?: boolean;
openWorld?: boolean;
};
}[];
};
`
```
###
[​
](#mcpserverstatusconfig)
`McpServerStatusConfig`
The configuration of an MCP server as reported by `mcpServerStatus()`. This is the union of all MCP server transport types.
```
`type McpServerStatusConfig =
| McpStdioServerConfig
| McpSSEServerConfig
| McpHttpServerConfig
| McpSdkServerConfig
| McpClaudeAIProxyServerConfig;
`
```
See [`McpServerConfig`](#mcpserverconfig) for details on each transport type.
###
[​
](#accountinfo)
`AccountInfo`
Account information for the authenticated user.
```
`type AccountInfo = {
email?: string;
organization?: string;
subscriptionType?: string;
tokenSource?: string;
apiKeySource?: string;
};
`
```
###
[​
](#modelusage)
`ModelUsage`
Per-model usage statistics returned in result messages. The `costUSD` value is a client-side estimate. See [Track cost and usage](/docs/en/agent-sdk/cost-tracking) for billing caveats.
```
`type ModelUsage = {
inputTokens: number;
outputTokens: number;
cacheReadInputTokens: number;
cacheCreationInputTokens: number;
webSearchRequests: number;
costUSD: number;
contextWindow: number;
maxOutputTokens: number;
};
`
```
###
[​
](#configscope)
`ConfigScope`
```
`type ConfigScope = "local" | "user" | "project";
`
```
###
[​
](#nonnullableusage)
`NonNullableUsage`
A version of [`Usage`](#usage) with all nullable fields made non-nullable.
```
`type NonNullableUsage = {
[K in keyof Usage]: NonNullable\<Usage[K]\>;
};
`
```
###
[​
](#usage)
`Usage`
Token usage statistics (from `@anthropic-ai/sdk`).
```
`type Usage = {
input\_tokens: number | null;
output\_tokens: number | null;
cache\_creation\_input\_tokens?: number | null;
cache\_read\_input\_tokens?: number | null;
};
`
```
###
[​
](#calltoolresult)
`CallToolResult`
MCP tool result type (from `@modelcontextprotocol/sdk/types.js`). `structuredContent` is a JSON object that can be returned alongside `content`, including image blocks. See [Return structured data](/docs/en/agent-sdk/custom-tools#return-structured-data).
```
`type CallToolResult = {
content: Array\<{
type: "text" | "image" | "resource";
// Additional fields vary by type
}\>;
structuredContent?: Record\<string, unknown\>;
isError?: boolean;
};
`
```
###
[​
](#thinkingconfig)
`ThinkingConfig`
Controls Claude’s thinking/reasoning behavior. Takes precedence over the deprecated `maxThinkingTokens`.
```
`type ThinkingConfig =
| { type: "adaptive" } // The model determines when and how much to reason (Opus 4.6+)
| { type: "enabled"; budgetTokens?: number } // Fixed thinking token budget
| { type: "disabled" }; // No extended thinking
`
```
###
[​
](#spawnedprocess)
`SpawnedProcess`
Interface for custom process spawning (used with `spawnClaudeCodeProcess` option). `ChildProcess` already satisfies this interface.
```
`interface SpawnedProcess {
stdin: Writable;
stdout: Readable;
readonly killed: boolean;
readonly exitCode: number | null;
kill(signal: NodeJS.Signals): boolean;
on(
event: "exit",
listener: (code: number | null, signal: NodeJS.Signals | null) =\> void
): void;
on(event: "error", listener: (error: Error) =\> void): void;
once(
event: "exit",
listener: (code: number | null, signal: NodeJS.Signals | null) =\> void
): void;
once(event: "error", listener: (error: Error) =\> void): void;
off(
event: "exit",
listener: (code: number | null, signal: NodeJS.Signals | null) =\> void
): void;
off(event: "error", listener: (error: Error) =\> void): void;
}
`
```
###
[​
](#spawnoptions)
`SpawnOptions`
Options passed to the custom spawn function.
```
`interface SpawnOptions {
command: string;
args: string[];
cwd?: string;
env: Record\<string, string | undefined\>;
signal: AbortSignal;
}
`
```
###
[​
](#mcpsetserversresult)
`McpSetServersResult`
Result of a `setMcpServers()` operation.
```
`type McpSetServersResult = {
added: string[];
removed: string[];
errors: Record\<string, string\>;
};
`
```
###
[​
](#rewindfilesresult)
`RewindFilesResult`
Result of a `rewindFiles()` operation.
```
`type RewindFilesResult = {
canRewind: boolean;
error?: string;
filesChanged?: string[];
insertions?: number;
deletions?: number;
};
`
```
###
[​
](#sdkstatusmessage)
`SDKStatusMessage`
Status update message (e.g., compacting).
```
`type SDKStatusMessage = {
type: "system";
subtype: "status";
status: "compacting" | null;
permissionMode?: PermissionMode;
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdktasknotificationmessage)
`SDKTaskNotificationMessage`
Notification when a background task completes, fails, or is stopped. Background tasks include `run\_in\_background` Bash commands, [Monitor](#monitor) watches, and background subagents.
```
`type SDKTaskNotificationMessage = {
type: "system";
subtype: "task\_notification";
task\_id: string;
tool\_use\_id?: string;
status: "completed" | "failed" | "stopped";
output\_file: string;
summary: string;
usage?: {
total\_tokens: number;
tool\_uses: number;
duration\_ms: number;
};
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdktoolusesummarymessage)
`SDKToolUseSummaryMessage`
Summary of tool usage in a conversation.
```
`type SDKToolUseSummaryMessage = {
type: "tool\_use\_summary";
summary: string;
preceding\_tool\_use\_ids: string[];
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdkhookstartedmessage)
`SDKHookStartedMessage`
Emitted when a hook begins executing.
```
`type SDKHookStartedMessage = {
type: "system";
subtype: "hook\_started";
hook\_id: string;
hook\_name: string;
hook\_event: string;
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdkhookprogressmessage)
`SDKHookProgressMessage`
Emitted while a hook is running, with stdout/stderr output.
```
`type SDKHookProgressMessage = {
type: "system";
subtype: "hook\_progress";
hook\_id: string;
hook\_name: string;
hook\_event: string;
stdout: string;
stderr: string;
output: string;
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdkhookresponsemessage)
`SDKHookResponseMessage`
Emitted when a hook finishes executing.
```
`type SDKHookResponseMessage = {
type: "system";
subtype: "hook\_response";
hook\_id: string;
hook\_name: string;
hook\_event: string;
output: string;
stdout: string;
stderr: string;
exit\_code?: number;
outcome: "success" | "error" | "cancelled";
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdktoolprogressmessage)
`SDKToolProgressMessage`
Emitted periodically while a tool is executing to indicate progress.
```
`type SDKToolProgressMessage = {
type: "tool\_progress";
tool\_use\_id: string;
tool\_name: string;
parent\_tool\_use\_id: string | null;
elapsed\_time\_seconds: number;
task\_id?: string;
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdkauthstatusmessage)
`SDKAuthStatusMessage`
Emitted during authentication flows.
```
`type SDKAuthStatusMessage = {
type: "auth\_status";
isAuthenticating: boolean;
output: string[];
error?: string;
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdktaskstartedmessage)
`SDKTaskStartedMessage`
Emitted when a background task begins. The `task\_type` field is `"local\_bash"` for background Bash commands and [Monitor](#monitor) watches, `"local\_agent"` for subagents, or `"remote\_agent"`.
```
`type SDKTaskStartedMessage = {
type: "system";
subtype: "task\_started";
task\_id: string;
tool\_use\_id?: string;
description: string;
task\_type?: string;
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdktaskprogressmessage)
`SDKTaskProgressMessage`
Emitted periodically while a background task is running.
```
`type SDKTaskProgressMessage = {
type: "system";
subtype: "task\_progress";
task\_id: string;
tool\_use\_id?: string;
description: string;
usage: {
total\_tokens: number;
tool\_uses: number;
duration\_ms: number;
};
last\_tool\_name?: string;
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdktaskupdatedmessage)
`SDKTaskUpdatedMessage`
Emitted when a background task’s state changes, such as when it transitions from `running` to `completed`. Merge `patch` into your local task map keyed by `task\_id`. The `end\_time` field is a Unix epoch timestamp in milliseconds, comparable with `Date.now()`.
```
`type SDKTaskUpdatedMessage = {
type: "system";
subtype: "task\_updated";
task\_id: string;
patch: {
status?: "pending" | "running" | "completed" | "failed" | "killed";
description?: string;
end\_time?: number;
total\_paused\_ms?: number;
error?: string;
is\_backgrounded?: boolean;
};
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdkfilespersistedevent)
`SDKFilesPersistedEvent`
Emitted when file checkpoints are persisted to disk.
```
`type SDKFilesPersistedEvent = {
type: "system";
subtype: "files\_persisted";
files: { filename: string; file\_id: string }[];
failed: { filename: string; error: string }[];
processed\_at: string;
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdkratelimitevent)
`SDKRateLimitEvent`
Emitted when the session encounters a rate limit.
```
`type SDKRateLimitEvent = {
type: "rate\_limit\_event";
rate\_limit\_info: {
status: "allowed" | "allowed\_warning" | "rejected";
resetsAt?: number;
utilization?: number;
};
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdklocalcommandoutputmessage)
`SDKLocalCommandOutputMessage`
Output from a local slash command (for example, `/voice` or `/usage`). Displayed as assistant-style text in the transcript.
```
`type SDKLocalCommandOutputMessage = {
type: "system";
subtype: "local\_command\_output";
content: string;
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#sdkpromptsuggestionmessage)
`SDKPromptSuggestionMessage`
Emitted after each turn when `promptSuggestions` is enabled. Contains a predicted next user prompt.
```
`type SDKPromptSuggestionMessage = {
type: "prompt\_suggestion";
suggestion: string;
uuid: UUID;
session\_id: string;
};
`
```
###
[​
](#aborterror)
`AbortError`
Custom error class for abort operations.
```
`class AbortError extends Error {}
`
```
##
[​
](#sandbox-configuration)
Sandbox Configuration
###
[​
](#sandboxsettings)
`SandboxSettings`
Configuration for sandbox behavior. Use this to enable command sandboxing and configure network restrictions programmatically.
```
`type SandboxSettings = {
enabled?: boolean;
autoAllowBashIfSandboxed?: boolean;
excludedCommands?: string[];
allowUnsandboxedCommands?: boolean;
network?: SandboxNetworkConfig;
filesystem?: SandboxFilesystemConfig;
ignoreViolations?: Record\<string, string[]\>;
enableWeakerNestedSandbox?: boolean;
ripgrep?: { command: string; args?: string[] };
};
`
```
|Property|Type|Default|Description|
|`enabled`|`boolean`|`false`|Enable sandbox mode for command execution|
|`autoAllowBashIfSandboxed`|`boolean`|`true`|Auto-approve bash commands when sandbox is enabled|
|`excludedCommands`|`string[]`|`[]`|Commands that always bypass sandbox restrictions (e.g., `['docker']`). These run unsandboxed automatically without model involvement|
|`allowUnsandboxedCommands`|`boolean`|`true`|Allow the model to request running commands outside the sandbox. When `true`, the model can set `dangerouslyDisableSandbox` in tool input, which falls back to the [permissions system](#permissions-fallback-for-unsandboxed-commands)|
|`network`|[`SandboxNetworkConfig`](#sandboxnetworkconfig)|`undefined`|Network-specific sandbox configuration|
|`filesystem`|[`SandboxFilesystemConfig`](#sandboxfilesystemconfig)|`undefined`|Filesystem-specific sandbox configuration for read/write restrictions|
|`ignoreViolations`|`Record\<string, string[]\>`|`undefined`|Map of violation categories to patterns to ignore (e.g., `{ file: ['/tmp/\*'], network: ['localhost'] }`)|
|`enableWeakerNestedSandbox`|`boolean`|`false`|Enable a weaker nested sandbox for compatibility|
|`ripgrep`|`{ command: string; args?: string[] }`|`undefined`|Custom ripgrep binary configuration for sandbox environments|
####
[​
](#example-usage)
Example usage
```
`import { query } from "@anthropic-ai/claude-agent-sdk";
for await (const message of query({
prompt: "Build and test my project",
options: {
sandbox: {
enabled: true,
autoAllowBashIfSandboxed: true,
network: {
allowLocalBinding: true
}
}
}
})) {
if ("result" in message) console.log(message.result);
}
`
```
**Unix socket security:** The `allowUnixSockets` option can grant access to powerful system services. For example, allowing `/var/run/docker.sock` effectively grants full host system access through the Docker API, bypassing sandbox isolation. Only allow Unix sockets that are strictly necessary and understand the security implications of each.
###
[​
](#sandboxnetworkconfig)
`SandboxNetworkConfig`
Network-specific configuration for sandbox mode.
```
`type SandboxNetworkConfig = {
allowedDomains?: string[];
deniedDomains?: string[];
allowManagedDomainsOnly?: boolean;
allowLocalBinding?: boolean;
allowUnixSockets?: string[];
allowAllUnixSockets?: boolean;
httpProxyPort?: number;
socksProxyPort?: number;
};
`
```
|Property|Type|Default|Description|
|`allowedDomains`|`string[]`|`[]`|Domain names that sandboxed processes can access|
|`deniedDomains`|`string[]`|`[]`|Domain names that sandboxed processes cannot access. Takes precedence over `allowedDomains`|
|`allowManagedDomainsOnly`|`boolean`|`false`|Restrict network access to only the domains in `allowedDomains`|
|`allowLocalBinding`|`boolean`|`false`|Allow processes to bind to local ports (e.g., for dev servers)|
|`allowUnixSockets`|`string[]`|`[]`|Unix socket paths that processes can access (e.g., Docker socket)|
|`allowAllUnixSockets`|`boolean`|`false`|Allow access to all Unix sockets|
|`httpProxyPort`|`number`|`undefined`|HTTP proxy port for network requests|
|`socksProxyPort`|`number`|`undefined`|SOCKS proxy port for network requests|
The built-in sandbox proxy enforces `allowedDomains` based on the requested hostname and does not terminate or inspect TLS traffic, so techniques such as [domain fronting](https://en.wikipedia.org/wiki/Domain_fronting) can potentially bypass it. See [Sandboxing security limitations](/docs/en/sandboxing#security-limitations) for details and [Secure deployment](/docs/en/agent-sdk/secure-deployment#traffic-forwarding) for configuring a TLS-terminating proxy.
###
[​
](#sandboxfilesystemconfig)
`SandboxFilesystemConfig`
Filesystem-specific configuration for sandbox mode.
```
`type SandboxFilesystemConfig = {
allowWrite?: string[];
denyWrite?: string[];
denyRead?: string[];
};
`
```
|Property|Type|Default|Description|
|`allowWrite`|`string[]`|`[]`|File path patterns to allow write access to|
|`denyWrite`|`string[]`|`[]`|File path patterns to deny write access to|
|`denyRead`|`string[]`|`[]`|File path patterns to deny read access to|
###
[​
](#permissions-fallback-for-unsandboxed-commands)
Permissions Fallback for Unsandboxed Commands
When `allowUnsandboxedCommands` is enabled, the model can request to run commands outside the sandbox by setting `dangerouslyDisableSandbox: true` in the tool input. These requests fall back to the existing permissions system, meaning your `canUseTool` handler is invoked, allowing you to implement custom authorization logic.
**`excludedCommands` vs `allowUnsandboxedCommands`:**
* `excludedCommands`: A static list of commands that always bypass the sandbox automatically (e.g., `['docker']`). The model has no control over this.
* `allowUnsandboxedCommands`: Lets the model decide at runtime whether to request unsandboxed execution by setting `dangerouslyDisableSandbox: true` in the tool input.
```
`import { query } from "@anthropic-ai/claude-agent-sdk";
for await (const message of query({
prompt: "Deploy my application",
options: {
sandbox: {
enabled: true,
allowUnsandboxedCommands: true // Model can request unsandboxed execution
},
permissionMode: "default",
canUseTool: async (tool, input) =\> {
// Check if the model is requesting to bypass the sandbox
if (tool === "Bash" && input.dangerouslyDisableSandbox) {
// The model is requesting to run this command outside the sandbox
console.log(`Unsandboxed command requested: ${input.command}`);
if (isCommandAuthorized(input.command)) {
return { behavior: "allow" as const, updatedInput: input };
}
return {
behavior: "deny" as const,
message: "Command not authorized for unsandboxed execution"
};
}
return { behavior: "allow" as const, updatedInput: input };
}
}
})) {
if ("result" in message) console.log(message.result);
}
`
```
This pattern enables you to:
* **Audit model requests:** Log when the model requests unsandboxed execution
* **Implement allowlists:** Only permit specific commands to run unsandboxed
* **Add approval workflows:** Require explicit authorization for privileged operations
Commands running with `dangerouslyDisableSandbox: true` have full system access. Ensure your `canUseTool` handler validates these requests carefully.If `permissionMode` is set to `bypassPermissions` and `allowUnsandboxedCommands` is enabled, the model can autonomously execute commands outside the sandbox without any approval prompts. This combination effectively allows the model to escape sandbox isolation silently.
##
[​
](#see-also)
See also
* [SDK overview](/docs/en/agent-sdk/overview) - General SDK concepts
* [Python SDK reference](/docs/en/agent-sdk/python) - Python SDK documentation
* [CLI reference](/docs/en/cli-reference) - Command-line interface
* [Common workflows](/docs/en/common-workflows) - Step-by-step guides
⌘I