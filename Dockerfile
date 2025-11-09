#
# Configuration
#
ARG APP_NAME=oeis-mcp-server
ARG PLATFORM=x86_64-unknown-linux-musl


#
# Build stage
#
FROM rust:1.90-slim AS builder

ARG PLATFORM

WORKDIR /app

# Install dependencies for musl building
RUN apt update -qq && \
    apt install -y musl-tools
RUN rustup target add ${PLATFORM}

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target=${PLATFORM}
RUN rm -rf src

# Copy source code
COPY src ./src

# Build the application
# Touch main.rs to force rebuild of our code only
RUN touch src/main.rs && cargo build --release --target=${PLATFORM}


#
# Runtime stage
#
FROM scratch

ARG APP_NAME
ARG PLATFORM

# Copy the binary from builder
COPY --from=builder /app/target/${PLATFORM}/release/${APP_NAME} ./app

# Set environment variables
ENV PORT=8000
ENV RUST_LOG=info

# Expose the port
EXPOSE 8000

# Run the application
ENTRYPOINT ["./app"]
