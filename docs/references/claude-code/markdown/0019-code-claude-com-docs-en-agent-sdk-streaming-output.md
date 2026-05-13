Stream responses in real-time - Claude Code Docs
## > Documentation Index
> Fetch the complete documentation index at:
[> https://code.claude.com/docs/llms.txt
](https://code.claude.com/docs/llms.txt)
> Use this file to discover all available pages before exploring further.
By default, the Agent SDK yields complete `AssistantMessage` objects after Claude finishes generating each response. To receive incremental updates as text and tool calls are generated, enable partial message streaming by setting `include\_partial\_messages` (Python) or `includePartialMessages` (TypeScript) to `true` in your options.
This page covers output streaming (receiving tokens in real-time). For input modes (how you send messages), see [Send messages to agents](/docs/en/agent-sdk/streaming-vs-single-mode). You can also [stream responses using the Agent SDK via the CLI](/docs/en/headless).
##
[​
](#enable-streaming-output)
Enable streaming output
To enable streaming, set `include\_partial\_messages` (Python) or `includePartialMessages` (TypeScript) to `true` in your options. This causes the SDK to yield `StreamEvent` messages containing raw API events as they arrive, in addition to the usual `AssistantMessage` and `ResultMessage`.
Your code then needs to:
1. Check each message’s type to distinguish `StreamEvent` from other message types
2. For `StreamEvent`, extract the `event` field and check its `type`
3. Look for `content\_block\_delta` events where `delta.type` is `text\_delta`, which contain the actual text chunks
The example below enables streaming and prints text chunks as they arrive. Notice the nested type checks: first for `StreamEvent`, then for `content\_block\_delta`, then for `text\_delta`:
Python
TypeScript
```
`from claude\_agent\_sdk import query, ClaudeAgentOptions
from claude\_agent\_sdk.types import StreamEvent
import asyncio
async def stream\_response():
options = ClaudeAgentOptions(
include\_partial\_messages=True,
allowed\_tools=["Bash", "Read"],
)
async for message in query(prompt="List the files in my project", options=options):
if isinstance(message, StreamEvent):
event = message.event
if event.get("type") == "content\_block\_delta":
delta = event.get("delta", {})
if delta.get("type") == "text\_delta":
print(delta.get("text", ""), end="", flush=True)
asyncio.run(stream\_response())
`
```
##
[​
](#streamevent-reference)
StreamEvent reference
When partial messages are enabled, you receive raw Claude API streaming events wrapped in an object. The type has different names in each SDK:
* **Python**: `StreamEvent` (import from `claude\_agent\_sdk.types`)
* **TypeScript**: `SDKPartialAssistantMessage` with `type: 'stream\_event'`
Both contain raw Claude API events, not accumulated text. You need to extract and accumulate text deltas yourself. Here’s the structure of each type:
Python
TypeScript
```
`@dataclass
class StreamEvent:
uuid: str # Unique identifier for this event
session\_id: str # Session identifier
event: dict[str, Any] # The raw Claude API stream event
parent\_tool\_use\_id: str | None # Parent tool ID if from a subagent
`
```
The `event` field contains the raw streaming event from the [Claude API](https://platform.claude.com/docs/en/build-with-claude/streaming#event-types). Common event types include:
|Event Type|Description|
|`message\_start`|Start of a new message|
|`content\_block\_start`|Start of a new content block (text or tool use)|
|`content\_block\_delta`|Incremental update to content|
|`content\_block\_stop`|End of a content block|
|`message\_delta`|Message-level updates (stop reason, usage)|
|`message\_stop`|End of the message|
##
[​
](#message-flow)
Message flow
With partial messages enabled, you receive messages in this order:
```
`StreamEvent (message\_start)
StreamEvent (content\_block\_start) - text block
StreamEvent (content\_block\_delta) - text chunks...
StreamEvent (content\_block\_stop)
StreamEvent (content\_block\_start) - tool\_use block
StreamEvent (content\_block\_delta) - tool input chunks...
StreamEvent (content\_block\_stop)
StreamEvent (message\_delta)
StreamEvent (message\_stop)
AssistantMessage - complete message with all content
... tool executes ...
... more streaming events for next turn ...
ResultMessage - final result
`
```
Without partial messages enabled (`include\_partial\_messages` in Python, `includePartialMessages` in TypeScript), you receive all message types except `StreamEvent`. Common types include `SystemMessage` (session initialization), `AssistantMessage` (complete responses), `ResultMessage` (final result), and a compact boundary message indicating when conversation history was compacted (`SDKCompactBoundaryMessage` in TypeScript; `SystemMessage` with subtype `"compact\_boundary"` in Python).
##
[​
](#stream-text-responses)
Stream text responses
To display text as it’s generated, look for `content\_block\_delta` events where `delta.type` is `text\_delta`. These contain the incremental text chunks. The example below prints each chunk as it arrives:
Python
TypeScript
```
`from claude\_agent\_sdk import query, ClaudeAgentOptions
from claude\_agent\_sdk.types import StreamEvent
import asyncio
async def stream\_text():
options = ClaudeAgentOptions(include\_partial\_messages=True)
async for message in query(prompt="Explain how databases work", options=options):
if isinstance(message, StreamEvent):
event = message.event
if event.get("type") == "content\_block\_delta":
delta = event.get("delta", {})
if delta.get("type") == "text\_delta":
# Print each text chunk as it arrives
print(delta.get("text", ""), end="", flush=True)
print() # Final newline
asyncio.run(stream\_text())
`
```
##
[​
](#stream-tool-calls)
Stream tool calls
Tool calls also stream incrementally. You can track when tools start, receive their input as it’s generated, and see when they complete. The example below tracks the current tool being called and accumulates the JSON input as it streams in. It uses three event types:
* `content\_block\_start`: tool begins
* `content\_block\_delta` with `input\_json\_delta`: input chunks arrive
* `content\_block\_stop`: tool call complete
Python
TypeScript
```
`from claude\_agent\_sdk import query, ClaudeAgentOptions
from claude\_agent\_sdk.types import StreamEvent
import asyncio
async def stream\_tool\_calls():
options = ClaudeAgentOptions(
include\_partial\_messages=True,
allowed\_tools=["Read", "Bash"],
)
# Track the current tool and accumulate its input JSON
current\_tool = None
tool\_input = ""
async for message in query(prompt="Read the README.md file", options=options):
if isinstance(message, StreamEvent):
event = message.event
event\_type = event.get("type")
if event\_type == "content\_block\_start":
# New tool call is starting
content\_block = event.get("content\_block", {})
if content\_block.get("type") == "tool\_use":
current\_tool = content\_block.get("name")
tool\_input = ""
print(f"Starting tool: {current\_tool}")
elif event\_type == "content\_block\_delta":
delta = event.get("delta", {})
if delta.get("type") == "input\_json\_delta":
# Accumulate JSON input as it streams in
chunk = delta.get("partial\_json", "")
tool\_input += chunk
print(f" Input chunk: {chunk}")
elif event\_type == "content\_block\_stop":
# Tool call complete - show final input
if current\_tool:
print(f"Tool {current\_tool} called with: {tool\_input}")
current\_tool = None
asyncio.run(stream\_tool\_calls())
`
```
##
[​
](#build-a-streaming-ui)
Build a streaming UI
This example combines text and tool streaming into a cohesive UI. It tracks whether the agent is currently executing a tool (using an `in\_tool` flag) to show status indicators like `[Using Read...]` while tools run. Text streams normally when not in a tool, and tool completion triggers a “done” message. This pattern is useful for chat interfaces that need to show progress during multi-step agent tasks.
Python
TypeScript
```
`from claude\_agent\_sdk import query, ClaudeAgentOptions, ResultMessage
from claude\_agent\_sdk.types import StreamEvent
import asyncio
import sys
async def streaming\_ui():
options = ClaudeAgentOptions(
include\_partial\_messages=True,
allowed\_tools=["Read", "Bash", "Grep"],
)
# Track whether we're currently in a tool call
in\_tool = False
async for message in query(
prompt="Find all TODO comments in the codebase", options=options
):
if isinstance(message, StreamEvent):
event = message.event
event\_type = event.get("type")
if event\_type == "content\_block\_start":
content\_block = event.get("content\_block", {})
if content\_block.get("type") == "tool\_use":
# Tool call is starting - show status indicator
tool\_name = content\_block.get("name")
print(f"\\n[Using {tool\_name}...]", end="", flush=True)
in\_tool = True
elif event\_type == "content\_block\_delta":
delta = event.get("delta", {})
# Only stream text when not executing a tool
if delta.get("type") == "text\_delta" and not in\_tool:
sys.stdout.write(delta.get("text", ""))
sys.stdout.flush()
elif event\_type == "content\_block\_stop":
if in\_tool:
# Tool call finished
print(" done", flush=True)
in\_tool = False
elif isinstance(message, ResultMessage):
# Agent finished all work
print(f"\\n\\n--- Complete ---")
asyncio.run(streaming\_ui())
`
```
##
[​
](#known-limitations)
Known limitations
Some SDK features are incompatible with streaming:
* **Extended thinking**: when you explicitly set `max\_thinking\_tokens` (Python) or `maxThinkingTokens` (TypeScript), `StreamEvent` messages are not emitted. You’ll only receive complete messages after each turn. Note that thinking is disabled by default in the SDK, so streaming works unless you enable it.
* **Structured output**: the JSON result appears only in the final `ResultMessage.structured\_output`, not as streaming deltas. See [structured outputs](/docs/en/agent-sdk/structured-outputs) for details.
##
[​
](#next-steps)
Next steps
Now that you can stream text and tool calls in real-time, explore these related topics:
* [Interactive vs one-shot queries](/docs/en/agent-sdk/streaming-vs-single-mode): choose between input modes for your use case
* [Structured outputs](/docs/en/agent-sdk/structured-outputs): get typed JSON responses from the agent
* [Permissions](/docs/en/agent-sdk/permissions): control which tools the agent can use
⌘I