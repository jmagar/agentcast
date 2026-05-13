Claude Code on Google Vertex AI - Claude Code Docs
## > Documentation Index
> Fetch the complete documentation index at:
[> https://code.claude.com/docs/llms.txt
](https://code.claude.com/docs/llms.txt)
> Use this file to discover all available pages before exploring further.
##
[​
](#prerequisites)
Prerequisites
Before configuring Claude Code with Vertex AI, ensure you have:
* A Google Cloud Platform (GCP) account with billing enabled
* A GCP project with Vertex AI API enabled
* Access to desired Claude models (for example, Claude Sonnet 4.6)
* Google Cloud SDK (`gcloud`) installed and configured
* Quota allocated in desired GCP region
To sign in with your own Vertex AI credentials, follow [Sign in with Vertex AI](#sign-in-with-vertex-ai) below. To deploy Claude Code across a team, use the [manual setup](#set-up-manually) steps and [pin your model versions](#5-pin-model-versions) before rolling out.
##
[​
](#sign-in-with-vertex-ai)
Sign in with Vertex AI
If you have Google Cloud credentials and want to start using Claude Code through Vertex AI, the login wizard walks you through it. You complete the GCP-side prerequisites once per project; the wizard handles the Claude Code side.
The Vertex AI setup wizard requires Claude Code v2.1.98 or later. Run `claude --version` to check.
1
[
](#)
Enable Claude models in your GCP project
[Enable the Vertex AI API](#1-enable-vertex-ai-api) for your project, then request access to the Claude models you want in the [Vertex AI Model Garden](https://console.cloud.google.com/vertex-ai/model-garden). See [IAM configuration](#iam-configuration) for the permissions your account needs.
2
[
](#)
Start Claude Code and choose Vertex AI
Run `claude`. At the login prompt, select **3rd-party platform**, then **Google Vertex AI**.
3
[
](#)
Follow the wizard prompts
Choose how you authenticate to Google Cloud: Application Default Credentials from `gcloud`, a service account key file, or credentials already in your environment. The wizard detects your project and region, verifies which Claude models your project can invoke, and lets you pin them. It saves the result to the `env` block of your [user settings file](/docs/en/settings), so you don’t need to export environment variables yourself.
After you’ve signed in, run `/setup-vertex` any time to reopen the wizard and change your credentials, project, region, or model pins.
##
[​
](#region-configuration)
Region configuration
Claude Code supports Vertex AI [global](https://cloud.google.com/blog/products/ai-machine-learning/global-endpoint-for-claude-models-generally-available-on-vertex-ai), multi-region, and regional endpoints. Set `CLOUD\_ML\_REGION` to `global`, a multi-region location such as `eu` or `us`, or a specific region such as `us-east5`. Claude Code selects the correct Vertex AI hostname for each form, including the `aiplatform.eu.rep.googleapis.com` and `aiplatform.us.rep.googleapis.com` hosts for multi-region locations.
Vertex AI may not support the Claude Code default models on every endpoint type. Model availability varies across [specific regions](https://cloud.google.com/vertex-ai/generative-ai/docs/learn/locations#genai-partner-models), multi-region locations, and [global endpoints](https://cloud.google.com/vertex-ai/generative-ai/docs/partner-models/use-partner-models#supported_models). You may need to switch to a supported location or specify a supported model.
##
[​
](#set-up-manually)
Set up manually
To configure Vertex AI through environment variables instead of the wizard, for example in CI or a scripted enterprise rollout, follow the steps below.
###
[​
](#1-enable-vertex-ai-api)
1. Enable Vertex AI API
Enable the Vertex AI API in your GCP project:
```
`# Set your project ID
gcloud config set project YOUR-PROJECT-ID
# Enable Vertex AI API
gcloud services enable aiplatform.googleapis.com
`
```
###
[​
](#2-request-model-access)
2. Request model access
Request access to Claude models in Vertex AI:
1. Navigate to the [Vertex AI Model Garden](https://console.cloud.google.com/vertex-ai/model-garden)
2. Search for “Claude” models
3. Request access to desired Claude models (for example, Claude Sonnet 4.6)
4. Wait for approval (may take 24-48 hours)
###
[​
](#3-configure-gcp-credentials)
3. Configure GCP credentials
Claude Code uses standard Google Cloud authentication.
For more information, see [Google Cloud authentication documentation](https://cloud.google.com/docs/authentication).
Claude Code v2.1.121 or later supports [X.509 certificate-based Workload Identity Federation](https://cloud.google.com/iam/docs/workload-identity-federation-with-x509-certificates) through the same Application Default Credentials chain. Set `GOOGLE\_APPLICATION\_CREDENTIALS` to the path of your credential configuration file.
Claude Code uses `ANTHROPIC\_VERTEX\_PROJECT\_ID` as the project ID for Vertex AI requests. The `GCLOUD\_PROJECT` and `GOOGLE\_CLOUD\_PROJECT` environment variables and the credential file referenced by `GOOGLE\_APPLICATION\_CREDENTIALS` take precedence over it. If none of these are set, the project ID is resolved from your `gcloud` configuration or the attached service account.
####
[​
](#advanced-credential-configuration)
Advanced credential configuration
Claude Code supports automatic credential refresh for GCP through the `gcpAuthRefresh` setting. When Claude Code detects that your GCP credentials are expired or cannot be loaded, it runs the configured command to obtain new credentials before retrying the request.
```
`{
"gcpAuthRefresh": "gcloud auth application-default login",
"env": {
"ANTHROPIC\_VERTEX\_PROJECT\_ID": "your-project-id"
}
}
`
```
The command’s output is displayed to the user, but interactive input isn’t supported. This works well for browser-based authentication flows where the CLI shows a URL and you complete authentication in the browser. The refresh command times out after three minutes if authentication does not complete. If you set `gcpAuthRefresh` in project settings such as `.claude/settings.json`, the command runs only after you accept the workspace trust prompt.
###
[​
](#4-configure-claude-code)
4. Configure Claude Code
Set the following environment variables:
```
`# Enable Vertex AI integration
export CLAUDE\_CODE\_USE\_VERTEX=1
export CLOUD\_ML\_REGION=global
export ANTHROPIC\_VERTEX\_PROJECT\_ID=YOUR-PROJECT-ID
# Optional: Override the Vertex endpoint URL for custom endpoints or gateways
# export ANTHROPIC\_VERTEX\_BASE\_URL=https://aiplatform.googleapis.com
# Optional: Disable prompt caching if needed
export DISABLE\_PROMPT\_CACHING=1
# Optional: Request 1-hour prompt cache TTL instead of the 5-minute default
export ENABLE\_PROMPT\_CACHING\_1H=1
# When CLOUD\_ML\_REGION=global, override region for models that don't support global endpoints
export VERTEX\_REGION\_CLAUDE\_HAIKU\_4\_5=us-east5
export VERTEX\_REGION\_CLAUDE\_4\_6\_SONNET=europe-west1
`
```
Most model versions have a corresponding `VERTEX\_REGION\_CLAUDE\_\*` variable. See the [Environment variables reference](/docs/en/env-vars) for the full list. Check [Vertex Model Garden](https://console.cloud.google.com/vertex-ai/model-garden) to determine which models support global endpoints versus regional only.
[Prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching) is enabled automatically. To disable it, set `DISABLE\_PROMPT\_CACHING=1`. To request a 1-hour cache TTL instead of the 5-minute default, set `ENABLE\_PROMPT\_CACHING\_1H=1`; cache writes with a 1-hour TTL are billed at a higher rate. For heightened rate limits, contact Google Cloud support. When using Vertex AI, the `/login` and `/logout` commands are disabled since authentication is handled through Google Cloud credentials.
[MCP tool search](/docs/en/mcp#scale-with-mcp-tool-search) is disabled by default on Vertex AI because the endpoint does not accept the required beta header. All MCP tool definitions load upfront instead. Setting `ENABLE\_TOOL\_SEARCH=true` forces Claude Code to send the header anyway, which causes Vertex AI to reject requests.
###
[​
](#5-pin-model-versions)
5. Pin model versions
Pin specific model versions when deploying to multiple users. Without pinning, model aliases such as `sonnet` and `opus` resolve to the latest version, which may not yet be enabled in your Vertex AI project when Anthropic releases an update. Claude Code [falls back](#startup-model-checks) to the previous version at startup when the latest is unavailable, but pinning lets you control when your users move to a new model.
Set these environment variables to specific Vertex AI model IDs.
Without `ANTHROPIC\_DEFAULT\_OPUS\_MODEL`, the `opus` alias on Vertex resolves to Opus 4.6. Set it to the Opus 4.7 ID to use the latest model:
```
`export ANTHROPIC\_DEFAULT\_OPUS\_MODEL='claude-opus-4-7'
export ANTHROPIC\_DEFAULT\_SONNET\_MODEL='claude-sonnet-4-6'
export ANTHROPIC\_DEFAULT\_HAIKU\_MODEL='claude-haiku-4-5@20251001'
`
```
For current and legacy model IDs, see [Models overview](https://platform.claude.com/docs/en/about-claude/models/overview). See [Model configuration](/docs/en/model-config#pin-models-for-third-party-deployments) for the full list of environment variables.
Claude Code uses these default models when no pinning variables are set:
|Model type|Default value|
|Primary model|`claude-sonnet-4-5@20250929`|
|Small/fast model|`claude-haiku-4-5@20251001`|
To customize models further:
```
`export ANTHROPIC\_MODEL='claude-opus-4-7'
export ANTHROPIC\_DEFAULT\_HAIKU\_MODEL='claude-haiku-4-5@20251001'
`
```
##
[​
](#startup-model-checks)
Startup model checks
When Claude Code starts with Vertex AI configured, it verifies that the models it intends to use are accessible in your project. This check requires Claude Code v2.1.98 or later.
If you have pinned a model version that is older than the current Claude Code default, and your project can invoke the newer version, Claude Code prompts you to update the pin. Accepting writes the new model ID to your [user settings file](/docs/en/settings) and restarts Claude Code. Declining is remembered until the next default version change.
If you have not pinned a model and the current default is unavailable in your project, Claude Code falls back to the previous version for the current session and shows a notice. The fallback is not persisted. Enable the newer model in [Model Garden](https://console.cloud.google.com/vertex-ai/model-garden) or [pin a version](#5-pin-model-versions) to make the choice permanent.
##
[​
](#iam-configuration)
IAM configuration
Assign the required IAM permissions:
The `roles/aiplatform.user` role includes the required permissions:
* `aiplatform.endpoints.predict` - Required for model invocation and token counting
For more restrictive permissions, create a custom role with only the permissions above.
For details, see [Vertex IAM documentation](https://cloud.google.com/vertex-ai/docs/general/access-control).
Create a dedicated GCP project for Claude Code to simplify cost tracking and access control.
##
[​
](#1m-token-context-window)
1M token context window
Claude Opus 4.7, Opus 4.6, and Sonnet 4.6 support the [1M token context window](https://platform.claude.com/docs/en/build-with-claude/context-windows#1m-token-context-window) on Vertex AI. Claude Code automatically enables the extended context window when you select a 1M model variant.
The [setup wizard](#sign-in-with-vertex-ai) offers a 1M context option when it pins models. To enable it for a manually pinned model instead, append `[1m]` to the model ID. See [Pin models for third-party deployments](/docs/en/model-config#pin-models-for-third-party-deployments) for details.
##
[​
](#troubleshooting)
Troubleshooting
If you encounter “Could not load the default credentials” errors:
* Run `gcloud auth application-default login` to set up Application Default Credentials
* Set `GOOGLE\_APPLICATION\_CREDENTIALS` to a service account key file path
* See [Configure GCP credentials](#3-configure-gcp-credentials) for all options
If you encounter quota issues:
* Check current quotas or request quota increase through [Cloud Console](https://cloud.google.com/docs/quotas/view-manage)
If you encounter “model not found” 404 errors:
* Confirm model is Enabled in [Model Garden](https://console.cloud.google.com/vertex-ai/model-garden)
* Verify the model is available in the location you specified. Some models are offered only on `global` or multi-region locations such as `eu` and `us`, not in specific regions
* If using `CLOUD\_ML\_REGION=global`, check that your models support global endpoints in [Model Garden](https://console.cloud.google.com/vertex-ai/model-garden) under “Supported features”. For models that don’t support global endpoints, either:
* Specify a supported model via `ANTHROPIC\_MODEL` or `ANTHROPIC\_DEFAULT\_HAIKU\_MODEL`, or
* Set a region or multi-region location using `VERTEX\_REGION\_\<MODEL\_NAME\>` environment variables
If you encounter 429 errors:
* For regional endpoints, ensure the primary model and small/fast model are supported in your selected region
* Consider switching to `CLOUD\_ML\_REGION=global` for better availability
##
[​
](#additional-resources)
Additional resources
* [Vertex AI documentation](https://cloud.google.com/vertex-ai/docs)
* [Vertex AI pricing](https://cloud.google.com/vertex-ai/pricing)
* [Vertex AI quotas and limits](https://cloud.google.com/vertex-ai/docs/quotas)
⌘I