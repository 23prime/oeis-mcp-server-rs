import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StreamableHTTPClientTransport } from "@modelcontextprotocol/sdk/client/streamableHttp.js";
import { expect, test } from "vitest";

const port = 8000;
const baseUrl = new URL(`http://localhost:${port}/mcp`);
const transport = new StreamableHTTPClientTransport(new URL(baseUrl));

const client = new Client({
  name: "test-client",
  version: "1.0.0",
});

await client.connect(transport);

const printObject = (obj) => {
  console.dir(obj, { depth: null });
};

test("server version", async () => {
  const version = await client.getServerVersion();
  printObject(version);
  expect(version.name).toBe("rmcp");
  expect(version.version).toBe("0.8.3");
});

test("server capabilities", async () => {
  const capabilities = await client.getServerCapabilities();
  printObject(capabilities);
  expect(capabilities.prompts).toBeDefined();
  expect(capabilities.resources).toBeDefined();
  expect(capabilities.tools).toBeDefined();
});

test("server instructions", async () => {
  const instructions = await client.getInstructions();
  printObject(instructions);
  expect(instructions).toContain(
    "This server provides access to the OEIS (Online Encyclopedia of Integer Sequences) database."
  );
});

test("list prompts", async () => {
  const response = await client.listPrompts();
  printObject(response);
  expect(response.prompts).toHaveLength(1);
});

test("Prompt(sequence_analysis)", async () => {
  const response = await client.getPrompt({
    name: "sequence_analysis",
    arguments: {
      sequence_id: "A000045",
    },
  });

  expect(response.messages).toHaveLength(2);

  const userMessage = response.messages.find((msg) => msg.role === "user");
  expect(userMessage).toBeDefined();
  expect(userMessage.content.type).toBe("text");
  expect(userMessage.content.text).toBeDefined();

  const assistantMessage = response.messages.find((msg) => msg.role === "assistant");
  expect(assistantMessage).toBeDefined();
  expect(assistantMessage.content.type).toBe("text");
  expect(assistantMessage.content.text).toBeDefined();
});

test("list tools", async () => {
  const response = await client.listTools();
  printObject(response);
  expect(response.tools).toHaveLength(2);
});

test("Tool(get_url)", async () => {
  const response = await client.callTool({ name: "get_url" });

  expect(response.isError).toBe(false);
  expect(response.content).toHaveLength(1);

  const content = response.content[0];
  expect(content.type).toBe("text");
  expect(content.text).toBeDefined();
});

test("Tool(find_by_id)", async () => {
  const response = await client.callTool({ name: "find_by_id", arguments: { id: "A000045" } });

  expect(response.isError).toBe(false);
  expect(response.content).toHaveLength(1);

  const content = response.content[0];
  expect(content.type).toBe("text");
  expect(content.text).toBeDefined();

  expect(response.structuredContent).toBeDefined();
});

test("list resource templates", async () => {
  const response = await client.listResourceTemplates();
  printObject(response);
  expect(response.resourceTemplates).toHaveLength(1);
});

test("Resource(oeis://sequence/{id})", async () => {
  const response = await client.readResource({ uri: "oeis://sequence/A000045" });

  expect(response.contents).toHaveLength(1);

  const content = response.contents[0];
  expect(content.uri).toBe("oeis://sequence/A000045");
  expect(content.mimeType).toBe("text");
  expect(content.text).toBeDefined();
});
