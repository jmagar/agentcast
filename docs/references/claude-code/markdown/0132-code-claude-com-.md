Claude Code by Anthropic | AI Coding Agent, Terminal, IDE
Explore here
Alert
[Next](#)Next
Claude Code on the web is not available for the selected organization. Switch to a different organization or use the CLI.
Latest news
[Next](#)Next
Routines
Configure a routine once and it can run on a schedule, from an API call, or in response to an event.
[Learn more](https://claude.com/blog/introducing-routines-in-claude-code)Learn more
◓
Debugging...
# Claude Code: AI-powered coding assistant for developers
Built for
developer
Work with Claude directly in your codebase. Build, debug, and ship from your terminal, IDE, Slack, or the web. Describe what you need, and Claude handles the rest.
Get Claude Code
curl -fsSL https://claude.ai/install.sh | bash
Copy command to clipboard
irm https://claude.ai/install.ps1 | iex
Copy command to clipboard
Or readthe [documentation](https://code.claude.com/docs/en/overview)
[Try Claude Code](https://claude.ai/code)Try Claude Code
[Developer docs](https://code.claude.com/docs/en/overview)Developer docs
## Use Claude Code where you work
Desktop
Terminal
IDE
Web and iOS
Slack
Prompt
My colleagues recently published the attached single-cell dataset that describes gene expression differences between adult and pediatric liver samples with a focus on the immune system. I would like to explore these samples but focus on the parenchymal cells and differences between adult and pediatric liver. Can you help me first go through the differentially expressed genes and create a heatmap and then also identify pathways or sets of genes that are enriched in each sample?
### Claude Code on desktop
Run Claude Code in the desktop app. Manage multiple parallel tasks, review visual diffs, preview servers, and monitor PR status, all from one place.
[Download Claude for desktop](/download)Download Claude for desktop
##
Individual
Team & Enterprise
### Pro
Claude Code is included in your Pro plan. Perfect for short coding sprints in small codebases with access to both Sonnet 4.6 and Opus 4.7.
$17
Per month with annual subscription discount ($200 billed up front). $20 if billed monthly.
[Try Claude](https://claude.ai/login?plan=pro)Try Claude
### Max 5x
Claude Code is included in your Max plan. Great value for everyday use in larger codebases. 
$100
Per month
[Try Claude](https://claude.ai/login?plan=max)Try Claude
### Max 20x
Even more Claude Code included in your Max plan. Great value for power users with the most access to Claude models.
$200
Per month
[Try Claude](https://claude.ai/login?plan=max)Try Claude
[Usage limits](https://support.anthropic.com/en/articles/9797557-usage-limit-best-practices) apply. Prices shown don’t include applicable tax. Price and plans are subject to change at Anthropic's discretion.
##
Route tasks to the desktop app from the Claude mobile app. Claude runs on your local machine and can open your apps, click through your UI, and test what it built. Now in research preview on Pro and Max plans.
[Pair with your phone](/download)Pair with your phone
### Message Claude from your phone
Describe the bug, the feature, or the refactor. Claude picks up your codebase context where you left off.
### Claude works on your local machine
It reads code in your local environment, writes changes, runs tests, and opens a PR. No setup or context files to select.
### You get a working pull request
Not just a status update: a clean diff, passing tests, a ready-to-merge branch. You review, run CI, and decide what ships next.
##
Get Claude Code
curl -fsSL https://claude.ai/install.sh | bash
Copy command to clipboard
irm https://claude.ai/install.ps1 | iex
Copy command to clipboard
Or readthe [documentation](https://code.claude.com/docs/en/overview)
[Try Claude Code](https://claude.ai/code)Try Claude Code
[Developer docs](https://code.claude.com/docs/en/overview)Developer docs
Onboarding
Triage issues
Refactor code
.
This is some text inside of a div block.
\>
I'm new to this codebase. Can you explain it to me?
⏺ I'll analyze the Excalidraw codebase structure and provide a high-level overview.
‍
⏺ Analyze(what: "Excalidraw codebase structure and provide a high-level overview of what it is, its main components, and how they're organized. Focus on understanding the project's purpose, architecture, and key features.")
‍
Excalidraw is an open-source virtual whiteboard application with a hand-drawn style. It's designed for creating diagrams, wireframes, and collaborative drawings.
‍
## Purpose
- Web-based drawing tool with infinite canvas
- Hand-drawn aesthetic with support for shapes, arrows, text
- End-to-end encrypted collaboration
- Works offline as a PWA (Progressive Web App)
‍
## Architecture
The project is organized as a monorepo with these main components:
‍
1. Core Package (packages/excalidraw)
- React component library for embedding Excalidraw in any application
- Handles drawing, rendering, and core functionality
- Uses roughjs for the hand-drawn style
- Includes export capabilities (PNG, SVG)
‍
2. Web Application (excalidraw-app/)
- Full-featured web app at excalidraw.com
- Built on top of the core package
- Adds collaboration features, local storage, sharing
‍
3. Support Packages
- common - Shared utilities and constants
- element - Element manipulation and type definitions
- math - Mathematical functions for drawing
- utils - General utility functions
‍
4. Examples
- Integration examples (Next.js, browser script)
- Shows how to embed Excalidraw in different environments
‍
5. Documentation (dev-docs/)
- API documentation
- Integration guides
- Component documentation
‍
## Key Features
- Real-time collaboration with end-to-end encryption
- Export to various formats (PNG, SVG, JSON)
- Customizable drawing tools (rectangle, circle, arrows, freehand)
- Dark mode support
- Localization
- Library of shapes and components
- Arrow bindings and labels
- Image support
‍
## Technology Stack
- React for UI
- TypeScript for type safety
- Firebase for collaboration backend
- Jotai for state management
- Vite for development and building
- IndexedDB/LocalStorage for offline storage
‍
The architecture allows for both using Excalidraw as a component in other applications and as a standalone web application with additional features.
### Code onboarding
Claude Code maps and explains entire codebases in a few seconds. It uses agentic search to understand project structure and dependencies without you having to manually select context files.
### Turn issues into PRs
Stop bouncing between tools. Claude Code integrates with GitHub, GitLab, and your command line tools to handle the entire workflow—reading issues, writing code, running tests, and submitting PRs—all from your terminal.
### Make powerful edits
Claude Code’s understanding of your codebase and dependencies enables it to make powerful, multi-file edits that work.
### Powerful intelligence
* Uses agentic search to understand your entire codebase without manual context selection
* Makes coordinated changes across multiple files
* Optimized specifically for code understanding and generation with Claude models
### Works where you work
* Lives right inside your terminal—no  context switching
* Integrates with [VS Code](https://marketplace.visualstudio.com/items?itemName=anthropic.claude-code) / [JetBrains](https://plugins.jetbrains.com/plugin/27310-claude-code-beta-) IDEs
* Uses your test suites and build systems
### You’re in control
* Never modifies your files without explicit approval
* Adapts to your coding standards and patterns
* Configurable: Build on the SDK or run on GitHub Actions
Results with Claude Code
7.6x
more frequent deployments by Claude Code teams
14%
week-over-week deployment growth
Introducing Claude Code
[Next](#)Next
See Claude Code in action—from concept to commit in one seamless workflow.
[Prev](#)Prev
[Next](#)Next
##
### Start in your terminal
Super powerful terminal integration. Works with all your CLI tools alongside any IDE.
Get Claude Code
curl -fsSL https://claude.ai/install.sh | bash
Copy command to clipboard
irm https://claude.ai/install.ps1 | iex
Copy command to clipboard
Or readthe [documentation](https://code.claude.com/docs/en/overview)
[Try Claude Code](https://claude.ai/code)Try Claude Code
[Developer docs](https://code.claude.com/docs/en/overview)Developer docs
### Integrate with your editor
Native extensions for VS Code (+ Cursor, Windsurf) and JetBrains IDEs.
[VS Code](https://marketplace.visualstudio.com/items?itemName=anthropic.claude-code)VS Code
[JetBrains](https://plugins.jetbrains.com/plugin/27310-claude-code-beta-)JetBrains
### Access anywhere
Quick access from browser, mobile app, or Claude on desktop. Great for parallel work or on-the-go coding.
[Open in browser](https://claude.ai/code/family)Open in browser
[Download app](/download)Download app
Kick off coding tasks in Slack
[Add to Slack](https://slack.com/oauth/v2/authorize?client_id=1601185624273.8899143856786&amp;scope=app_mentions:read,assistant:write,channels:history,channels:read,chat:write,files:read,files:write,groups:history,groups:read,im:history,im:read,im:write,mpim:history,reactions:write,users:read,users:read.email,commands,search:read.public&amp;user_scope=bookmarks:read,channels:history,channels:read,chat:write,emoji:read,files:read,groups:history,groups:read,groups:write,im:history,im:read,im:write,links:read,mpim:history,mpim:read,mpim:write,mpim:write.topic,pins:read,reactions:read,reactions:write,remote_files:read,team:read,users:read,users:read.email,search:read.public,search:read.private,search:read.im,search:read.mpim,search:read.files,search:read.users,canvases:read,canvases:write)Add to Slack
[Learn more](/claude-for-slack)Learn more
## Latest feature announcements
[**Redesigned Desktop app: ** Built to help you run more Claude Code tasks at once.](https://claude.com/blog/claude-code-desktop-redesign)**Redesigned Desktop app: ** Built to help you run more Claude Code tasks at once.
**Redesigned Desktop app: ** Built to help you run more Claude Code tasks at once.
Blog
[Blog](https://claude.com/blog/claude-code-desktop-redesign)Blog
Apr 14, 2026
[**Routines: **Configure a routine once, and it can run on a schedule, from an API call, or in response to an event](https://claude.com/blog/introducing-routines-in-claude-code)**Routines: **Configure a routine once, and it can run on a schedule, from an API call, or in response to an event
**Routines: **Configure a routine once, and it can run on a schedule, from an API call, or in response to an event
Blog
[Blog](https://claude.com/blog/introducing-routines-in-claude-code)Blog
Apr 14, 2026
[**Auto mode:** A safer long-running alternative to --dangerously-skip-permissions](https://claude.com/blog/auto-mode)**Auto mode:** A safer long-running alternative to --dangerously-skip-permissions
**Auto mode:** A safer long-running alternative to --dangerously-skip-permissions
Blog
[Blog](https://claude.com/blog/auto-mode)Blog
Mar 24, 2026
[View changelog](https://code.claude.com/docs/en/changelog)View changelog
## What developers are saying
“Claude Code has dramatically accelerated our team’s coding efficiency. I can now write EDA code in a notebook—pulling data, training a model, and evaluating it with basic metrics—and then ask Claude to convert that into a Metaflow pipeline. This process saves 1-2 days of routine (and often boring!) work per model.”
Anton Biryukov, Staff Software Engineer
[Read story](https://www.claude.com/customers/ramp)Read story
“With Claude, we’re not just automating customer service—we’re elevating it to truly human quality. This lets support teams think more strategically about customer experience and what makes interactions genuinely valuable.”
Fergal Reid, VP of AI
[Read story](https://www.claude.com/customers/intercom)Read story
“Claude Code is moving our team up a level: we decide what needs to happen, and smooth the process so it can build and verify end-to-end. A big part of my job now is to keep as many instances of Claude Code busy as possible.”
Simon Last, Co-founder
[Read story](https://www.claude.com/customers/notion)Read story
## Connects with your favorite command line tools
Your terminal is where real work happens. Claude Code connects with the tools that power development—deployment, databases, monitoring, version control. Rather than adding another interface to juggle, it enhances your existing stack.
##
### How do I get started with Claude?
You can access Claude Code with a Claude Pro or Max plan, a Team or Enterprise plan premium seat, or a Claude Console account. [Download Claude Code](https://code.claude.com/docs/en/overview) and sign in with your respective Claude or Console credentials.
### What kinds of tasks can Claude Code handle?
Claude Code excels at both routine development tasks like bug fixes and testing, as well as transformative work like refactors and feature implementation that require deep codebase understanding.
### How does Claude Code work with my existing tools?
Claude Code runs in your terminal and works alongside your preferred IDE and development tools without requiring you to change your workflow. Claude Code can also use command line tools (like Git) and MCP servers (like GitHub) to extend its own capabilities using your tools.
### Is Claude Code secure?
Yes. Claude Code runs locally in your terminal and talks directly to model APIs without requiring a backend server or remote code index. It also asks for permission before making changes to your files or running commands.
### Which models does Claude Code use?
Claude Code works with the Opus 4.7, Sonnet 4.6, and Haiku 4.5 models. Enterprise users can run Claude Code using models in existing Amazon Bedrock or Google Cloud Vertex AI instances.
### What are the system requirements to run Claude Code?
Claude Code works on macOS, Linux, and Windows. [See full system requirements](https://docs.claude.com/en/docs/claude-code/setup#system-requirements).
### How much does Claude Code cost?
When used with a Claude Console account, Claude Code consumes API tokens at [standard API pricing](https://anthropic.com/pricing#api).
### Does Claude Code work with the Claude desktop app?
Yes. Max, Pro, Team, and Enterprise users can access Claude Code on the [Claude desktop app](https://www.claude.com/download).
### What is fast mode on Claude Code?
Fast mode is a high-speed configuration for Opus 4.6, making the model 2.5x faster at a higher cost per token. Fast mode is available:
* In research preview on Claude Code, and is priced at $30/$150 per million tokens.
* On consumption-based plans.
* Via extra usage for users on subscription plans.
[Prev](#)Prev
[Next](#)Next
80%
faster incident investigation time
2x
faster execution speed for delivering features and fixes
89%
AI adoption across all employees
Improving frontend design through Skills
Best practices for building richer, more customized frontend design with Claude and Skills.
[Read article](https://claude.com/blog/building-skills-for-claude-code)Read article
[Prev](#)Prev
[Next](#)Next
##
[Claude Code documentation](https://code.claude.com/docs/en/overview)Claude Code documentation
Claude Code documentation
Developer docs
[Developer docs](https://code.claude.com/docs/en/overview)Developer docs
[Common workflows](https://code.claude.com/docs/en/common-workflows)Common workflows
Common workflows
Developer docs
[Developer docs](https://code.claude.com/docs/en/common-workflows)Developer docs
[Using CLAUDE.md files: Customizing Claude Code for your codebase](https://claude.com/blog/using-claude-md-files)Using CLAUDE.md files: Customizing Claude Code for your codebase
Using CLAUDE.md files: Customizing Claude Code for your codebase
Blog
[Blog](https://claude.com/blog/using-claude-md-files)Blog
[Introduction to agentic coding
](https://claude.com/blog/introduction-to-agentic-coding)Introduction to agentic coding
Introduction to agentic coding
Blog
[Blog](https://claude.com/blog/introduction-to-agentic-coding)Blog
[How Anthropic teams use Claude Code](https://www.anthropic.com/news/how-anthropic-teams-use-claude-code)How Anthropic teams use Claude Code
How Anthropic teams use Claude Code
Case study
[Case study](https://www.anthropic.com/news/how-anthropic-teams-use-claude-code)Case study
[Fix software bugs faster with Claude](https://claude.com/blog/fix-software-bugs-faster-with-claude)Fix software bugs faster with Claude
Fix software bugs faster with Claude
Blog
[Blog](https://claude.com/blog/fix-software-bugs-faster-with-claude)Blog
## Create what’s exciting. Maintain what’s essential.
Use Claude Code where you work
Get Claude Code
curl -fsSL https://claude.ai/install.sh | bash
Copy command to clipboard
irm https://claude.ai/install.ps1 | iex
Copy command to clipboard
Or readthe [documentation](https://code.claude.com/docs/en/overview)
[Try Claude Code](https://claude.ai/code)Try Claude Code
[Developer docs](https://code.claude.com/docs/en/overview)Developer docs
### Get the developer newsletter
Product updates, how-tos, community spotlights, and more. Delivered monthly to your inbox.
[Subscribe](#)Subscribe
Please provide your email address if you'd like to receive our monthly developer newsletter. You can unsubscribe at any time.
Thank you! You’re subscribed.
Sorry, there was a problem with your submission, please try again later.
[Homepage](https://claude.com)Homepage
[Next](#)Next
Thank you! Your submission has been received!
Oops! Something went wrong while submitting the form.
[Button Text](#)Button Text
[Button Text](#)Button Text
[Button Text](#)Button Text
Write
*
Help me develop a unique voice for an audience
[](#)
Hi Claude! Could you help me develop a unique voice for an audience? If you need more information from me, ask me 1-2 key questions right away. If you think I should upload any documents that would help you do a better job, let me know. You can use the tools you have access to— like Google Drive, web search, etc.—if they’ll help you better accomplish this task. Do not use analysis tool. Please keep your responses friendly, brief and conversational.
Please execute the task as soon as you can—an artifact would be great if it makes sense. If using an artifact, consider what kind of artifact (interactive, visual, checklist, etc.) might be most helpful for this specific task. Thanks for your help!
*
Improve my writing style
[](#)
Hi Claude! Could you improve my writing style? If you need more information from me, ask me 1-2 key questions right away. If you think I should upload any documents that would help you do a better job, let me know. You can use the tools you have access to— like Google Drive, web search, etc.—if they’ll help you better accomplish this task. Do not use analysis tool. Please keep your responses friendly, brief and conversational.
Please execute the task as soon as you can—an artifact would be great if it makes sense. If using an artifact, consider what kind of artifact (interactive, visual, checklist, etc.) might be most helpful for this specific task. Thanks for your help!
*
Brainstorm creative ideas
[](#)
Hi Claude! Could you brainstorm creative ideas? If you need more information from me, ask me 1-2 key questions right away. If you think I should upload any documents that would help you do a better job, let me know. You can use the tools you have access to— like Google Drive, web search, etc.—if they’ll help you better accomplish this task. Do not use analysis tool. Please keep your responses friendly, brief and conversational.
Please execute the task as soon as you can—an artifact would be great if it makes sense. If using an artifact, consider what kind of artifact (interactive, visual, checklist, etc.) might be most helpful for this specific task. Thanks for your help!
Learn
*
Explain a complex topic simply
[](#)
Hi Claude! Could you explain a complex topic simply? If you need more information from me, ask me 1-2 key questions right away. If you think I should upload any documents that would help you do a better job, let me know. You can use the tools you have access to— like Google Drive, web search, etc.—if they’ll help you better accomplish this task. Do not use analysis tool. Please keep your responses friendly, brief and conversational.
Please execute the task as soon as you can—an artifact would be great if it makes sense. If using an artifact, consider what kind of artifact (interactive, visual, checklist, etc.) might be most helpful for this specific task. Thanks for your help!
*
Help me make sense of these ideas
[](#)
Hi Claude! Could you help me make sense of these ideas? If you need more information from me, ask me 1-2 key questions right away. If you think I should upload any documents that would help you do a better job, let me know. You can use the tools you have access to— like Google Drive, web search, etc.—if they’ll help you better accomplish this task. Do not use analysis tool. Please keep your responses friendly, brief and conversational.
Please execute the task as soon as you can—an artifact would be great if it makes sense. If using an artifact, consider what kind of artifact (interactive, visual, checklist, etc.) might be most helpful for this specific task. Thanks for your help!
*
Prepare for an exam or interview
[](#)
Hi Claude! Could you prepare for an exam or interview? If you need more information from me, ask me 1-2 key questions right away. If you think I should upload any documents that would help you do a better job, let me know. You can use the tools you have access to— like Google Drive, web search, etc.—if they’ll help you better accomplish this task. Do not use analysis tool. Please keep your responses friendly, brief and conversational.
Please execute the task as soon as you can—an artifact would be great if it makes sense. If using an artifact, consider what kind of artifact (interactive, visual, checklist, etc.) might be most helpful for this specific task. Thanks for your help!
Code
*
Explain a programming concept
[](#)
Hi Claude! Could you explain a programming concept? If you need more information from me, ask me 1-2 key questions right away. If you think I should upload any documents that would help you do a better job, let me know. You can use the tools you have access to— like Google Drive, web search, etc.—if they’ll help you better accomplish this task. Do not use analysis tool. Please keep your responses friendly, brief and conversational.
Please execute the task as soon as you can—an artifact would be great if it makes sense. If using an artifact, consider what kind of artifact (interactive, visual, checklist, etc.) might be most helpful for this specific task. Thanks for your help!
*
Look over my code and give me tips
[](#)
Hi Claude! Could you look over my code and give me tips? If you need more information from me, ask me 1-2 key questions right away. If you think I should upload any documents that would help you do a better job, let me know. You can use the tools you have access to— like Google Drive, web search, etc.—if they’ll help you better accomplish this task. Do not use analysis tool. Please keep your responses friendly, brief and conversational.
Please execute the task as soon as you can—an artifact would be great if it makes sense. If using an artifact, consider what kind of artifact (interactive, visual, checklist, etc.) might be most helpful for this specific task. Thanks for your help!
*
Vibe code with me
[](#)
Hi Claude! Could you vibe code with me? If you need more information from me, ask me 1-2 key questions right away. If you think I should upload any documents that would help you do a better job, let me know. You can use the tools you have access to— like Google Drive, web search, etc.—if they’ll help you better accomplish this task. Do not use analysis tool. Please keep your responses friendly, brief and conversational.
Please execute the task as soon as you can—an artifact would be great if it makes sense. If using an artifact, consider what kind of artifact (interactive, visual, checklist, etc.) might be most helpful for this specific task. Thanks for your help!
More
*
Write case studies
[](#)
This is another test
*
Write grant proposals
[](#)
Hi Claude! Could you write grant proposals? If you need more information from me, ask me 1-2 key questions right away. If you think I should upload any documents that would help you do a better job, let me know. You can use the tools you have access to — like Google Drive, web search, etc. — if they’ll help you better accomplish this task. Do not use analysis tool. Please keep your responses friendly, brief and conversational.
Please execute the task as soon as you can - an artifact would be great if it makes sense. If using an artifact, consider what kind of artifact (interactive, visual, checklist, etc.) might be most helpful for this specific task. Thanks for your help!
*
Write video scripts
[](#)
this is a test
[Anthropic](https://www.anthropic.com/)Anthropic
© 2026 Anthropic PBC
Products
*
[Claude](/product/overview)Claude
*
[Claude Code](/product/claude-code)Claude Code
*
[Claude Code for Enterprise](/product/claude-code/enterprise)Claude Code for Enterprise
*
[Claude Cowork](/product/cowork)Claude Cowork
*
[Claude Security](/product/claude-security)Claude Security
*
[Pro plan](/pricing/pro)Pro plan
*
[Max plan](/pricing/max)Max plan
*
[Team plan](/pricing/team)Team plan
*
[Enterprise plan](/pricing/enterprise)Enterprise plan
*
[Download app](/download)Download app
*
[Pricing](/pricing)Pricing
*
[Log in](https://claude.ai/login)Log in
Features
*
[Claude for Chrome](/claude-for-chrome)Claude for Chrome
*
[Claude for Slack](/claude-for-slack)Claude for Slack
*
[Claude for Microsoft 365](/claude-for-microsoft-365)Claude for Microsoft 365
*
[Skills](/skills)Skills
Models
*
[Mythos preview](https://www.anthropic.com/glasswing)Mythos preview
*
[Opus](https://www.anthropic.com/claude/opus)Opus
*
[Sonnet](https://www.anthropic.com/claude/sonnet)Sonnet
*
[Haiku](https://www.anthropic.com/claude/haiku)Haiku
Solutions
*
[AI agents](/solutions/agents)AI agents
*
[Code modernization](/solutions/code-modernization)Code modernization
*
[Coding](/solutions/coding)Coding
*
[Customer support](/solutions/customer-support)Customer support
*
[Education](/solutions/education)Education
*
[Financial services](/solutions/financial-services)Financial services
*
[Government](/solutions/government)Government
*
[Healthcare](/solutions/healthcare)Healthcare
*
[Legal](/solutions/legal)Legal
*
[Life sciences](/solutions/life-sciences)Life sciences
*
[Nonprofits](/solutions/nonprofits)Nonprofits
*
[Security](/solutions/security)Security
Claude Platform
*
[Overview](/platform/api)Overview
*
[Developer docs](https://platform.claude.com/docs)Developer docs
*
[Pricing](https://claude.com/pricing#api)Pricing
*
[Marketplace](/platform/marketplace)Marketplace
*
[Claude on AWS](/partners/claude-on-aws)Claude on AWS
*
[Google Cloud’s Vertex AI](/partners/google-cloud-vertex-ai)Google Cloud’s Vertex AI
*
[Microsoft Foundry](/partners/microsoft-foundry)Microsoft Foundry
*
[Regional compliance](/regional-compliance)Regional compliance
*
[Console login](https://platform.claude.com/)Console login
Resources
*
[Blog](/blog)Blog
*
[Claude partner network](/partners)Claude partner network
*
[Community](/community)Community
*
[Connectors](/connectors)Connectors
*
[Courses](https://www.anthropic.com/learn)Courses
*
[Customer stories](/customers)Customer stories
*
[Engineering at Anthropic](https://www.anthropic.com/engineering)Engineering at Anthropic
*
[Events](https://www.anthropic.com/events)Events
*
[Plugins](/plugins)Plugins
*
[Powered by Claude](/partners/powered-by-claude)Powered by Claude
*
[Service partners](/partners/services)Service partners
*
[Startups program](/programs/startups)Startups program
*
[Tutorials](/resources/tutorials)Tutorials
*
[Use cases](/resources/use-cases)Use cases
Company
*
[Anthropic](https://www.anthropic.com/)Anthropic
*
[Careers](https://www.anthropic.com/careers)Careers
*
[Economic Futures](https://www.anthropic.com/economic-futures)Economic Futures
*
[Research](https://www.anthropic.com/research)Research
*
[News](https://www.anthropic.com/news)News
*
[Responsible Scaling Policy](https://www.anthropic.com/news/announcing-our-updated-responsible-scaling-policy)Responsible Scaling Policy
*
[Security and compliance](https://trust.anthropic.com/)Security and compliance
*
[Transparency](https://anthropic.com/transparency)Transparency
Help and security
*
[Availability](https://www.anthropic.com/supported-countries)Availability
*
[Status](https://status.anthropic.com/)Status
*
[Support center](https://support.claude.com/en/)Support center
Terms and policies
*
### Cookie settings
We use cookies to deliver and improve our services, analyze site usage, and if you agree, to customize or personalize your experience and market our services to you. You can read our Cookie Policy [here](https://www.anthropic.com/legal/cookies).
Customize cookie settings
Reject all cookies
Accept all cookies
###### Necessary
Enables security and basic functionality.
Required
###### Analytics
Enables tracking of site performance.
Off
###### Marketing
Enables ads personalization and tracking.
Off
Save preferences
*
[Privacy policy](https://www.anthropic.com/legal/privacy)Privacy policy
*
[Responsible disclosure policy](https://www.anthropic.com/responsible-disclosure-policy)Responsible disclosure policy
*
[Terms of service: Commercial](https://www.anthropic.com/legal/commercial-terms)Terms of service: Commercial
*
[Terms of service: Consumer](https://www.anthropic.com/legal/consumer-terms)Terms of service: Consumer
*
[Usage policy](https://www.anthropic.com/legal/aup)Usage policy
[x.com](https://x.com/claudeai)x.com
[LinkedIn](https://www.linkedin.com/showcase/claude/)LinkedIn
[YouTube](https://www.youtube.com/@anthropic-ai)YouTube
[Instagram](https://www.instagram.com/claudeai)Instagram
English (US)
[Next](#)Next