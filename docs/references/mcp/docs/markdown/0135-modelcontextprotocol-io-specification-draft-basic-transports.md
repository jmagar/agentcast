Transports - Model Context Protocol
## > Documentation Index
> Fetch the complete documentation index at:
[> https://modelcontextprotocol.io/llms.txt
](https://modelcontextprotocol.io/llms.txt)
> Use this file to discover all available pages before exploring further.
MCP uses JSON-RPC to encode messages. JSON-RPC messages **MUST** be UTF-8 encoded.
The protocol currently defines two standard transport mechanisms for client-server
communication:
1. [stdio](#stdio), communication over standard in and standard out
2. [Streamable HTTP](#streamable-http)
Clients **SHOULD** support stdio whenever possible.
It is also possible for clients and servers to implement
[custom transports](#custom-transports) in a pluggable fashion.
##
[​
](#stdio)
stdio
In the **stdio** transport, the client launches the MCP server as a subprocess.
The two ends communicate over the subprocess’s standard streams:
* The server reads JSON-RPC messages from `stdin` and writes JSON-RPC messages to
`stdout`.
* Messages are individual JSON-RPC requests, notifications, or responses.
* Messages are delimited by newlines, and **MUST NOT** contain embedded newlines.
* The server **MAY** write UTF-8 strings to `stderr` for any logging purposes
including informational, debug, and error messages.
* The client **MAY** capture, forward, or ignore the server’s `stderr` output and
**SHOULD NOT** assume `stderr` output indicates error conditions.
* The server **MUST NOT** write anything to its `stdout` that is not a valid MCP
message.
* The client **MUST NOT** write anything to the server’s `stdin` that is not a
valid MCP message.
###
[​
](#sending-messages)
Sending Messages
The client sends messages by writing JSON-RPC requests, notifications, or
responses to the server’s `stdin`, one message per line.
###
[​
](#receiving-messages)
Receiving Messages
All server-to-client messages — responses to client requests, in-flight
notifications (`notifications/progress`, `notifications/message`), and
deliveries on a [`subscriptions/listen`](/specification/draft/basic/utilities/subscriptions)
stream — arrive on `stdout`, one message per line, multiplexed onto a single
shared channel.
To distinguish notifications belonging to different concurrent subscriptions,
clients **MUST** correlate notifications using the
`io.modelcontextprotocol/subscriptionId` field carried in `\_meta`. See the
schema for [`SubscriptionsListenRequest`](/specification/draft/schema#subscriptionslistenrequest)
for details.
###
[​
](#request-metadata)
Request Metadata
All request metadata for the stdio transport is carried inline in the
JSON-RPC message body. The protocol version, client identity, and
per-request capabilities live in
[`\_meta.io.modelcontextprotocol/\*`](/specification/draft/basic/index#meta);
the method name and arguments live where JSON-RPC puts them. There is no
header layer.
###
[​
](#cancellation)
Cancellation
To cancel an in-flight request, the client **MUST** send a
`notifications/cancelled` notification referencing the request’s ID. Because
stdio is a single shared bidirectional channel, there is no per-request stream
to close. Servers **SHOULD** stop work on a cancelled request as soon as
practical and **MUST NOT** send any further messages for it. See
[Cancellation](/specification/draft/basic/utilities/cancellation) for the full rules.
###
[​
](#shutdown)
Shutdown
The client **SHOULD** initiate shutdown by:
1. Closing the input stream to the child process (the server).
2. Waiting for the server to exit.
3. If the server does not exit within a reasonable time, forcibly terminating
the process using the mechanism appropriate for the operating system.
On POSIX systems, forced termination typically escalates from
[`SIGTERM`](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/signal.h.html)
to `SIGKILL`. On Windows, where POSIX signals are not available, clients can
use [`TerminateProcess`](https://learn.microsoft.com/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminateprocess)
or [Job Objects](https://learn.microsoft.com/windows/win32/procthread/job-objects).
Servers **SHOULD** exit promptly when their standard input is closed or reads
return end-of-file. This is the primary graceful-shutdown signal and the only
portable one, so honoring it reduces the need for forced termination.
The server **MAY** initiate shutdown by closing its output stream to the
client and exiting.
###
[​
](#unexpected-termination)
Unexpected Termination
If the server process exits unexpectedly, the client **SHOULD** restart it.
Because the protocol is stateless, any in-flight requests are simply lost and
the client can retry them against the fresh process. Active
[`subscriptions/listen`](/specification/draft/basic/utilities/subscriptions) streams must also be
re-established after restart.
###
[​
](#backward-compatibility)
Backward Compatibility
A client that supports both modern (per-request-metadata) MCP versions and a
legacy version that requires an `initialize` handshake **SHOULD** probe with
[`server/discover`](/specification/draft/schema#discoverrequest) before sending
any other request. If the server returns `Method not found` (`-32601`), the
client falls back to the legacy `initialize` handshake. If the server returns
`UnsupportedProtocolVersionError`, it speaks a version of MCP without
`initialize` — the client **SHOULD** retry using one of the advertised
`supportedVersions` rather than falling back to `initialize`. See
[Lifecycle: Backward Compatibility](/specification/draft/basic/lifecycle#backward-compatibility-with-initialization-based-versions)
for details.
A client that only supports modern versions does not need to probe.
##
[​
](#streamable-http)
Streamable HTTP
This replaces the [HTTP+SSE transport](/specification/2024-11-05/basic/transports#http-with-sse) from
protocol version 2024-11-05. See [Backward Compatibility](#backward-compatibility-1)
below.
In the **Streamable HTTP** transport, the server operates as an independent
process that can handle multiple client connections. The transport uses HTTP
POST. The server can optionally use
[Server-Sent Events](https://en.wikipedia.org/wiki/Server-sent_events) (SSE)
to stream multiple server messages in response to a single request.
The server **MUST** provide a single HTTP endpoint path (hereafter referred to
as the **MCP endpoint**) that supports POST. For example, this could be a URL
like `https://example.com/mcp`.
###
[​
](#security-&amp;-endpoint)
Security & Endpoint
When implementing Streamable HTTP transport:
1. Servers **MUST** validate the `Origin` header on all incoming connections
to prevent DNS rebinding attacks.
* If the `Origin` header is present and invalid, servers **MUST** respond
with HTTP 403 Forbidden. The HTTP response body **MAY** comprise a
JSON-RPC *error response* that has no `id`.
* When running locally, servers **SHOULD** bind only to localhost
(127.0.0.1) rather than all network interfaces (0.0.0.0).
* Servers **SHOULD** implement proper authentication for all connections.
Without these protections, attackers could use DNS rebinding to interact with
local MCP servers from remote websites.
###
[​
](#sending-messages-2)
Sending Messages
Every JSON-RPC message sent from the client **MUST** be a new HTTP POST
request to the MCP endpoint.
1. The client **MUST** use HTTP POST to send JSON-RPC messages.
2. The client **MUST** include an `Accept` header listing both
`application/json` and `text/event-stream` as supported content types.
3. The client **MUST** include the [request metadata headers](#request-metadata-1)
on each POST request.
4. The body of the HTTP POST **MUST** be a single JSON-RPC *request*,
*notification*, or *response* to a server-initiated input request (see
[Receiving Messages](#receiving-messages-1)).
5. If the body is a JSON-RPC *notification* or a *response* to a
server-initiated input request:
* If the server accepts it, the server **MUST** return HTTP status code
`202 Accepted` with no body.
* If the server cannot accept it, it **MUST** return an HTTP error status
code (e.g., `400 Bad Request`). The HTTP response body **MAY** comprise
a JSON-RPC *error response* that has no `id`.
* If the body is a JSON-RPC *request*, the server **MUST** return either
`Content-Type: application/json` (a single JSON object) or
`Content-Type: text/event-stream` (an SSE response stream). The client
**MUST** support both.
###
[​
](#receiving-messages-2)
Receiving Messages
When the server returns an SSE response stream
(`Content-Type: text/event-stream`):
* The server **MAY** send JSON-RPC *notifications* — for example,
[`notifications/progress`](/specification/draft/basic/utilities/progress)
or [`notifications/message`](/specification/draft/server/utilities/logging) —
before the final response. These notifications **MUST** relate to the
originating client request.
* The server **MUST NOT** send independent JSON-RPC *requests* on this stream.
Server-to-client interactions (sampling, elicitation, list-roots) are
embedded as input requests inside an
[`IncompleteResult`](/specification/draft/schema#inputrequiredresult) per
[SEP-2322 (MRTR)](/seps/2322-MRTR), not delivered as separate requests on
this or any other stream.
* The final JSON-RPC *response* **SHOULD** terminate the stream.
Long-lived notification streams are obtained by sending a
[`subscriptions/listen`](/specification/draft/basic/utilities/subscriptions)
request. The server’s response is itself an SSE stream that stays open and
delivers `notifications/tools/list\_changed`,
`notifications/resources/updated`, `notifications/message`, etc. for the
notification types the client opted in to.
When initiating an SSE stream, servers **SHOULD** include the
`X-Accel-Buffering: no` header in the HTTP response. This instructs reverse
proxies (such as nginx) to disable response buffering, ensuring that SSE
events are delivered to clients immediately rather than being held in a
buffer. Without this header, proxies may accumulate messages before sending
them to the client, introducing unwanted latency and potentially breaking the
real-time nature of SSE communication.
For workloads that need durability across connection drops, use the
[tasks primitive](/specification/draft/basic/utilities/tasks); resumable SSE
streams via `Last-Event-ID` are not supported.
###
[​
](#cancellation-2)
Cancellation
Closing the SSE response stream **MUST** be treated by the server as
cancellation of that request. Because each request has its own response
stream, the transport-level disconnect is unambiguous. The server **SHOULD**
stop work on the cancelled request as soon as practical and **MUST NOT** send
any further messages for it. See
[Cancellation](/specification/draft/basic/utilities/cancellation) for the full rules.
###
[​
](#request-metadata-2)
Request Metadata
The Streamable HTTP transport mirrors selected JSON-RPC body fields into HTTP
headers so that intermediaries (load balancers, gateways, observability
tooling) can route and inspect requests without parsing the body.
####
[​
](#protocol-version-header)
Protocol Version Header
Every POST request to the MCP endpoint **MUST** include an
`MCP-Protocol-Version` header.
For example: `MCP-Protocol-Version: DRAFT-2026-v1`
The header value **MUST** match the
`io.modelcontextprotocol/protocolVersion` field carried in the request body’s
`\_meta`. If the values do not match, the server **MUST** reject the request
with `400 Bad Request` and a `HeaderMismatch` JSON-RPC error
(see [Server Validation](#server-validation)).
If the server does not implement the requested protocol version (whether the
version is unknown to the server, or is a known version the server has chosen
not to support), it **MUST** respond with `400 Bad Request` and an
[`UnsupportedProtocolVersionError`](/specification/draft/schema#unsupportedprotocolversionerror)
listing its supported versions. See
[Lifecycle: Protocol Version Negotiation](/specification/draft/basic/lifecycle#protocol-version-negotiation)
for the negotiation flow.
If the server does not implement the requested RPC method, it **MUST** respond
with `404 Not Found` and a JSON-RPC error with code `-32601`
(`Method not found`).
For backward compatibility, if the server does *not* receive an
`MCP-Protocol-Version` header and has no other way to identify the version,
the server **SHOULD** assume protocol version `2025-03-26`.
####
[​
](#standard-request-headers)
Standard Request Headers
|Header Name|Source Field|Required For|
|`Mcp-Method`|`method`|All requests and notifications|
|`Mcp-Name`|`params.name` or `params.uri`|`tools/call`, `resources/read`, `prompts/get` requests|
These headers are **REQUIRED** for compliance.
**`tools/call` request:**
```
`POST /mcp HTTP/1.1
Content-Type: application/json
MCP-Protocol-Version: DRAFT-2026-v1
Mcp-Method: tools/call
Mcp-Name: get\_weather
{
"jsonrpc": "2.0",
"id": 1,
"method": "tools/call",
"params": {
"name": "get\_weather",
"arguments": {
"location": "Seattle, WA"
},
"\_meta": {
"io.modelcontextprotocol/protocolVersion": "DRAFT-2026-v1",
"io.modelcontextprotocol/clientInfo": {
"name": "ExampleClient",
"version": "1.0.0"
},
"io.modelcontextprotocol/clientCapabilities": {}
}
}
}
`
```
**`resources/read` request:**
```
`POST /mcp HTTP/1.1
Content-Type: application/json
MCP-Protocol-Version: DRAFT-2026-v1
Mcp-Method: resources/read
Mcp-Name: file:///projects/myapp/config.json
{
"jsonrpc": "2.0",
"id": 2,
"method": "resources/read",
"params": {
"uri": "file:///projects/myapp/config.json",
"\_meta": {
"io.modelcontextprotocol/protocolVersion": "DRAFT-2026-v1",
"io.modelcontextprotocol/clientInfo": {
"name": "ExampleClient",
"version": "1.0.0"
},
"io.modelcontextprotocol/clientCapabilities": {}
}
}
}
`
```
####
[​
](#custom-headers-from-tool-parameters)
Custom Headers from Tool Parameters
MCP servers **MAY** designate specific tool parameters to be mirrored into
HTTP headers using an `x-mcp-header` extension property in the parameter’s
schema within the tool’s `inputSchema`. See
[Tool Definitions](/specification/draft/server/tools#x-mcp-header) for
details on how to annotate tool parameters.
While the use of `x-mcp-header` is optional for servers, clients **MUST**
support this feature. When a server’s tool definition includes
`x-mcp-header` annotations, conforming clients **MUST** mirror the
designated parameter values into HTTP headers.
##### Schema Extension
The `x-mcp-header` property specifies the name portion used to construct
the header name `Mcp-Param-{name}`.
**Constraints on `x-mcp-header` values**:
* **MUST NOT** be empty
* **MUST** contain only ASCII characters (excluding space and `:`)
* **MUST** be case-insensitively unique among all `x-mcp-header` values in
the `inputSchema`
* **MUST** only be applied to parameters with primitive types (number,
string, boolean)
Clients **MUST** reject tool definitions where any `x-mcp-header` value
violates these constraints. Rejection means the client **MUST** exclude the
invalid tool from the result of `tools/list`. Clients **SHOULD** log a
warning when rejecting a tool definition, including the tool name and the
reason for rejection.
**Example tool definition:**
```
`{
"name": "execute\_sql",
"description": "Execute SQL on Google Cloud Spanner",
"inputSchema": {
"type": "object",
"properties": {
"region": {
"type": "string",
"description": "The region to execute the query in",
"x-mcp-header": "Region"
},
"query": {
"type": "string",
"description": "The SQL query to execute"
}
},
"required": ["region", "query"]
}
}
`
```
**Resulting HTTP request:**
```
`POST /mcp HTTP/1.1
Content-Type: application/json
MCP-Protocol-Version: DRAFT-2026-v1
Mcp-Method: tools/call
Mcp-Name: execute\_sql
Mcp-Param-Region: us-west1
{
"jsonrpc": "2.0",
"id": 1,
"method": "tools/call",
"params": {
"name": "execute\_sql",
"arguments": {
"region": "us-west1",
"query": "SELECT \* FROM users"
}
}
}
`
```
##### Value Encoding
Clients **MUST** encode parameter values before including them in HTTP
headers to ensure safe transmission and prevent injection attacks.
**Type conversion**: Convert the parameter value to its string representation:
* `string`: Use the value as-is
* `number`: Convert to decimal string representation (e.g., `42`, `3.14`)
* `boolean`: Convert to lowercase `"true"` or `"false"`
Per [RFC 9110](https://datatracker.ietf.org/doc/html/rfc9110#name-field-values),
HTTP header field values must consist of visible ASCII characters
(0x21-0x7E), space (0x20), and horizontal tab (0x09). When a value cannot
be safely represented as a plain ASCII header value (e.g., it contains
non-ASCII characters, control characters, or has leading/trailing
whitespace), clients **MUST** use Base64 encoding of the UTF-8
representation with the following format:
```
`Mcp-Param-{Name}: =?base64?{Base64EncodedValue}?=
`
```
The prefix `=?base64?` and suffix `?=` indicate that the value is
Base64-encoded. Servers and intermediaries that need to inspect these
values **MUST** decode them accordingly.
**Encoding examples:**
|Original Value|Reason|Encoded Header Value|
|`"us-west1"`|Plain ASCII|`Mcp-Param-Region: us-west1`|
|`"Hello, 世界"`|Contains non-ASCII|`Mcp-Param-Greeting: =?base64?SGVsbG8sIOS4lueVjA==?=`|
|`" padded "`|Leading/trailing spaces|`Mcp-Param-Text: =?base64?IHBhZGRlZCA=?=`|
|`"line1\\nline2"`|Contains newline|`Mcp-Param-Text: =?base64?bGluZTEKbGluZTI=?=`|
##### Client Behavior
When constructing a `tools/call` request via HTTP transport, the client
**MUST**:
1. Extract the values for any standard headers from the request body (e.g.,
`method`, `params.name`, `params.uri`).
2. Append the `Mcp-Method` header and, if applicable, `Mcp-Name` header to
the request.
3. Inspect the tool’s `inputSchema` for properties marked with
`x-mcp-header` and extract the value for each parameter.
4. Encode the values according to the [Value Encoding](#value-encoding)
rules.
5. Append a `Mcp-Param-{Name}: {Value}` header to the request.
##### Server Behavior for Custom Headers
Intermediate servers that do not recognize an `Mcp-Param-{Name}` header
**MUST** forward it and otherwise ignore it, as required by the
[HTTP Semantics RFC](https://www.rfc-editor.org/rfc/rfc9110.html#name-field-names).
Servers **MUST** reject requests with a recognized `Mcp-Param-{Name}` header
that contains invalid characters (see [Value Encoding](#value-encoding)).
Any server that processes the message body **MUST** validate that encoded
header values, after decoding if Base64-encoded, match the corresponding
values in the request body. Servers **MUST** reject requests with a
`400 Bad Request` HTTP status and JSON-RPC error code `-32001`
(`HeaderMismatch`) if any validation fails.
|Scenario|Client Behavior|Server Behavior|
|Parameter value provided|Client MUST include the header|Server MUST validate header matches body|
|Parameter value is `null`|Client MUST omit the header|Server MUST NOT expect the header|
|Parameter not in arguments|Client MUST omit the header|Server MUST NOT expect the header|
|Client omits header but value is in body|Non-conforming client|Server MUST reject the request|
####
[​
](#case-sensitivity)
Case Sensitivity
Header names (called “field names” in
[RFC 9110](https://datatracker.ietf.org/doc/html/rfc9110#name-field-names))
are case-insensitive. Clients and servers **MUST** use case-insensitive
comparisons for header names. Header *values* (such as method names) are
case-sensitive.
####
[​
](#server-validation)
Server Validation
Servers that process the request body **MUST** reject requests where the
values specified in the headers do not match the corresponding values in the
request body. This prevents potential security vulnerabilities when
different components in the network rely on different sources of truth
(e.g., a load balancer routing on the header value while the MCP server
executes based on the body value).
When rejecting a request due to header validation failure, servers **MUST**
return HTTP status `400 Bad Request` and **SHOULD** include a JSON-RPC error
response using the following error code:
|Code|Name|Description|
|`-32001`|`HeaderMismatch`|The HTTP headers do not match the corresponding values in the request body, or required headers are missing/malformed.|
This error code is in the JSON-RPC implementation-defined server error range
(`-32000` to `-32099`).
**Example error response:**
```
`{
"jsonrpc": "2.0",
"id": 1,
"error": {
"code": -32001,
"message": "Header mismatch: Mcp-Name header value 'foo' does not match body value 'bar'"
}
}
`
```
Validation failure conditions include:
* A required standard header (`MCP-Protocol-Version`, `Mcp-Method`,
`Mcp-Name`) is missing.
* A header value does not match the corresponding request body value.
* A header value contains invalid characters.
Intermediaries **MUST** return an appropriate HTTP error status (e.g.,
`400 Bad Request`) for validation failures but are not required to return
a JSON-RPC error response.
###
[​
](#backward-compatibility-2)
Backward Compatibility
A client that supports both modern (per-request-metadata) MCP versions and a
legacy version that requires an `initialize` handshake **MAY** detect which
era the server implements by attempting a modern request first. If the
server returns `400 Bad Request` (or any other version error indicating the
server does not implement the modern protocol), the client falls back to
`initialize` and continues with the legacy version for subsequent requests.
See [Lifecycle: Backward Compatibility](/specification/draft/basic/lifecycle#backward-compatibility-with-initialization-based-versions)
for details.
Separately, clients and servers can maintain backward compatibility with the
deprecated [HTTP+SSE transport](/specification/2024-11-05/basic/transports#http-with-sse) (from
protocol version 2024-11-05) as follows:
**Servers** wanting to support older clients should:
* Continue to host both the SSE and POST endpoints of the old transport,
alongside the new “MCP endpoint” defined for the Streamable HTTP transport.
* It is also possible to combine the old POST endpoint and the new MCP
endpoint, but this may introduce unneeded complexity.
**Clients** wanting to support older servers should:
1. Accept an MCP server URL from the user, which may point to either a server
using the old transport or the new transport.
2. Attempt to POST a request to the server URL, with an `Accept` header as
defined above:
* If it succeeds, the client can assume this is a server supporting the
new Streamable HTTP transport.
* If it fails with HTTP status codes “400 Bad Request”, “404 Not Found”,
or “405 Method Not Allowed”:
* Issue a GET request to the server URL, expecting that this will open
an SSE stream and return an `endpoint` event as the first event.
* When the `endpoint` event arrives, the client can assume this is a
server running the old HTTP+SSE transport, and should use that
transport for all subsequent communication.
##
[​
](#custom-transports)
Custom Transports
Clients and servers **MAY** implement additional custom transport mechanisms
to suit their specific needs. The protocol is transport-agnostic and can be
implemented over any communication channel that supports bidirectional
message exchange.
Implementers who choose to support custom transports **MUST** ensure they
preserve the JSON-RPC message format. Custom transports **SHOULD** document
their specific connection establishment and message exchange patterns to aid
interoperability.