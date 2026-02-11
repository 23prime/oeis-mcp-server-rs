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

- [Taskfile](https://taskfile.dev)
- [mise](https://mise.jdx.dev)

### Run application

1. Setup

    ```sh
    task setup
    ```

2. Run app

    ```sh
    task rs:dev
    ```

3. Run integration tests at another terminal

    ```sh
    task t:check
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

2. Create release tag

    ```sh
    task tag
    ```

3. Push release tag

    ```sh
    task tag:push
    ```

4. GitHub Actions will automatically build and release binaries
