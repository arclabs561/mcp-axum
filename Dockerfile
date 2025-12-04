# Multi-stage build for production MCP server
FROM rust:1.75 as builder

WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY examples ./examples

# Build release binary (example - users should build their own)
RUN cargo build --release --example basic_server

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/mcp-axum /usr/local/bin/mcp-server

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV BIND_ADDRESS=0.0.0.0:8080

# Run server
CMD ["mcp-server"]

