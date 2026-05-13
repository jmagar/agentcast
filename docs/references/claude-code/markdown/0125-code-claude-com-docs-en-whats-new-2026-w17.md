Week 17 · April 20–24, 2026 - Claude Code Docs
## > Documentation Index
> Fetch the complete documentation index at:
[> https://code.claude.com/docs/llms.txt
](https://code.claude.com/docs/llms.txt)
> Use this file to discover all available pages before exploring further.
Releases [v2.1.114 → v2.1.119](/docs/en/changelog#2-1-114)4 features · April 20–24
/ultrareviewresearch preview
Now in public research preview. Ultrareview runs a fleet of bug-hunting agents in the cloud against your branch or a PR, and findings land back in the CLI or Desktop automatically. Run it before merging critical changes such as auth or data migrations.
Review the branch you’re on:
Claude Code
```
`\> /ultrareview
`
```
Or point it at a PR:
Claude Code
```
`\> /ultrareview 1234
`
```
[Ultrareview guide](/docs/en/ultrareview)
Session recapCLI
Switch focus away from a session and come back to a one-line recap of what happened while you were gone. Helpful for staying in flow while running several Claude sessions at once.
Generate a recap on demand, or turn the automatic one off from `/config`:
Claude Code
```
`\> /recap
`
```
[Interactive mode: session recap](/docs/en/interactive-mode#session-recap)
Custom themesv2.1.118
Build and switch between named color themes from `/theme`, or hand-edit JSON files in `\~/.claude/themes/`. Each theme picks a base preset and overrides only the tokens you care about. Plugins can ship themes too.
Open the theme picker and create a new one:
Claude Code
```
`\> /theme
`
```
[Terminal config: create a custom theme](/docs/en/terminal-config#create-a-custom-theme)
Claude Code on the webweb
A new look for [claude.ai/code](https://claude.ai/code) that matches the redesigned desktop app: sessions sidebar, drag-and-drop layout, and a refreshed routines view. Key parts were rebuilt for quicker responses and a more reliable experience.
[Claude Code on the web](/docs/en/claude-code-on-the-web)
Other wins
[Vim visual mode](/docs/en/interactive-mode#vim-editor-mode): press `v` for character selection or `V` for line selection in the prompt input, with operators and visual feedback
Hooks can now call MCP tools directly via [`type: “mcp\_tool”`](/docs/en/hooks#mcp-tool-hook-fields), so a hook can hit an already-connected server without spawning a process
`/cost` and `/stats` are merged into [`/usage`](/docs/en/commands); the old names still work as typing shortcuts that open the relevant tab
`/config` changes (theme, editor mode, verbose, and similar) now persist to `\~/.claude/settings.json` and follow the same project/local/policy precedence as other [settings](/docs/en/settings)
[Forked subagents](/docs/en/sub-agents#fork-the-current-conversation) can be enabled on external builds with `CLAUDE\_CODE\_FORK\_SUBAGENT=1`: a fork inherits your full conversation context instead of starting fresh
Default [effort level](/docs/en/model-config#adjust-effort-level) for Pro and Max subscribers on Opus 4.6 and Sonnet 4.6 is now `high` (was `medium`)
Native macOS and Linux builds replace the `Glob` and `Grep` tools with embedded `bfs` and `ugrep` available through Bash, for faster searches without a separate tool round-trip
`—from-pr` now accepts GitLab merge request, Bitbucket pull request, and GitHub Enterprise PR URLs in addition to github.com
Auto mode: include `“$defaults”` in [`autoMode.allow`, `soft\_deny`, or `environment`](/docs/en/auto-mode-config) to add custom rules alongside the built-in list instead of replacing it
New [`claude plugin tag`](/docs/en/plugin-dependencies#tag-plugin-releases-for-version-resolution) command creates release git tags for plugins with version validation
Opus 4.7 sessions now compute against the model’s native 1M context window, fixing inflated `/context` percentages and premature autocompaction
`/resume` on large sessions is up to 67% faster and now offers to summarize stale, large sessions before re-reading them
[Full changelog for v2.1.114–v2.1.119 →](/docs/en/changelog#2-1-114)
⌘I