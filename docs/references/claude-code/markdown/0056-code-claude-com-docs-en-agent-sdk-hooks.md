Intercept and control agent behavior with hooks - Claude Code Docs
## > Documentation Index
> Fetch the complete documentation index at:
[> https://code.claude.com/docs/llms.txt
](https://code.claude.com/docs/llms.txt)
> Use this file to discover all available pages before exploring further.
Hooks are callback functions that run your code in response to agent events, like a tool being called, a session starting, or execution stopping. With hooks, you can:
* **Block dangerous operations** before they execute, like destructive shell commands or unauthorized file access
* **Log and audit** every tool call for compliance, debugging, or analytics
* **Transform inputs and outputs** to sanitize data, inject credentials, or redirect file paths
* **Require human approval** for sensitive actions like database writes or API calls
* **Track session lifecycle** to manage state, clean up resources, or send notifications
This guide covers how hooks work, how to configure them, and provides examples for common patterns like blocking tools, modifying inputs, and forwarding notifications.
##
[​
](#how-hooks-work)
How hooks work
1
[
](#)
An event fires
Something happens during agent execution and the SDK fires an event: a tool is about to be called (`PreToolUse`), a tool returned a result (`PostToolUse`), a subagent started or stopped, the agent is idle, or execution finished. See the [full list of events](#available-hooks).
2
[
](#)
The SDK collects registered hooks
The SDK checks for hooks registered for that event type. This includes callback hooks you pass in `options.hooks` and shell command hooks from settings files when the corresponding [`settingSources`](/docs/en/agent-sdk/typescript#settingsource) or [`setting\_sources`](/docs/en/agent-sdk/python#settingsource) entry is enabled, which it is for default `query()` options.
3
[
](#)
Matchers filter which hooks run
If a hook has a [`matcher`](#matchers) pattern (like `"Write|Edit"`), the SDK tests it against the event’s target (for example, the tool name). Hooks without a matcher run for every event of that type.
4
[
](#)
Callback functions execute
Each matching hook’s [callback function](#callback-functions) receives input about what’s happening: the tool name, its arguments, the session ID, and other event-specific details.
5
[
](#)
Your callback returns a decision
After performing any operations (logging, API calls, validation), your callback returns an [output object](#outputs) that tells the agent what to do: allow the operation, block it, modify the input, or inject context into the conversation.
The following example puts these steps together. It registers a `PreToolUse` hook (step 1) with a `"Write|Edit"` matcher (step 3) so the callback only fires for file-writing tools. When triggered, the callback receives the tool’s input (step 4), checks if the file path targets a `.env` file, and returns `permissionDecision: "deny"` to block the operation (step 5):
Python
TypeScript
```
`import asyncio
from claude\_agent\_sdk import (
AssistantMessage,
ClaudeSDKClient,
ClaudeAgentOptions,
HookMatcher,
ResultMessage,
)
# Define a hook callback that receives tool call details
async def protect\_env\_files(input\_data, tool\_use\_id, context):
# Extract the file path from the tool's input arguments
file\_path = input\_data["tool\_input"].get("file\_path", "")
file\_name = file\_path.split("/")[-1]
# Block the operation if targeting a .env file
if file\_name == ".env":
return {
"hookSpecificOutput": {
"hookEventName": input\_data["hook\_event\_name"],
"permissionDecision": "deny",
"permissionDecisionReason": "Cannot modify .env files",
}
}
# Return empty object to allow the operation
return {}
async def main():
options = ClaudeAgentOptions(
hooks={
# Register the hook for PreToolUse events
# The matcher filters to only Write and Edit tool calls
"PreToolUse": [HookMatcher(matcher="Write|Edit", hooks=[protect\_env\_files])]
}
)
async with ClaudeSDKClient(options=options) as client:
await client.query("Update the database configuration")
async for message in client.receive\_response():
# Filter for assistant and result messages
if isinstance(message, (AssistantMessage, ResultMessage)):
print(message)
asyncio.run(main())
`
```
##
[​
](#available-hooks)
Available hooks
The SDK provides hooks for different stages of agent execution. Some hooks are available in both SDKs, while others are TypeScript-only.
|Hook Event|Python SDK|TypeScript SDK|What triggers it|Example use case|
|`PreToolUse`|Yes|Yes|Tool call request (can block or modify)|Block dangerous shell commands|
|`PostToolUse`|Yes|Yes|Tool execution result|Log all file changes to audit trail|
|`PostToolUseFailure`|Yes|Yes|Tool execution failure|Handle or log tool errors|
|`PostToolBatch`|No|Yes|A full batch of tool calls resolves, once per batch before the next model call|Inject conventions once for the whole batch|
|`UserPromptSubmit`|Yes|Yes|User prompt submission|Inject additional context into prompts|
|`Stop`|Yes|Yes|Agent execution stop|Save session state before exit|
|`SubagentStart`|Yes|Yes|Subagent initialization|Track parallel task spawning|
|`SubagentStop`|Yes|Yes|Subagent completion|Aggregate results from parallel tasks|
|`PreCompact`|Yes|Yes|Conversation compaction request|Archive full transcript before summarizing|
|`PermissionRequest`|Yes|Yes|Permission dialog would be displayed|Custom permission handling|
|`SessionStart`|No|Yes|Session initialization|Initialize logging and telemetry|
|`SessionEnd`|No|Yes|Session termination|Clean up temporary resources|
|`Notification`|Yes|Yes|Agent status messages|Send agent status updates to Slack or PagerDuty|
|`Setup`|No|Yes|Session setup/maintenance|Run initialization tasks|
|`TeammateIdle`|No|Yes|Teammate becomes idle|Reassign work or notify|
|`TaskCompleted`|No|Yes|Background task completes|Aggregate results from parallel tasks|
|`ConfigChange`|No|Yes|Configuration file changes|Reload settings dynamically|
|`WorktreeCreate`|No|Yes|Git worktree created|Track isolated workspaces|
|`WorktreeRemove`|No|Yes|Git worktree removed|Clean up workspace resources|
##
[​
](#configure-hooks)
Configure hooks
To configure a hook, pass it in the `hooks` field of your agent options (`ClaudeAgentOptions` in Python, the `options` object in TypeScript):
Python
TypeScript
```
`options = ClaudeAgentOptions(
hooks={"PreToolUse": [HookMatcher(matcher="Bash", hooks=[my\_callback])]}
)
async with ClaudeSDKClient(options=options) as client:
await client.query("Your prompt")
async for message in client.receive\_response():
print(message)
`
```
The `hooks` option is a dictionary (Python) or object (TypeScript) where:
* **Keys** are [hook event names](#available-hooks) (e.g., `'PreToolUse'`, `'PostToolUse'`, `'Stop'`)
* **Values** are arrays of [matchers](#matchers), each containing an optional filter pattern and your [callback functions](#callback-functions)
###
[​
](#matchers)
Matchers
Use matchers to filter when your callbacks fire. The `matcher` field is a regex string that matches against a different value depending on the hook event type. For example, tool-based hooks match against the tool name, while `Notification` hooks match against the notification type. See the [Claude Code hooks reference](/docs/en/hooks#matcher-patterns) for the full list of matcher values for each event type.
|Option|Type|Default|Description|
|`matcher`|`string`|`undefined`|Regex pattern matched against the event’s filter field. For tool hooks, this is the tool name. Built-in tools include `Bash`, `Read`, `Write`, `Edit`, `Glob`, `Grep`, `WebFetch`, `Agent`, and others (see [Tool Input Types](/docs/en/agent-sdk/typescript#tool-input-types) for the full list). MCP tools use the pattern `mcp\_\_\<server\>\_\_\<action\>`.|
|`hooks`|`HookCallback[]`|-|Required. Array of callback functions to execute when the pattern matches|
|`timeout`|`number`|`60`|Timeout in seconds|
Use the `matcher` pattern to target specific tools whenever possible. A matcher with `'Bash'` only runs for Bash commands, while omitting the pattern runs your callbacks for every occurrence of the event. Note that for tool-based hooks, matchers only filter by **tool name**, not by file paths or other arguments. To filter by file path, check `tool\_input.file\_path` inside your callback.
**Discovering tool names:** See [Tool Input Types](/docs/en/agent-sdk/typescript#tool-input-types) for the full list of built-in tool names, or add a hook without a matcher to log all tool calls your session makes.**MCP tool naming:** MCP tools always start with `mcp\_\_` followed by the server name and action: `mcp\_\_\<server\>\_\_\<action\>`. For example, if you configure a server named `playwright`, its tools will be named `mcp\_\_playwright\_\_browser\_screenshot`, `mcp\_\_playwright\_\_browser\_click`, etc. The server name comes from the key you use in the `mcpServers` configuration.
###
[​
](#callback-functions)
Callback functions
####
[​
](#inputs)
Inputs
Every hook callback receives three arguments:
* **Input data:** a typed object containing event details. Each hook type has its own input shape (for example, `PreToolUseHookInput` includes `tool\_name` and `tool\_input`, while `NotificationHookInput` includes `message`). See the full type definitions in the [TypeScript](/docs/en/agent-sdk/typescript#hookinput) and [Python](/docs/en/agent-sdk/python#hookinput) SDK references.
* All hook inputs share `session\_id`, `cwd`, and `hook\_event\_name`.
* `agent\_id` and `agent\_type` are populated when the hook fires inside a subagent. In TypeScript, these are on the base hook input and available to all hook types. In Python, they are on `PreToolUse`, `PostToolUse`, and `PostToolUseFailure` only.
* **Tool use ID** (`str | None` / `string | undefined`): correlates `PreToolUse` and `PostToolUse` events for the same tool call.
* **Context:** in TypeScript, contains a `signal` property (`AbortSignal`) for cancellation. In Python, this argument is reserved for future use.
####
[​
](#outputs)
Outputs
Your callback returns an object with two categories of fields:
* **Top-level fields** control the conversation: `systemMessage` injects a message into the conversation visible to the model, and `continue` (`continue\_` in Python) determines whether the agent keeps running after this hook.
* **`hookSpecificOutput`** controls the current operation. The fields inside depend on the hook event type. For `PreToolUse` hooks, this is where you set `permissionDecision` (`"allow"`, `"deny"`, `"ask"`, or `"defer"`), `permissionDecisionReason`, and `updatedInput`. Returning `"defer"` ends the query so you can [resume it later](/docs/en/hooks#defer-a-tool-call-for-later). For `PostToolUse` hooks, you can set `additionalContext` to append information to the tool result, or `updatedToolOutput` to replace the tool’s output entirely before Claude sees it.
Return `{}` to allow the operation without changes. SDK callback hooks use the same JSON output format as [Claude Code shell command hooks](/docs/en/hooks#json-output), which documents every field and event-specific option. For the SDK type definitions, see the [TypeScript](/docs/en/agent-sdk/typescript#synchookjsonoutput) and [Python](/docs/en/agent-sdk/python#synchookjsonoutput) SDK references.
When multiple hooks or permission rules apply, **deny** takes priority over **defer**, which takes priority over **ask**, which takes priority over **allow**. If any hook returns `deny`, the operation is blocked regardless of other hooks.
####
[​
](#asynchronous-output)
Asynchronous output
By default, the agent waits for your hook to return before proceeding. If your hook performs a side effect (logging, sending a webhook) and doesn’t need to influence the agent’s behavior, you can return an async output instead. This tells the agent to continue immediately without waiting for the hook to finish:
Python
TypeScript
```
`async def async\_hook(input\_data, tool\_use\_id, context):
# Start a background task, then return immediately
asyncio.create\_task(send\_to\_logging\_service(input\_data))
return {"async\_": True, "asyncTimeout": 30000}
`
```
|Field|Type|Description|
|`async`|`true`|Signals async mode. The agent proceeds without waiting. In Python, use `async\_` to avoid the reserved keyword.|
|`asyncTimeout`|`number`|Optional timeout in milliseconds for the background operation|
Async outputs cannot block, modify, or inject context into the operation since the agent has already moved on. Use them only for side effects like logging, metrics, or notifications.
##
[​
](#examples)
Examples
###
[​
](#modify-tool-input)
Modify tool input
This example intercepts Write tool calls and rewrites the `file\_path` argument to prepend `/sandbox`, redirecting all file writes to a sandboxed directory. The callback returns `updatedInput` with the modified path and `permissionDecision: 'allow'` to auto-approve the rewritten operation:
Python
TypeScript
```
`async def redirect\_to\_sandbox(input\_data, tool\_use\_id, context):
if input\_data["hook\_event\_name"] != "PreToolUse":
return {}
if input\_data["tool\_name"] == "Write":
original\_path = input\_data["tool\_input"].get("file\_path", "")
return {
"hookSpecificOutput": {
"hookEventName": input\_data["hook\_event\_name"],
"permissionDecision": "allow",
"updatedInput": {
\*\*input\_data["tool\_input"],
"file\_path": f"/sandbox{original\_path}",
},
}
}
return {}
`
```
When using `updatedInput`, you must also include `permissionDecision: 'allow'` to auto-approve the modified input or `permissionDecision: 'ask'` to show it to the user. With `'defer'`, `updatedInput` is ignored. Always return a new object rather than mutating the original `tool\_input`.
###
[​
](#add-context-and-block-a-tool)
Add context and block a tool
This example blocks any attempt to write to the `/etc` directory and uses two output fields together: `permissionDecision: 'deny'` stops the tool call, while `systemMessage` injects a reminder into the conversation so the agent receives context about why the operation was blocked and avoids retrying it:
Python
TypeScript
```
`async def block\_etc\_writes(input\_data, tool\_use\_id, context):
file\_path = input\_data["tool\_input"].get("file\_path", "")
if file\_path.startswith("/etc"):
return {
# Top-level field: inject guidance into the conversation
"systemMessage": "Remember: system directories like /etc are protected.",
# hookSpecificOutput: block the operation
"hookSpecificOutput": {
"hookEventName": input\_data["hook\_event\_name"],
"permissionDecision": "deny",
"permissionDecisionReason": "Writing to /etc is not allowed",
},
}
return {}
`
```
###
[​
](#auto-approve-specific-tools)
Auto-approve specific tools
By default, the agent may prompt for permission before using certain tools. This example auto-approves read-only filesystem tools (Read, Glob, Grep) by returning `permissionDecision: 'allow'`, letting them run without user confirmation while leaving all other tools subject to normal permission checks:
Python
TypeScript
```
`async def auto\_approve\_read\_only(input\_data, tool\_use\_id, context):
if input\_data["hook\_event\_name"] != "PreToolUse":
return {}
read\_only\_tools = ["Read", "Glob", "Grep"]
if input\_data["tool\_name"] in read\_only\_tools:
return {
"hookSpecificOutput": {
"hookEventName": input\_data["hook\_event\_name"],
"permissionDecision": "allow",
"permissionDecisionReason": "Read-only tool auto-approved",
}
}
return {}
`
```
###
[​
](#register-multiple-hooks)
Register multiple hooks
When an event fires, all matching hooks run in parallel. For permission decisions, the most restrictive result wins: a single `deny` blocks the tool call regardless of what the other hooks return. Because completion order is non-deterministic, write each hook to act independently rather than relying on another hook having run first.
The example below registers three independent checks for every tool call:
Python
TypeScript
```
`options = ClaudeAgentOptions(
hooks={
"PreToolUse": [
HookMatcher(hooks=[authorization\_check]),
HookMatcher(hooks=[input\_validator]),
HookMatcher(hooks=[audit\_logger]),
]
}
)
`
```
###
[​
](#filter-with-regex-matchers)
Filter with regex matchers
Use regex patterns to match multiple tools. This example registers three matchers with different scopes: the first triggers `file\_security\_hook` only for file modification tools, the second triggers `mcp\_audit\_hook` for any MCP tool (tools whose names start with `mcp\_\_`), and the third triggers `global\_logger` for every tool call regardless of name:
Python
TypeScript
```
`options = ClaudeAgentOptions(
hooks={
"PreToolUse": [
# Match file modification tools
HookMatcher(matcher="Write|Edit|Delete", hooks=[file\_security\_hook]),
# Match all MCP tools
HookMatcher(matcher="^mcp\_\_", hooks=[mcp\_audit\_hook]),
# Match everything (no matcher)
HookMatcher(hooks=[global\_logger]),
]
}
)
`
```
###
[​
](#track-subagent-activity)
Track subagent activity
Use `SubagentStop` hooks to monitor when subagents finish their work. See the full input type in the [TypeScript](/docs/en/agent-sdk/typescript#hookinput) and [Python](/docs/en/agent-sdk/python#hookinput) SDK references. This example logs a summary each time a subagent completes:
Python
TypeScript
```
`async def subagent\_tracker(input\_data, tool\_use\_id, context):
# Log subagent details when it finishes
print(f"[SUBAGENT] Completed: {input\_data['agent\_id']}")
print(f" Transcript: {input\_data['agent\_transcript\_path']}")
print(f" Tool use ID: {tool\_use\_id}")
print(f" Stop hook active: {input\_data.get('stop\_hook\_active')}")
return {}
options = ClaudeAgentOptions(
hooks={"SubagentStop": [HookMatcher(hooks=[subagent\_tracker])]}
)
`
```
###
[​
](#make-http-requests-from-hooks)
Make HTTP requests from hooks
Hooks can perform asynchronous operations like HTTP requests. Catch errors inside your hook instead of letting them propagate, since an unhandled exception can interrupt the agent.
This example sends a webhook after each tool completes, logging which tool ran and when. The hook catches errors so a failed webhook doesn’t interrupt the agent:
Python
TypeScript
```
`import asyncio
import json
import urllib.request
from datetime import datetime
def \_send\_webhook(tool\_name):
"""Synchronous helper that POSTs tool usage data to an external webhook."""
data = json.dumps(
{
"tool": tool\_name,
"timestamp": datetime.now().isoformat(),
}
).encode()
req = urllib.request.Request(
"https://api.example.com/webhook",
data=data,
headers={"Content-Type": "application/json"},
method="POST",
)
urllib.request.urlopen(req)
async def webhook\_notifier(input\_data, tool\_use\_id, context):
# Only fire after a tool completes (PostToolUse), not before
if input\_data["hook\_event\_name"] != "PostToolUse":
return {}
try:
# Run the blocking HTTP call in a thread to avoid blocking the event loop
await asyncio.to\_thread(\_send\_webhook, input\_data["tool\_name"])
except Exception as e:
# Log the error but don't raise. A failed webhook shouldn't stop the agent
print(f"Webhook request failed: {e}")
return {}
`
```
###
[​
](#forward-notifications-to-slack)
Forward notifications to Slack
Use `Notification` hooks to receive system notifications from the agent and forward them to external services. Notifications fire for specific event types: `permission\_prompt` (Claude needs permission), `idle\_prompt` (Claude is waiting for input), `auth\_success` (authentication completed), `elicitation\_dialog` (Claude is prompting the user), `elicitation\_response` (the user answered an elicitation), and `elicitation\_complete` (an elicitation closed). Each notification includes a `message` field with a human-readable description and optionally a `title`.
This example forwards every notification to a Slack channel. It requires a [Slack incoming webhook URL](https://api.slack.com/messaging/webhooks), which you create by adding an app to your Slack workspace and enabling incoming webhooks:
Python
TypeScript
```
`import asyncio
import json
import urllib.request
from claude\_agent\_sdk import ClaudeSDKClient, ClaudeAgentOptions, HookMatcher
def \_send\_slack\_notification(message):
"""Synchronous helper that sends a message to Slack via incoming webhook."""
data = json.dumps({"text": f"Agent status: {message}"}).encode()
req = urllib.request.Request(
"https://hooks.slack.com/services/YOUR/WEBHOOK/URL",
data=data,
headers={"Content-Type": "application/json"},
method="POST",
)
urllib.request.urlopen(req)
async def notification\_handler(input\_data, tool\_use\_id, context):
try:
# Run the blocking HTTP call in a thread to avoid blocking the event loop
await asyncio.to\_thread(\_send\_slack\_notification, input\_data.get("message", ""))
except Exception as e:
print(f"Failed to send notification: {e}")
# Return empty object. Notification hooks don't modify agent behavior
return {}
async def main():
options = ClaudeAgentOptions(
hooks={
# Register the hook for Notification events (no matcher needed)
"Notification": [HookMatcher(hooks=[notification\_handler])],
},
)
async with ClaudeSDKClient(options=options) as client:
await client.query("Analyze this codebase")
async for message in client.receive\_response():
print(message)
asyncio.run(main())
`
```
##
[​
](#fix-common-issues)
Fix common issues
###
[​
](#hook-not-firing)
Hook not firing
* Verify the hook event name is correct and case-sensitive (`PreToolUse`, not `preToolUse`)
* Check that your matcher pattern matches the tool name exactly
* Ensure the hook is under the correct event type in `options.hooks`
* For non-tool hooks like `Stop` and `SubagentStop`, matchers match against different fields (see [matcher patterns](/docs/en/hooks#matcher-patterns))
* Hooks may not fire when the agent hits the [`max\_turns`](/docs/en/agent-sdk/python#claudeagentoptions) limit because the session ends before hooks can execute
###
[​
](#matcher-not-filtering-as-expected)
Matcher not filtering as expected
Matchers only match **tool names**, not file paths or other arguments. To filter by file path, check `tool\_input.file\_path` inside your hook:
```
`const myHook: HookCallback = async (input, toolUseID, { signal }) =\> {
const preInput = input as PreToolUseHookInput;
const toolInput = preInput.tool\_input as Record\<string, unknown\>;
const filePath = toolInput?.file\_path as string;
if (!filePath?.endsWith(".md")) return {}; // Skip non-markdown files
// Process markdown files...
return {};
};
`
```
###
[​
](#hook-timeout)
Hook timeout
* Increase the `timeout` value in the `HookMatcher` configuration
* Use the `AbortSignal` from the third callback argument to handle cancellation gracefully in TypeScript
###
[​
](#tool-blocked-unexpectedly)
Tool blocked unexpectedly
* Check all `PreToolUse` hooks for `permissionDecision: 'deny'` returns
* Add logging to your hooks to see what `permissionDecisionReason` they’re returning
* Verify matcher patterns aren’t too broad (an empty matcher matches all tools)
###
[​
](#modified-input-not-applied)
Modified input not applied
* Ensure `updatedInput` is inside `hookSpecificOutput`, not at the top level:
```
`return {
hookSpecificOutput: {
hookEventName: "PreToolUse",
permissionDecision: "allow",
updatedInput: { command: "new command" }
}
};
`
```
* You must also return `permissionDecision: 'allow'` or `'ask'` for the input modification to take effect
* Include `hookEventName` in `hookSpecificOutput` to identify which hook type the output is for
###
[​
](#session-hooks-not-available-in-python)
Session hooks not available in Python
`SessionStart` and `SessionEnd` can be registered as SDK callback hooks in TypeScript, but are not available in the Python SDK (`HookEvent` omits them). In Python, they are only available as [shell command hooks](/docs/en/hooks#hook-events) defined in settings files (for example, `.claude/settings.json`). To load shell command hooks from your SDK application, include the appropriate setting source with [`setting\_sources`](/docs/en/agent-sdk/python#settingsource) or [`settingSources`](/docs/en/agent-sdk/typescript#settingsource):
Python
TypeScript
```
`options = ClaudeAgentOptions(
setting\_sources=["project"], # Loads .claude/settings.json including hooks
)
`
```
To run initialization logic as a Python SDK callback instead, use the first message from `client.receive\_response()` as your trigger.
###
[​
](#subagent-permission-prompts-multiplying)
Subagent permission prompts multiplying
When spawning multiple subagents, each one may request permissions separately. Subagents do not automatically inherit parent agent permissions. To avoid repeated prompts, use `PreToolUse` hooks to auto-approve specific tools, or configure permission rules that apply to subagent sessions.
###
[​
](#recursive-hook-loops-with-subagents)
Recursive hook loops with subagents
A `UserPromptSubmit` hook that spawns subagents can create infinite loops if those subagents trigger the same hook. To prevent this:
* Check for a subagent indicator in the hook input before spawning
* Use a shared variable or session state to track whether you’re already inside a subagent
* Scope hooks to only run for the top-level agent session
###
[​
](#systemmessage-not-appearing-in-output)
systemMessage not appearing in output
The `systemMessage` field adds context to the conversation that the model sees, but it may not appear in all SDK output modes. If you need to surface hook decisions to your application, log them separately or use a dedicated output channel.
##
[​
](#related-resources)
Related resources
* [Claude Code hooks reference](/docs/en/hooks): full JSON input/output schemas, event documentation, and matcher patterns
* [Claude Code hooks guide](/docs/en/hooks-guide): shell command hook examples and walkthroughs
* [TypeScript SDK reference](/docs/en/agent-sdk/typescript): hook types, input/output definitions, and configuration options
* [Python SDK reference](/docs/en/agent-sdk/python): hook types, input/output definitions, and configuration options
* [Permissions](/docs/en/agent-sdk/permissions): control what your agent can do
* [Custom tools](/docs/en/agent-sdk/custom-tools): build tools to extend agent capabilities
⌘I