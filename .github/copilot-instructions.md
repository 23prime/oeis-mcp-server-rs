# GitHub Copilot Instructions

This file provides guidance to GitHub Copilot when generating code for this repository.

## Project Context

This is a Rust-based MCP (Model Context Protocol) server for OEIS (Online Encyclopedia of Integer Sequences) lookups. It uses the `rmcp` library to expose OEIS functionality via HTTP.

## Code Style and Conventions

### Language Guidelines

- Write all code comments, documentation, and variable names in English
- Use English for commit messages
- Adapt explanations to the user's language (Japanese/English) while keeping code in English

### General Rust Guidelines

- Follow standard Rust formatting: use `cargo fmt` before committing
- Run `cargo clippy` to catch common mistakes and anti-patterns
- Prefer `task rs:check` for comprehensive validation (runs fix, lint, and test)
- Use descriptive variable names and add comments for complex logic
- Leverage Rust's type system for safety

### Async/Await Patterns

- All MCP handlers are async functions
- Use `tokio` runtime for async operations
- HTTP client operations use `reqwest` with rustls

### Error Handling

- Map errors to MCP error codes (`INTERNAL_ERROR`, `INVALID_PARAMS`)
- Use `?` operator for error propagation
- Log errors with appropriate context using `tracing`

## MCP Server Patterns

### Tool Definitions

When adding new MCP tools:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ToolNameRequest {
    /// Parameter description
    pub param_name: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ToolNameResponse {
    pub result: String,
}

#[tool_router]
impl<C: OEISClient + Clone + 'static> OEIS<C> {
    #[tool(description = "Description for the tool")]
    pub async fn tool_name(&self, request: Parameters<ToolNameRequest>) -> Result<CallToolResult, McpError> {
        // Implementation
    }
}
```

### Prompt Definitions

When adding new MCP prompts:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PromptNameRequest {
    /// Argument description
    pub arg_name: String,
}

#[prompt_router]
impl<C: OEISClient + Clone + 'static> OEIS<C> {
    #[prompt(description = "Description for the prompt")]
    pub async fn prompt_name(&self, request: Parameters<PromptNameRequest>) -> Result<Vec<PromptMessage>, McpError> {
        // Return conversation-style messages
    }
}
```

### Resource Patterns

Resources follow the URI pattern `oeis://sequence/{id}`:

- Implement `list_resource_templates` for resource discovery
- Implement `read_resource` for resource content retrieval
- Return JSON with appropriate MIME types

## Testing Patterns

### Unit Tests

- Place tests in `#[cfg(test)]` modules within the same file
- Use `tokio::test` for async tests
- Mock external HTTP calls with `httpmock`

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_function_name() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/search");
            then.status(200).json_body(json!({"results": []}));
        });

        // Test implementation
    }
}
```

## Common Patterns

### HTTP Client Usage

- Use `OEISClient` for all OEIS API interactions
- Client is initialized with base URL
- Methods should be async and return `Result<T, reqwest::Error>`

### Logging

- Use `tracing` macros: `debug!`, `info!`, `warn!`, `error!`
- Include relevant context in log messages
- Set log level via `RUST_LOG` environment variable

## Project Structure

```
src/
├── main.rs           # HTTP server setup and MCP binding
├── oeis.rs           # MCP tool, prompt, and resource definitions
├── oeis_client.rs    # OEIS API HTTP client
└── tracer.rs         # Tracing configuration
```

## Dependencies

Key crates used:

- `rmcp`: MCP server framework with procedural macros
- `axum`: HTTP server framework
- `tokio`: Async runtime
- `reqwest`: HTTP client
- `anyhow`: Error handling
- `serde`, `serde_json`: Serialization
- `schemars`: JSON Schema generation
- `tracing`: Structured logging

## Quick Commands

- Run dev server: `task rs:dev`
- Run tests: `task rs:test` or `cargo test -- --nocapture`
- Comprehensive check: `task rs:check`
- Format: `task rs:fmt`
- Lint: `task rs:lint`

## Best Practices

1. Always derive `JsonSchema` for MCP request/response types
2. Use descriptive names for tools and prompts
3. Provide clear descriptions in `#[tool]` and `#[prompt]` attributes
4. Add unit tests for new functionality
5. Use the `?` operator for clean error propagation
6. Leverage Rust's ownership system to avoid clones when possible
7. Document public APIs with doc comments (`///`)
