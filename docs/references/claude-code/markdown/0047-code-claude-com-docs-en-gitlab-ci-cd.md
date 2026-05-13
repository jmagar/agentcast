Claude Code GitLab CI/CD - Claude Code Docs
## > Documentation Index
> Fetch the complete documentation index at:
[> https://code.claude.com/docs/llms.txt
](https://code.claude.com/docs/llms.txt)
> Use this file to discover all available pages before exploring further.
Claude Code for GitLab CI/CD is currently in beta. Features and functionality may evolve as we refine the experience.This integration is maintained by GitLab. For support, see the following [GitLab issue](https://gitlab.com/gitlab-org/gitlab/-/issues/573776).
This integration is built on top of the [Claude Code CLI and Agent SDK](/docs/en/agent-sdk/overview), enabling programmatic use of Claude in your CI/CD jobs and custom automation workflows.
##
[​
](#why-use-claude-code-with-gitlab)
Why use Claude Code with GitLab?
* **Instant MR creation**: Describe what you need, and Claude proposes a complete MR with changes and explanation
* **Automated implementation**: Turn issues into working code with a single command or mention
* **Project-aware**: Claude follows your `CLAUDE.md` guidelines and existing code patterns
* **Simple setup**: Add one job to `.gitlab-ci.yml` and a masked CI/CD variable
* **Enterprise-ready**: Choose Claude API, Amazon Bedrock, or Google Vertex AI to meet data residency and procurement needs
* **Secure by default**: Runs in your GitLab runners with your branch protection and approvals
##
[​
](#how-it-works)
How it works
Claude Code uses GitLab CI/CD to run AI tasks in isolated jobs and commit results back via MRs:
1. **Event-driven orchestration**: GitLab listens for your chosen triggers (for example, a comment that mentions `@claude` in an issue, MR, or review thread). The job collects context from the thread and repository, builds prompts from that input, and runs Claude Code.
2. **Provider abstraction**: Use the provider that fits your environment:
* Claude API (SaaS)
* Amazon Bedrock (IAM-based access, cross-region options)
* Google Vertex AI (GCP-native, Workload Identity Federation)
* **Sandboxed execution**: Each interaction runs in a container with strict network and filesystem rules. Claude Code enforces workspace-scoped permissions to constrain writes. Every change flows through an MR so reviewers see the diff and approvals still apply.
Pick regional endpoints to reduce latency and meet data-sovereignty requirements while using existing cloud agreements.
##
[​
](#what-can-claude-do)
What can Claude do?
Claude Code enables powerful CI/CD workflows that transform how you work with code:
* Create and update MRs from issue descriptions or comments
* Analyze performance regressions and propose optimizations
* Implement features directly in a branch, then open an MR
* Fix bugs and regressions identified by tests or comments
* Respond to follow-up comments to iterate on requested changes
##
[​
](#setup)
Setup
###
[​
](#quick-setup)
Quick setup
The fastest way to get started is to add a minimal job to your `.gitlab-ci.yml` and set your API key as a masked variable.
1. **Add a masked CI/CD variable**
* Go to **Settings** → **CI/CD** → **Variables**
* Add `ANTHROPIC\_API\_KEY` (masked, protected as needed)
* **Add a Claude job to `.gitlab-ci.yml`**
```
`stages:
- ai
claude:
stage: ai
image: node:24-alpine3.21
# Adjust rules to fit how you want to trigger the job:
# - manual runs
# - merge request events
# - web/API triggers when a comment contains '@claude'
rules:
- if: '$CI\_PIPELINE\_SOURCE == "web"'
- if: '$CI\_PIPELINE\_SOURCE == "merge\_request\_event"'
variables:
GIT\_STRATEGY: fetch
before\_script:
- apk update
- apk add --no-cache git curl bash
- curl -fsSL https://claude.ai/install.sh | bash
script:
# Optional: start a GitLab MCP server if your setup provides one
- /bin/gitlab-mcp-server || true
# Use AI\_FLOW\_\* variables when invoking via web/API triggers with context payloads
- echo "$AI\_FLOW\_INPUT for $AI\_FLOW\_CONTEXT on $AI\_FLOW\_EVENT"
- \>
claude
-p "${AI\_FLOW\_INPUT:-'Review this MR and implement the requested changes'}"
--permission-mode acceptEdits
--allowedTools "Bash Read Edit Write mcp\_\_gitlab"
--debug
`
```
After adding the job and your `ANTHROPIC\_API\_KEY` variable, test by running the job manually from **CI/CD** → **Pipelines**, or trigger it from an MR to let Claude propose updates in a branch and open an MR if needed.
To run on Amazon Bedrock or Google Vertex AI instead of the Claude API, see the [Using with Amazon Bedrock & Google Vertex AI](#using-with-amazon-bedrock--google-vertex-ai) section below for authentication and environment setup.
###
[​
](#manual-setup-recommended-for-production)
Manual setup (recommended for production)
If you prefer a more controlled setup or need enterprise providers:
1. **Configure provider access**:
* **Claude API**: Create and store `ANTHROPIC\_API\_KEY` as a masked CI/CD variable
* **Amazon Bedrock**: **Configure GitLab** → **AWS OIDC** and create an IAM role for Bedrock
* **Google Vertex AI**: **Configure Workload Identity Federation for GitLab** → **GCP**
* **Add project credentials for GitLab API operations**:
* Use `CI\_JOB\_TOKEN` by default, or create a Project Access Token with `api` scope
* Store as `GITLAB\_ACCESS\_TOKEN` (masked) if using a PAT
* **Add the Claude job to `.gitlab-ci.yml`** (see examples below)
* **(Optional) Enable mention-driven triggers**:
* Add a project webhook for “Comments (notes)” to your event listener (if you use one)
* Have the listener call the pipeline trigger API with variables like `AI\_FLOW\_INPUT` and `AI\_FLOW\_CONTEXT` when a comment contains `@claude`
##
[​
](#example-use-cases)
Example use cases
###
[​
](#turn-issues-into-mrs)
Turn issues into MRs
In an issue comment:
```
`@claude implement this feature based on the issue description
`
```
Claude analyzes the issue and codebase, writes changes in a branch, and opens an MR for review.
###
[​
](#get-implementation-help)
Get implementation help
In an MR discussion:
```
`@claude suggest a concrete approach to cache the results of this API call
`
```
Claude proposes changes, adds code with appropriate caching, and updates the MR.
###
[​
](#fix-bugs-quickly)
Fix bugs quickly
In an issue or MR comment:
```
`@claude fix the TypeError in the user dashboard component
`
```
Claude locates the bug, implements a fix, and updates the branch or opens a new MR.
##
[​
](#using-with-amazon-bedrock-&amp;-google-vertex-ai)
Using with Amazon Bedrock & Google Vertex AI
For enterprise environments, you can run Claude Code entirely on your cloud infrastructure with the same developer experience.
*
Amazon Bedrock
*
Google Vertex AI
###
[​
](#prerequisites)
Prerequisites
Before setting up Claude Code with Amazon Bedrock, you need:
1. An AWS account with Amazon Bedrock access to the desired Claude models
2. GitLab configured as an OIDC identity provider in AWS IAM
3. An IAM role with Bedrock permissions and a trust policy restricted to your GitLab project/refs
4. GitLab CI/CD variables for role assumption:
* `AWS\_ROLE\_TO\_ASSUME` (role ARN)
* `AWS\_REGION` (Bedrock region)
###
[​
](#setup-instructions)
Setup instructions
Configure AWS to allow GitLab CI jobs to assume an IAM role via OIDC (no static keys).**Required setup:**
1. Enable Amazon Bedrock and request access to your target Claude models
2. Create an IAM OIDC provider for GitLab if not already present
3. Create an IAM role trusted by the GitLab OIDC provider, restricted to your project and protected refs
4. Attach least-privilege permissions for Bedrock invoke APIs
**Required values to store in CI/CD variables:**
* `AWS\_ROLE\_TO\_ASSUME`
* `AWS\_REGION`
Add variables in Settings → CI/CD → Variables:
```
`# For Amazon Bedrock:
- AWS\_ROLE\_TO\_ASSUME
- AWS\_REGION
`
```
Use the Amazon Bedrock job example above to exchange the GitLab job token for temporary AWS credentials at runtime.
###
[​
](#prerequisites-2)
Prerequisites
Before setting up Claude Code with Google Vertex AI, you need:
1. A Google Cloud project with:
* Vertex AI API enabled
* Workload Identity Federation configured to trust GitLab OIDC
* A dedicated service account with only the required Vertex AI roles
* GitLab CI/CD variables for WIF:
* `GCP\_WORKLOAD\_IDENTITY\_PROVIDER` (full resource name)
* `GCP\_SERVICE\_ACCOUNT` (service account email)
###
[​
](#setup-instructions-2)
Setup instructions
Configure Google Cloud to allow GitLab CI jobs to impersonate a service account via Workload Identity Federation.**Required setup:**
1. Enable IAM Credentials API, STS API, and Vertex AI API
2. Create a Workload Identity Pool and provider for GitLab OIDC
3. Create a dedicated service account with Vertex AI roles
4. Grant the WIF principal permission to impersonate the service account
**Required values to store in CI/CD variables:**
* `GCP\_WORKLOAD\_IDENTITY\_PROVIDER`
* `GCP\_SERVICE\_ACCOUNT`
Add variables in Settings → CI/CD → Variables:
```
`# For Google Vertex AI:
- GCP\_WORKLOAD\_IDENTITY\_PROVIDER
- GCP\_SERVICE\_ACCOUNT
- CLOUD\_ML\_REGION (for example, us-east5)
`
```
Use the Google Vertex AI job example above to authenticate without storing keys.
##
[​
](#configuration-examples)
Configuration examples
Below are ready-to-use snippets you can adapt to your pipeline.
###
[​
](#basic-gitlab-ci-yml-claude-api)
Basic .gitlab-ci.yml (Claude API)
```
`stages:
- ai
claude:
stage: ai
image: node:24-alpine3.21
rules:
- if: '$CI\_PIPELINE\_SOURCE == "web"'
- if: '$CI\_PIPELINE\_SOURCE == "merge\_request\_event"'
variables:
GIT\_STRATEGY: fetch
before\_script:
- apk update
- apk add --no-cache git curl bash
- curl -fsSL https://claude.ai/install.sh | bash
script:
- /bin/gitlab-mcp-server || true
- \>
claude
-p "${AI\_FLOW\_INPUT:-'Summarize recent changes and suggest improvements'}"
--permission-mode acceptEdits
--allowedTools "Bash Read Edit Write mcp\_\_gitlab"
--debug
# Claude Code will use ANTHROPIC\_API\_KEY from CI/CD variables
`
```
###
[​
](#amazon-bedrock-job-example-oidc)
Amazon Bedrock job example (OIDC)
**Prerequisites:**
* Amazon Bedrock enabled with access to your chosen Claude model(s)
* GitLab OIDC configured in AWS with a role that trusts your GitLab project and refs
* IAM role with Bedrock permissions (least privilege recommended)
**Required CI/CD variables:**
* `AWS\_ROLE\_TO\_ASSUME`: ARN of the IAM role for Bedrock access
* `AWS\_REGION`: Bedrock region (for example, `us-west-2`)
```
`claude-bedrock:
stage: ai
image: node:24-alpine3.21
rules:
- if: '$CI\_PIPELINE\_SOURCE == "web"'
before\_script:
- apk add --no-cache bash curl jq git python3 py3-pip
- pip install --no-cache-dir awscli
- curl -fsSL https://claude.ai/install.sh | bash
# Exchange GitLab OIDC token for AWS credentials
- export AWS\_WEB\_IDENTITY\_TOKEN\_FILE="${CI\_JOB\_JWT\_FILE:-/tmp/oidc\_token}"
- if [ -n "${CI\_JOB\_JWT\_V2}" ]; then printf "%s" "$CI\_JOB\_JWT\_V2" \> "$AWS\_WEB\_IDENTITY\_TOKEN\_FILE"; fi
- \>
aws sts assume-role-with-web-identity
--role-arn "$AWS\_ROLE\_TO\_ASSUME"
--role-session-name "gitlab-claude-$(date +%s)"
--web-identity-token "file://$AWS\_WEB\_IDENTITY\_TOKEN\_FILE"
--duration-seconds 3600 \> /tmp/aws\_creds.json
- export AWS\_ACCESS\_KEY\_ID="$(jq -r .Credentials.AccessKeyId /tmp/aws\_creds.json)"
- export AWS\_SECRET\_ACCESS\_KEY="$(jq -r .Credentials.SecretAccessKey /tmp/aws\_creds.json)"
- export AWS\_SESSION\_TOKEN="$(jq -r .Credentials.SessionToken /tmp/aws\_creds.json)"
script:
- /bin/gitlab-mcp-server || true
- \>
claude
-p "${AI\_FLOW\_INPUT:-'Implement the requested changes and open an MR'}"
--permission-mode acceptEdits
--allowedTools "Bash Read Edit Write mcp\_\_gitlab"
--debug
variables:
AWS\_REGION: "us-west-2"
`
```
Model IDs for Bedrock include region-specific prefixes (for example, `us.anthropic.claude-sonnet-4-6`). Pass the desired model via your job configuration or prompt if your workflow supports it.
###
[​
](#google-vertex-ai-job-example-workload-identity-federation)
Google Vertex AI job example (Workload Identity Federation)
**Prerequisites:**
* Vertex AI API enabled in your GCP project
* Workload Identity Federation configured to trust GitLab OIDC
* A service account with Vertex AI permissions
**Required CI/CD variables:**
* `GCP\_WORKLOAD\_IDENTITY\_PROVIDER`: Full provider resource name
* `GCP\_SERVICE\_ACCOUNT`: Service account email
* `CLOUD\_ML\_REGION`: Vertex region (for example, `us-east5`)
```
`claude-vertex:
stage: ai
image: gcr.io/google.com/cloudsdktool/google-cloud-cli:slim
rules:
- if: '$CI\_PIPELINE\_SOURCE == "web"'
before\_script:
- apt-get update && apt-get install -y git && apt-get clean
- curl -fsSL https://claude.ai/install.sh | bash
# Authenticate to Google Cloud via WIF (no downloaded keys)
- \>
gcloud auth login --cred-file=\<(cat \<\<EOF
{
"type": "external\_account",
"audience": "${GCP\_WORKLOAD\_IDENTITY\_PROVIDER}",
"subject\_token\_type": "urn:ietf:params:oauth:token-type:jwt",
"service\_account\_impersonation\_url": "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/${GCP\_SERVICE\_ACCOUNT}:generateAccessToken",
"token\_url": "https://sts.googleapis.com/v1/token"
}
EOF
)
- gcloud config set project "$(gcloud projects list --format='value(projectId)' --filter="name:${CI\_PROJECT\_NAMESPACE}" | head -n1)" || true
script:
- /bin/gitlab-mcp-server || true
- \>
CLOUD\_ML\_REGION="${CLOUD\_ML\_REGION:-us-east5}"
claude
-p "${AI\_FLOW\_INPUT:-'Review and update code as requested'}"
--permission-mode acceptEdits
--allowedTools "Bash Read Edit Write mcp\_\_gitlab"
--debug
variables:
CLOUD\_ML\_REGION: "us-east5"
`
```
With Workload Identity Federation, you do not need to store service account keys. Use repository-specific trust conditions and least-privilege service accounts.
##
[​
](#best-practices)
Best practices
###
[​
](#claude-md-configuration)
CLAUDE.md configuration
Create a `CLAUDE.md` file at the repository root to define coding standards, review criteria, and project-specific rules. Claude reads this file during runs and follows your conventions when proposing changes.
###
[​
](#security-considerations)
Security considerations
**Never commit API keys or cloud credentials to your repository**. Always use GitLab CI/CD variables:
* Add `ANTHROPIC\_API\_KEY` as a masked variable (and protect it if needed)
* Use provider-specific OIDC where possible (no long-lived keys)
* Limit job permissions and network egress
* Review Claude’s MRs like any other contributor
###
[​
](#optimizing-performance)
Optimizing performance
* Keep `CLAUDE.md` focused and concise
* Provide clear issue/MR descriptions to reduce iterations
* Configure sensible job timeouts to avoid runaway runs
* Cache npm and package installs in runners where possible
###
[​
](#ci-costs)
CI costs
When using Claude Code with GitLab CI/CD, be aware of associated costs:
* **GitLab Runner time**:
* Claude runs on your GitLab runners and consumes compute minutes
* See your GitLab plan’s runner billing for details
* **API costs**:
* Each Claude interaction consumes tokens based on prompt and response size
* Token usage varies by task complexity and codebase size
* See [Anthropic pricing](https://platform.claude.com/docs/en/about-claude/pricing) for details
* **Cost optimization tips**:
* Use specific `@claude` commands to reduce unnecessary turns
* Set appropriate `max\_turns` and job timeout values
* Limit concurrency to control parallel runs
##
[​
](#security-and-governance)
Security and governance
* Each job runs in an isolated container with restricted network access
* Claude’s changes flow through MRs so reviewers see every diff
* Branch protection and approval rules apply to AI-generated code
* Claude Code uses workspace-scoped permissions to constrain writes
* Costs remain under your control because you bring your own provider credentials
##
[​
](#troubleshooting)
Troubleshooting
###
[​
](#claude-not-responding-to-@claude-commands)
Claude not responding to @claude commands
* Verify your pipeline is being triggered (manually, MR event, or via a note event listener/webhook)
* Ensure CI/CD variables (`ANTHROPIC\_API\_KEY` or cloud provider settings) are present and unmasked
* Check that the comment contains `@claude` (not `/claude`) and that your mention trigger is configured
###
[​
](#job-can’t-write-comments-or-open-mrs)
Job can’t write comments or open MRs
* Ensure `CI\_JOB\_TOKEN` has sufficient permissions for the project, or use a Project Access Token with `api` scope
* Check the `mcp\_\_gitlab` tool is enabled in `--allowedTools`
* Confirm the job runs in the context of the MR or has enough context via `AI\_FLOW\_\*` variables
###
[​
](#authentication-errors)
Authentication errors
* **For Claude API**: Confirm `ANTHROPIC\_API\_KEY` is valid and unexpired
* **For Bedrock/Vertex**: Verify OIDC/WIF configuration, role impersonation, and secret names; confirm region and model availability
##
[​
](#advanced-configuration)
Advanced configuration
###
[​
](#common-parameters-and-variables)
Common parameters and variables
Claude Code supports these commonly used inputs:
* `prompt` / `prompt\_file`: Provide instructions inline (`-p`) or via a file
* `max\_turns`: Limit the number of back-and-forth iterations
* `timeout\_minutes`: Limit total execution time
* `ANTHROPIC\_API\_KEY`: Required for the Claude API (not used for Bedrock/Vertex)
* Provider-specific environment: `AWS\_REGION`, project/region vars for Vertex
Exact flags and parameters may vary by version of `@anthropic-ai/claude-code`. Run `claude --help` in your job to see supported options.
###
[​
](#customizing-claude’s-behavior)
Customizing Claude’s behavior
You can guide Claude in two primary ways:
1. **CLAUDE.md**: Define coding standards, security requirements, and project conventions. Claude reads this during runs and follows your rules.
2. **Custom prompts**: Pass task-specific instructions via `prompt`/`prompt\_file` in the job. Use different prompts for different jobs (for example, review, implement, refactor).
⌘I