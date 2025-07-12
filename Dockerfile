# Use the official Rust image as the base image
FROM rust:latest as builder

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src
COPY migrations ./migrations
COPY diesel.toml ./

# Build the application in release mode
RUN cargo build --release

# Start a new stage for the runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false -m -d /app appuser

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/role-channel-blacklist /app/role-channel-blacklist

# Change ownership of the app directory to the appuser
RUN chown -R appuser:appuser /app

# Switch to the non-root user
USER appuser

# Expose any necessary ports (Discord bots typically don't need exposed ports)
# EXPOSE 8080

# Set the entrypoint
CMD ["./role-channel-blacklist"]
