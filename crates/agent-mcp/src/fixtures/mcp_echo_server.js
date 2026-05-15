#!/usr/bin/env node

const readline = require("node:readline");

const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });

function send(message) {
  process.stdout.write(`${JSON.stringify(message)}\n`);
}

function result(id, value) {
  send({ jsonrpc: "2.0", id, result: value });
}

function error(id, code, message) {
  send({ jsonrpc: "2.0", id, error: { code, message } });
}

rl.on("line", (line) => {
  if (!line.trim()) return;
  const request = JSON.parse(line);
  if (request.id === undefined) return;

  switch (request.method) {
    case "initialize":
      result(request.id, {
        protocolVersion: "2025-11-25",
        capabilities: {
          tools: {},
          resources: {},
          prompts: {}
        },
        serverInfo: {
          name: "agentcast-fixture",
          version: "0.1.0"
        }
      });
      break;
    case "tools/list":
      result(request.id, {
        tools: [{
          name: "echo",
          title: "Echo",
          description: "Return input",
          inputSchema: {
            type: "object",
            properties: { message: { type: "string" } },
            required: ["message"]
          }
        }]
      });
      break;
    case "tools/call": {
      const message = request.params?.arguments?.message ?? "";
      result(request.id, {
        content: [{ type: "text", text: message }],
        isError: false
      });
      break;
    }
    case "resources/list":
      result(request.id, {
        resources: [{
          uri: "fixture://echo",
          name: "fixture",
          title: "Fixture",
          description: "Fixture resource",
          mimeType: "text/plain"
        }]
      });
      break;
    case "resources/templates/list":
      result(request.id, {
        resourceTemplates: [{
          uriTemplate: "fixture://{name}",
          name: "fixture-template",
          title: "Fixture Template",
          description: "Fixture resource template",
          mimeType: "text/plain"
        }]
      });
      break;
    case "resources/read":
      result(request.id, {
        contents: [{
          uri: request.params.uri,
          mimeType: "text/plain",
          text: "fixture resource"
        }]
      });
      break;
    case "prompts/list":
      result(request.id, {
        prompts: [{
          name: "summarize",
          title: "Summarize",
          description: "Summarize a topic",
          arguments: [{ name: "topic", required: true }]
        }]
      });
      break;
    case "prompts/get": {
      const topic = request.params?.arguments?.topic ?? "topic";
      result(request.id, {
        description: "Summarize a topic",
        messages: [{
          role: "user",
          content: { type: "text", text: `Summarize ${topic}` }
        }]
      });
      break;
    }
    default:
      error(request.id, -32601, `method not found: ${request.method}`);
  }
});
