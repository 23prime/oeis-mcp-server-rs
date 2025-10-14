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

test("list tools", async () => {
  const response = await client.listTools();
  printObject(response);
  expect(response.tools).toHaveLength(2);
});

test("list prompts", async () => {
  const response = await client.listPrompts();
  printObject(response);
  expect(response.prompts).toHaveLength(0);
});

test("list resource templates", async () => {
  const response = await client.listResourceTemplates();
  printObject(response);
  expect(response.resourceTemplates).toHaveLength(1);
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

test("Resource(oeis://sequence/{id})", async () => {
  const response = await client.readResource({ uri: "oeis://sequence/A000045" });

  expect(response.contents).toHaveLength(1);

  const content = response.contents[0];
  expect(content.uri).toBe("oeis://sequence/A000045");
  expect(content.mimeType).toBe("text");
  expect(content.text).toBeDefined();
});
