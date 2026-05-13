LLM gateway configuration - Claude Code Docs
## > Documentation Index
> Fetch the complete documentation index at:
[> https://code.claude.com/docs/llms.txt
](https://code.claude.com/docs/llms.txt)
> Use this file to discover all available pages before exploring further.
LLM gateways provide a centralized proxy layer between Claude Code and model providers, often providing:
* **Centralized authentication** - Single point for API key management
* **Usage tracking** - Monitor usage across teams and projects
* **Cost controls** - Implement budgets and rate limits
* **Audit logging** - Track all model interactions for compliance
* **Model routing** - Switch between providers without code changes
##
[​
](#gateway-requirements)
Gateway requirements
For an LLM gateway to work with Claude Code, it must meet the following requirements:
**API format**
The gateway must expose to clients at least one of the following API formats:
1. **Anthropic Messages**: `/v1/messages`, `/v1/messages/count\_tokens`
* Must forward request headers: `anthropic-beta`, `anthropic-version`
* **Bedrock InvokeModel**: `/invoke`, `/invoke-with-response-stream`
* Must preserve request body fields: `anthropic\_beta`, `anthropic\_version`
* **Vertex rawPredict**: `:rawPredict`, `:streamRawPredict`, `/count-tokens:rawPredict`
* Must forward request headers: `anthropic-beta`, `anthropic-version`
Failure to forward headers or preserve body fields may result in reduced functionality or inability to use Claude Code features.
Claude Code determines which features to enable based on the API format. When using the Anthropic Messages format with Bedrock or Vertex, you may need to set environment variable `CLAUDE\_CODE\_DISABLE\_EXPERIMENTAL\_BETAS=1`.
**Request headers**
Claude Code includes the following headers on API requests:
|Header|Description|
|`X-Claude-Code-Session-Id`|A unique identifier for the current Claude Code session. Proxies can use this to aggregate all API requests from a single session without parsing the request body.|
|`X-Claude-Code-Agent-Id`|Identifier of the subagent or teammate that issued the request. Your proxy can use this to attribute API cost to individual parallel subagents within a session, without parsing the request body. Present only for requests made by an in-process subagent or teammate.|
|`X-Claude-Code-Parent-Agent-Id`|Identifier of the agent that spawned the agent making the request. Use this with `X-Claude-Code-Agent-Id` to attribute API costs across nested agents in your proxy. Present only when the requesting agent was itself spawned by another agent.|
Both agent ID headers are ephemeral per-spawn identifiers, not persistent user or device IDs.
Claude Code also prepends a short attribution block to the system prompt containing the client version and a fingerprint derived from the conversation. The Anthropic API strips this block before processing, so it does not affect first-party prompt caching. If your gateway implements its own prompt cache keyed on the full request body, set [`CLAUDE\_CODE\_ATTRIBUTION\_HEADER=0`](/docs/en/env-vars) to omit it.
##
[​
](#configuration)
Configuration
###
[​
](#model-selection)
Model selection
By default, Claude Code uses standard model names for the selected API format.
When `ANTHROPIC\_BASE\_URL` points at a gateway that exposes the Anthropic Messages format, Claude Code can query the gateway’s `/v1/models` endpoint at startup and add the returned models to the `/model` picker. Set `CLAUDE\_CODE\_ENABLE\_GATEWAY\_MODEL\_DISCOVERY=1` to enable this. Discovery is off by default so that gateways backed by a shared API key do not surface every model the key can access to every user. Each discovered entry is labeled “From gateway” and uses the `display\_name` field from the response when one is provided. This requires Claude Code v2.1.129 or later.
Discovery applies only to the Anthropic Messages format. It does not run for Bedrock or Vertex pass-through endpoints, and it does not run when `ANTHROPIC\_BASE\_URL` is unset or points at `api.anthropic.com`.
The discovery request authenticates the same way as inference requests: it sends `ANTHROPIC\_AUTH\_TOKEN` as a bearer token, or `ANTHROPIC\_API\_KEY` as the `x-api-key` header when no auth token is set, along with any headers from `ANTHROPIC\_CUSTOM\_HEADERS`. Only models whose ID begins with `claude` or `anthropic` are added to the picker. Results are cached to `\~/.claude/cache/gateway-models.json` and refreshed on each startup. If the request fails or the gateway does not implement `/v1/models`, the picker falls back to the cached list from the previous startup or to the built-in model list.
If your gateway uses model names that do not match the discovery filter, use the environment variables documented in [Model configuration](/docs/en/model-config) to add them manually.
##
[​
](#litellm-configuration)
LiteLLM configuration
LiteLLM PyPI versions 1.82.7 and 1.82.8 were compromised with credential-stealing malware. Do not install these versions. If you have already installed them:
* Remove the package
* Rotate all credentials on affected systems
* Follow the remediation steps in [BerriAI/litellm#24518](https://github.com/BerriAI/litellm/issues/24518)
LiteLLM is a third-party proxy service. Anthropic doesn’t endorse, maintain, or audit LiteLLM’s security or functionality. This guide is provided for informational purposes and may become outdated. Use at your own discretion.
###
[​
](#prerequisites)
Prerequisites
* Claude Code updated to the latest version
* LiteLLM Proxy Server deployed and accessible
* Access to Claude models through your chosen provider
###
[​
](#basic-litellm-setup)
Basic LiteLLM setup
**Configure Claude Code**:
####
[​
](#authentication-methods)
Authentication methods
##### Static API key
Simplest method using a fixed API key:
```
`# Set in environment
export ANTHROPIC\_AUTH\_TOKEN=sk-litellm-static-key
# Or in Claude Code settings
{
"env": {
"ANTHROPIC\_AUTH\_TOKEN": "sk-litellm-static-key"
}
}
`
```
This value will be sent as the `Authorization` header.
##### Dynamic API key with helper
For rotating keys or per-user authentication:
1. Create an API key helper script:
```
`#!/bin/bash
# \~/bin/get-litellm-key.sh
# Example: Fetch key from vault
vault kv get -field=api\_key secret/litellm/claude-code
# Example: Generate JWT token
jwt encode \\
--secret="${JWT\_SECRET}" \\
--exp="+1h" \\
'{"user":"'${USER}'","team":"engineering"}'
`
```
1. Configure Claude Code settings to use the helper:
```
`{
"apiKeyHelper": "\~/bin/get-litellm-key.sh"
}
`
```
1. Set token refresh interval:
```
`# Refresh every hour (3600000 ms)
export CLAUDE\_CODE\_API\_KEY\_HELPER\_TTL\_MS=3600000
`
```
This value will be sent as `Authorization` and `X-Api-Key` headers. The `apiKeyHelper` has lower precedence than `ANTHROPIC\_AUTH\_TOKEN` or `ANTHROPIC\_API\_KEY`.
####
[​
](#unified-endpoint-recommended)
Unified endpoint (recommended)
Using LiteLLM’s [Anthropic format endpoint](https://docs.litellm.ai/docs/anthropic_unified):
```
`export ANTHROPIC\_BASE\_URL=https://litellm-server:4000
`
```
**Benefits of the unified endpoint over pass-through endpoints:**
* Load balancing
* Fallbacks
* Consistent support for cost tracking and end-user tracking
####
[​
](#provider-specific-pass-through-endpoints-alternative)
Provider-specific pass-through endpoints (alternative)
##### Claude API through LiteLLM
Using [pass-through endpoint](https://docs.litellm.ai/docs/pass_through/anthropic_completion):
```
`export ANTHROPIC\_BASE\_URL=https://litellm-server:4000/anthropic
`
```
##### Amazon Bedrock through LiteLLM
Using [pass-through endpoint](https://docs.litellm.ai/docs/pass_through/bedrock):
```
`export ANTHROPIC\_BEDROCK\_BASE\_URL=https://litellm-server:4000/bedrock
export CLAUDE\_CODE\_SKIP\_BEDROCK\_AUTH=1
export CLAUDE\_CODE\_USE\_BEDROCK=1
`
```
##### Google Vertex AI through LiteLLM
Using [pass-through endpoint](https://docs.litellm.ai/docs/pass_through/vertex_ai):
```
`export ANTHROPIC\_VERTEX\_BASE\_URL=https://litellm-server:4000/vertex\_ai/v1
export ANTHROPIC\_VERTEX\_PROJECT\_ID=your-gcp-project-id
export CLAUDE\_CODE\_SKIP\_VERTEX\_AUTH=1
export CLAUDE\_CODE\_USE\_VERTEX=1
export CLOUD\_ML\_REGION=us-east5
`
```
##### Claude Platform on AWS through a gateway
Route to a gateway that forwards to the [Claude Platform on AWS](/docs/en/claude-platform-on-aws) endpoint:
```
`export ANTHROPIC\_AWS\_BASE\_URL=https://litellm-server:4000/anthropic-aws
export ANTHROPIC\_AWS\_WORKSPACE\_ID=wrkspc\_01ABCDEFGHIJKLMN
export CLAUDE\_CODE\_SKIP\_ANTHROPIC\_AWS\_AUTH=1
export CLAUDE\_CODE\_USE\_ANTHROPIC\_AWS=1
`
```
For more detailed information, refer to the [LiteLLM documentation](https://docs.litellm.ai/).
##
[​
](#additional-resources)
Additional resources
* [LiteLLM documentation](https://docs.litellm.ai/)
* [Claude Code settings](/docs/en/settings)
* [Enterprise network configuration](/docs/en/network-config)
* [Third-party integrations overview](/docs/en/third-party-integrations)
⌘I