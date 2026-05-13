Explore the context window - Claude Code Docs
## > Documentation Index
> Fetch the complete documentation index at:
[> https://code.claude.com/docs/llms.txt
](https://code.claude.com/docs/llms.txt)
> Use this file to discover all available pages before exploring further.
Claude Code’s context window holds everything Claude knows about your session: your instructions, the files it reads, its own responses, and content that never appears in your terminal. The timeline below walks through what loads and when. See [the written breakdown](#what-the-timeline-shows) for the same content as a list.
##
[​
](#what-the-timeline-shows)
What the timeline shows
The session walks through a realistic flow with representative token counts:
* **Before you type anything**: CLAUDE.md, auto memory, MCP tool names, and skill descriptions all load into context. Your own setup may add more here, like an [output style](/docs/en/output-styles) or text from [`--append-system-prompt`](/docs/en/cli-reference), which both go into the system prompt the same way.
* **As Claude works**: each file read adds to context, [path-scoped rules](/docs/en/memory#path-specific-rules) load automatically alongside matching files, and a [PostToolUse hook](/docs/en/hooks-guide) fires after each edit.
* **The follow-up prompt**: a [subagent](/docs/en/sub-agents) handles the research in its own separate context window, so the large file reads stay out of yours. Only the summary and a small metadata trailer come back.
* **At the end**: `/compact` replaces the conversation with a structured summary. Most startup content reloads automatically; the table below shows what happens to each mechanism.
##
[​
](#what-survives-compaction)
What survives compaction
When a long session compacts, Claude Code summarizes the conversation history to fit the context window. What happens to your instructions depends on how they were loaded:
|Mechanism|After compaction|
|System prompt and output style|Unchanged; not part of message history|
|Project-root CLAUDE.md and unscoped rules|Re-injected from disk|
|Auto memory|Re-injected from disk|
|Rules with `paths:` frontmatter|Lost until a matching file is read again|
|Nested CLAUDE.md in subdirectories|Lost until a file in that subdirectory is read again|
|Invoked skill bodies|Re-injected, capped at 5,000 tokens per skill and 25,000 tokens total; oldest dropped first|
|Hooks|Not applicable; hooks run as code, not context|
Path-scoped rules and nested CLAUDE.md files load into message history when their trigger file is read, so compaction summarizes them away with everything else. They reload the next time Claude reads a matching file. If a rule must persist across compaction, drop the `paths:` frontmatter or move it to the project-root CLAUDE.md.
Skill bodies are re-injected after compaction, but large skills are truncated to fit the per-skill cap, and the oldest invoked skills are dropped once the total budget is exceeded. Truncation keeps the start of the file, so put the most important instructions near the top of `SKILL.md`.
##
[​
](#check-your-own-session)
Check your own session
The visualization uses representative numbers. To see your actual context usage at any point, run `/context` for a live breakdown by category with optimization suggestions. Run `/memory` to check which CLAUDE.md and auto memory files loaded at startup.
##
[​
](#related-resources)
Related resources
For deeper coverage of the features shown in the timeline, see these pages:
* [Extend Claude Code](/docs/en/features-overview): when to use CLAUDE.md vs skills vs rules vs hooks vs MCP
* [Store instructions and memories](/docs/en/memory): CLAUDE.md hierarchy and auto memory
* [Subagents](/docs/en/sub-agents): delegate research to a separate context window
* [Best practices](/docs/en/best-practices): managing context as your primary constraint
* [Reduce token usage](/docs/en/costs#reduce-token-usage): strategies for keeping context usage low
⌘I