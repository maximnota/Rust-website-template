# Use the official Rust image as a build environment
FROM rust:1.75 as builder

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/rwa ./

# Copy static assets and pages
COPY pages ./pages
COPY static ./static
COPY error_pages ./error_pages

# Expose port (Railway will set PORT env var)
EXPOSE $PORT

# Run the binary
CMD ["./rwa"]
