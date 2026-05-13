Model Config Option Category - Agent Client Protocol
[Protocol
](/get-started/introduction)[RFDs
](/rfds/about)[Community
](/community/communication)[Publications
](/publications)[Updates
](/updates)[Brand
](/brand)
## > Documentation Index
> Fetch the complete documentation index at:
[> https://agentclientprotocol.com/llms.txt
](https://agentclientprotocol.com/llms.txt)
> Use this file to discover all available pages before exploring further.
* Author(s): [anna239](https://github.com/anna239)
##
[​
](#elevator-pitch)
Elevator pitch
Add a new `model\_config` category to session configuration options, so that agents can expose model-related parameters (context size, speed/quality trade-offs, etc) and clients can group them alongside the main model selector in the UI.
##
[​
](#status-quo)
Status quo
The `category` field on `SessionConfigOption` currently supports three values: `mode`, `model`, and `thought\_level`. This works well when the model is a single selector, but some agents expose many model configurations — context window size, speed tier, and similar settings that logically belong next to the model picker.
##
[​
](#what-we-propose-to-do-about-it)
What we propose to do about it
Add a `model\_config` variant to `SessionConfigOptionCategory`.
Agents tag any model-related parameters with `"category": "model\_config"`, and clients render them near the primary `model` selector — for example as secondary controls within a model-picker popover or panel.
###
[​
](#relationship-to-thought_level)
Relationship to `thought\_level`
Once `model\_config` exists, `thought\_level` is semantically a special case of a model configuration parameter. We keep `thought\_level` as-is for backward compatibility — existing clients already handle it — but new model-related options should use `model\_config`.
##
[​
](#shiny-future)
Shiny future
Agents expose rich, parameterized model configurations over ACP.
##
[​
](#implementation-details-and-plan)
Implementation details and plan
###
[​
](#json-format)
JSON format
An agent declares model-config options in `configOptions`:
```
`{
"configOptions": [
{
"id": "model",
"name": "Model",
"category": "model",
"type": "select",
"currentValue": "sonnet-4.5",
"options": [
{ "value": "sonnet-4.5", "name": "Sonnet 4.5" },
{ "value": "opus-4.6", "name": "Opus 4.6" }
]
},
{
"id": "context\_size",
"name": "Context Size",
"category": "model\_config",
"type": "select",
"currentValue": "200k",
"options": [
{ "value": "200k", "name": "200K" },
{ "value": "1m", "name": "1M" }
]
},
{
"id": "fast\_mode",
"name": "Fast Mode",
"category": "model\_config",
"type": "boolean",
"currentValue": false
}
]
}
`
```
###
[​
](#client-behavior)
Client behavior
* Clients SHOULD render `model\_config` options near the `model` selector (e.g., in the same popover or panel).
* Clients that do not recognize the category MUST handle it gracefully per the existing spec — the option still renders, just without special placement.
* No new client capability negotiation is needed.
###
[​
](#should-thought_level-move-under-model_config)
Should `thought\_level` move under `model\_config`?
Not now. Existing clients already handle `thought\_level`, so changing its semantics would be a breaking change. New model-related parameters should use `model\_config`; `thought\_level` remains for backward compatibility.
##
[​
](#revision-history)
Revision history
* 2026-04-08: Initial proposal