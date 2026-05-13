Claude Code on Claude Platform on AWS - Claude Code Docs
## > Documentation Index
> Fetch the complete documentation index at:
[> https://code.claude.com/docs/llms.txt
](https://code.claude.com/docs/llms.txt)
> Use this file to discover all available pages before exploring further.
Claude Platform on AWS is the Anthropic-operated Claude API with AWS authentication, IAM access control, and AWS Marketplace billing. Requests reach Anthropic’s API directly, so you get the same models and features as the [Claude API](https://platform.claude.com/docs) on the same release schedule. You authenticate with AWS credentials or a workspace API key, and you pay through AWS Marketplace.
Use this guide to point Claude Code at a workspace you’ve already provisioned through Claude Platform on AWS. For the AWS subscription and workspace setup that comes before this, see the [Claude Platform on AWS documentation](https://platform.claude.com/docs/en/build-with-claude/claude-platform-on-aws).
Subscribing through AWS Marketplace provisions a new Anthropic organization tied to your AWS account. This organization is separate from any organization you already have with Anthropic, and credentials don’t transfer between them. Use the workspace ID and API keys from the AWS-linked organization, not from a pre-existing Claude Console account.
##
[​
](#prerequisites)
Prerequisites
Before configuring Claude Code, you need:
* An active Claude Platform on AWS subscription through AWS Marketplace
* A workspace in your AWS-linked Anthropic organization, with its workspace ID
* An IAM principal with permission to invoke the Anthropic service, or an API key scoped to the workspace
* AWS credentials in your environment, in `\~/.aws/credentials`, or from an attached IAM role if you want SigV4 authentication. The AWS CLI is required only for the SSO login flow.
##
[​
](#setup)
Setup
###
[​
](#1-configure-aws-credentials)
1. Configure AWS credentials
Claude Code supports two authentication methods for Claude Platform on AWS. Choose the method that fits how your team manages access.
**Option A: AWS credentials with SigV4**
Claude Code signs requests with SigV4 using the standard AWS credential chain: environment variables, shared credentials in `\~/.aws/credentials`, IAM roles, AWS SSO sessions, and any other sources the AWS SDK supports.
For local use, log in with the AWS CLI before starting Claude Code. The example below uses an SSO profile, but any method that produces credentials in the standard locations works.
```
`aws sso login --profile my-profile
export AWS\_PROFILE=my-profile
`
```
For CI and automation, give the runner an IAM role with permission to invoke the Anthropic service and set `AWS\_REGION`. The credential chain picks the role up automatically.
If your SSO credentials expire mid-session, configure [`awsAuthRefresh`](/docs/en/amazon-bedrock#advanced-credential-configuration) so Claude Code re-runs your login command and retries instead of failing. Add the command to your `settings.json`:
```
`{
"awsAuthRefresh": "aws sso login --profile my-profile"
}
`
```
**Option B: Workspace API key**
A workspace API key is a long-lived secret, useful when you don’t want to manage federated AWS credentials. Generate one in the AWS Console under **Claude Platform on AWS → API keys** and set it as `ANTHROPIC\_AWS\_API\_KEY`:
```
`export ANTHROPIC\_AWS\_API\_KEY=sk-ant-xxxxx
`
```
The key is sent as `x-api-key` and takes precedence over SigV4, so any AWS credentials in your environment are ignored. API keys from a separate Claude Console organization won’t work here.
Treat workspace API keys like any other production credential. The [user settings file](/docs/en/settings) `env` block is a convenient way to scope the key to your machine without exporting it globally.
The `/login` and `/logout` commands don’t change Claude Platform on AWS authentication. Authentication runs through your AWS credentials or workspace API key, not through a Claude.ai subscription.
###
[​
](#2-configure-claude-code)
2. Configure Claude Code
Set the environment variables that route Claude Code through Claude Platform on AWS instead of the default Anthropic API.
```
`export CLAUDE\_CODE\_USE\_ANTHROPIC\_AWS=1
export ANTHROPIC\_AWS\_WORKSPACE\_ID=wrkspc\_01ABCDEFGHIJKLMN
export AWS\_REGION=us-east-1
`
```
`ANTHROPIC\_AWS\_WORKSPACE\_ID` is required and is sent on every request as the `anthropic-workspace-id` header. The base URL is computed from `AWS\_REGION` as `https://aws-external-anthropic.{region}.api.aws`. To override the URL directly, set `ANTHROPIC\_AWS\_BASE\_URL`.
Claude Platform on AWS is opt-in even when AWS credentials are present in your environment. Bedrock and Foundry take precedence in provider routing, so unset `CLAUDE\_CODE\_USE\_BEDROCK` and `CLAUDE\_CODE\_USE\_FOUNDRY` if they’re set.
###
[​
](#3-pin-model-versions)
3. Pin model versions
Claude Platform on AWS uses the same model IDs as the direct Claude API. The default aliases `opus`, `sonnet`, and `haiku` resolve to the latest versions available in your workspace.
If you deploy Claude Code to a team, pin the model IDs explicitly so a new release doesn’t move everyone at once:
```
`export ANTHROPIC\_DEFAULT\_OPUS\_MODEL=claude-opus-4-7
export ANTHROPIC\_DEFAULT\_SONNET\_MODEL=claude-sonnet-4-6
export ANTHROPIC\_DEFAULT\_HAIKU\_MODEL=claude-haiku-4-5
`
```
For the full list of model IDs and aliases, see [Models overview](https://platform.claude.com/docs/en/about-claude/models/overview). For other model-related variables, see [Model configuration](/docs/en/model-config).
[Prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching) is enabled automatically. 1-hour cache writes are billed at a higher rate than 5-minute writes. To request a 1-hour cache TTL instead of the 5-minute default, set `ENABLE\_PROMPT\_CACHING\_1H=1`.
##
[​
](#use-the-agent-sdk)
Use the Agent SDK
The [Agent SDK](/docs/en/agent-sdk/overview) reads the same environment variables as the CLI, so any program that spawns the Claude Code subprocess can target Claude Platform on AWS by exporting `CLAUDE\_CODE\_USE\_ANTHROPIC\_AWS`, `ANTHROPIC\_AWS\_WORKSPACE\_ID`, and either `ANTHROPIC\_AWS\_API\_KEY` or AWS credentials before the call.
```
`import { query } from "@anthropic-ai/claude-agent-sdk";
process.env.CLAUDE\_CODE\_USE\_ANTHROPIC\_AWS = "1";
process.env.ANTHROPIC\_AWS\_WORKSPACE\_ID = "wrkspc\_01ABCDEFGHIJKLMN";
process.env.AWS\_REGION = "us-east-1";
for await (const msg of query({ prompt: "What's in this repo?" })) {
console.log(msg);
}
`
```
This example relies on the ambient AWS credential chain for SigV4. To authenticate with a workspace API key instead, set `ANTHROPIC\_AWS\_API\_KEY` the same way. For the broader Agent SDK surface, see [Agent SDK overview](/docs/en/agent-sdk/overview).
##
[​
](#route-through-a-corporate-proxy)
Route through a corporate proxy
To route traffic through a proxy or [LLM gateway](/docs/en/llm-gateway), set `ANTHROPIC\_AWS\_BASE\_URL` to the proxy’s address. Claude Code sends requests to that URL with the same workspace and authentication headers, so any gateway that forwards them unchanged works.
```
`export CLAUDE\_CODE\_USE\_ANTHROPIC\_AWS=1
export ANTHROPIC\_AWS\_WORKSPACE\_ID=wrkspc\_01ABCDEFGHIJKLMN
export ANTHROPIC\_AWS\_BASE\_URL=https://anthropic-proxy.example.com
`
```
If your gateway signs requests itself, set `CLAUDE\_CODE\_SKIP\_ANTHROPIC\_AWS\_AUTH=1` so Claude Code sends unsigned requests and lets the gateway add SigV4 headers before forwarding to AWS. If the gateway requires its own token, set it in `ANTHROPIC\_AUTH\_TOKEN`.
```
`export CLAUDE\_CODE\_USE\_ANTHROPIC\_AWS=1
export CLAUDE\_CODE\_SKIP\_ANTHROPIC\_AWS\_AUTH=1
export ANTHROPIC\_AWS\_WORKSPACE\_ID=wrkspc\_01ABCDEFGHIJKLMN
export ANTHROPIC\_AWS\_BASE\_URL=https://anthropic-proxy.example.com
`
```
##
[​
](#troubleshooting)
Troubleshooting
Run `/status` to see the resolved provider and any explicitly configured workspace ID, region, base URL override, and auth-skip setting. This is the fastest way to confirm Claude Code is targeting Claude Platform on AWS at all.
###
[​
](#403-forbidden-or-accessdenied-on-every-request)
`403 Forbidden` or `AccessDenied` on every request
The IAM principal Claude Code resolved likely lacks permission to invoke the Anthropic service in your workspace. Check the role attached to your AWS profile or the runner that started Claude Code, and verify it has the `aws-external-anthropic` actions documented in the [IAM action reference](https://platform.claude.com/docs/en/api/claude-platform-on-aws-iam-actions).
If you set `ANTHROPIC\_AWS\_API\_KEY`, the key takes precedence over SigV4 and a stale key produces the same error. Regenerate the key in the AWS Console under **Claude Platform on AWS → API keys** or unset the variable to fall back to your AWS credentials.
###
[​
](#requests-fail-with-a-missing-workspace-error)
Requests fail with a missing-workspace error
`ANTHROPIC\_AWS\_WORKSPACE\_ID` is likely unset or empty. Every Claude Platform on AWS request must include the workspace ID. It is not implied by your AWS credentials. Find the ID under **Workspaces** on the AWS Console service page and export it before starting Claude Code.
###
[​
](#requests-still-go-to-api-anthropic-com)
Requests still go to `api.anthropic.com`
`CLAUDE\_CODE\_USE\_ANTHROPIC\_AWS` is likely unset or set to a value that doesn’t parse as truthy. Set it to `1` and run `/status` to confirm the resolved provider. If `CLAUDE\_CODE\_USE\_BEDROCK` or `CLAUDE\_CODE\_USE\_FOUNDRY` is also set, those take precedence over Claude Platform on AWS.
##
[​
](#additional-resources)
Additional resources
The Claude Platform on AWS subscription, workspace, and IAM setup that comes before configuring Claude Code is covered in the platform documentation:
* [Claude Platform on AWS overview](https://platform.claude.com/docs/en/build-with-claude/claude-platform-on-aws): subscription, workspace setup, and product reference
* [IAM action reference](https://platform.claude.com/docs/en/api/claude-platform-on-aws-iam-actions): permissions and managed policies
⌘I