# Rust implementation of the OEIS MCP server

## Usage

### Use Docker (recommended)

1. Run on port 8000 (you can specify the host port)

    ```sh
    docker run -p 8000:8000 --name oeis-mcp-server ghcr.io/23prime/oeis-mcp-server:latest
    ```

1. Config your client

    e.g.) Claude Code

    ```sh
    claude mcp add --transport http oeis http://localhost:8000/mcp
    ```

## Development

### Pre-requirements

- [mise](https://mise.jdx.dev)
- [rustup](https://rustup.rs)

### Run application

1. Setup

    ```sh
    mise run setup
    ```

2. Run app

    ```sh
    mise run rs-dev
    ```

3. Run integration tests at another terminal

    ```sh
    mise run test-check
    ```

### Use Docker

1. Build

    ```sh
    docker build -t oeis-mcp-server:latest .
    ```

2. Run

    ```sh
    docker run -p 8000:8000 --name oeis-mcp-server oeis-mcp-server:latest
    ```

### Release

1. Update version in `Cargo.toml`

    ```toml
    [package]
    version = "0.2.0"
    ```

    (Alternative) You can update  with the command:

    ```sh
    cargo set-version 0.2.0
    ```

2. Create release tag

    ```sh
    mise run tag
    ```

3. Push release tag

    ```sh
    mise run tag-push
    ```

4. GitHub Actions will automatically build and release binaries
