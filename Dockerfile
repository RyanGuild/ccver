# Use the official Rust image as the base image
FROM rust:latest AS builder

# Install git (required for ccver to work)
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the Rust toolchain configuration
COPY rust-toolchain.toml .

# Install the nightly toolchain specified in rust-toolchain.toml
RUN rustup toolchain install nightly
RUN rustup default nightly

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Start a new stage for the final image
FROM alpine:latest

# Install git (Alpine doesn't include git by default)
RUN apk add --no-cache git

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/ccver /usr/local/bin/ccver

# Copy the entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Set the working directory to the workspace
WORKDIR /github/workspace

# Set the entrypoint to the ccver binary (can be overridden by action.yml)
ENTRYPOINT ["/usr/local/bin/ccver"]

# Default command (can be overridden)
CMD ["--ci"]
