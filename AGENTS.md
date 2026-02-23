# AGENTS.md

This file provides guidance to AI coding agents when working with code in this repository.

## General agent rules

- When users ask questions, answer them instead of doing the work.

### Shell Rules

- Always use `rm -f` (never bare `rm`)
- Before running a series of `git` commands, confirm you are in the project root; if not, `cd` there first. Then run all subsequent `git` commands from that directory without the `-C` option.

## Project Overview

This is a Rust implementation of an OEIS (Online Encyclopedia of Integer Sequences) MCP (Model Context Protocol) server. It exposes OEIS sequence lookup functionality through an HTTP-based MCP server using the `rmcp` library.

## Development Commands

The project uses mise for task management. Key commands:

- **Setup**: `mise run setup` - Sets up mise and installs tools
- **Run development server**: `mise run rs-dev` - Runs with hot-reloading using watchexec
- **Build**: `cargo build` or `mise run rs-build`
- **Build release**: `cargo build --release` or `mise run rs-build-release`
- **Auto-fix and format**: `mise run rs-fix` (runs clippy fix and cargo fmt)
- **Check Rust code**: `mise run rs-check` - Runs clippy, fmt check, and tests (ALWAYS use this for comprehensive Rust checks)
- **Fix all**: `mise run fix` - Runs all fix tasks (Markdown, Rust, integration tests)
- **Check all**: `mise run check` - Runs all check tasks (Markdown, GitHub Actions, Rust, integration tests)

### Running Single Tests

Use cargo's test filtering: `cargo test test_name -- --nocapture`

Example: `cargo test test_find_by_id -- --nocapture`

## Architecture

### MCP Server Structure

The server is built using the `rmcp` (Rust MCP) framework with HTTP transport:

- **main.rs**: Entry point that sets up the Axum HTTP server on port 8000 (configurable via `PORT` env var) and binds the MCP service at `/mcp` endpoint
- **oeis.rs**: Core MCP tool definitions using `rmcp` macros (`#[tool_router]`, `#[tool]`, `#[tool_handler]`)
- **oeis_client.rs**: HTTP client that queries the OEIS API at `https://oeis.org/search`
- **tracer.rs**: Tracing/logging setup using `tracing-subscriber`

### MCP Tools Exposed

The `OEIS` struct implements `ServerHandler` and exposes two MCP tools:

1. **get_url**: Returns the OEIS homepage URL
2. **find_by_id**: Searches OEIS by sequence ID (e.g., "A000045") and returns structured sequence data including number, data points, name, comments, formulas, cross-references, and keywords

### MCP Prompts Exposed

The server exposes MCP prompts for guided workflows:

1. **sequence_analysis**: Provides a comprehensive analysis prompt for an OEIS sequence
   - Takes a `sequence_id` parameter (e.g., "A000045")
   - Returns a conversation-style prompt with user request and sequence data context
   - Guides AI models to analyze mathematical properties, patterns, applications, and relationships

### MCP Resources Exposed

The server also exposes MCP resources for direct data access:

1. **Resource Template**: `oeis://sequence/{id}`
   - URI pattern for accessing individual OEIS sequences as resources
   - Example: `oeis://sequence/A000045` returns JSON representation of the Fibonacci sequence
   - MIME type: `application/json`
   - Enables AI models to directly read sequence data as context without invoking tools

### Key Design Patterns

- Uses `rmcp` procedural macros for tool, prompt, and resource definition and routing
- Tool and prompt handlers are async methods on the `OEIS` struct
- Request/response types derive `JsonSchema` for MCP protocol validation
- Error handling maps to MCP error codes (INTERNAL_ERROR, INVALID_PARAMS)
- Prompts return `Vec<PromptMessage>` with conversation-style interactions
- Client uses `reqwest` with rustls for HTTPS
- Tests use `httpmock` for mocking OEIS API responses

### Module Organization

- **OEIS** (oeis.rs): MCP tool, prompt, and resource definitions with request/response types. Implements:
  - `#[tool_router]` for tools
  - `#[prompt_router]` for prompts
  - `ServerHandler` methods for resources (`list_resource_templates`, `read_resource`)
  - `#[tool_handler]` and `#[prompt_handler]` for MCP protocol integration
- **OEISClient** (oeis_client.rs): HTTP client with `find_by_id` method, includes comprehensive unit tests
- **OEISSequence**: Shared data structure representing an OEIS sequence entry

### MCP Capabilities Overview

- **Tools** (Actions): `get_url` and `find_by_id` are tools that perform actions when called
- **Prompts** (Workflows): `sequence_analysis` provides guided conversation templates for AI models
- **Resources** (Data): The `oeis://sequence/{id}` resource provides direct read access to sequence data
- Resources enable AI models to load sequence information as context, while tools are for active operations, and prompts provide structured workflows

### Configuration

- Server port: Set via `PORT` environment variable (defaults to 8000)
- Tracing level: Set via `RUST_LOG` environment variable (defaults to "debug")

## Testing

Tests are located in `oeis_client.rs` using the `#[cfg(test)]` module pattern. They use:

- `tokio::test` for async tests
- `httpmock` for mocking HTTP responses
- Helper functions: `setup_test_client()` and `mock_oeis_search()`

Test coverage includes success cases, not-found scenarios, and error handling.
