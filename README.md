# Rust implementation of the OEIS MCP server

## Usage

### Use Docker (recommended)

1. Run on port 8000 (you can specify the host port)

    ```sh
    docker run -p 8000:8000 --name oeis-mcp-server ghcr.io/23prime/oeis-mcp-server:latest
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
