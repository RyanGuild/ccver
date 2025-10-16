# Use the official Rust image as the base image
FROM --platform=$BUILDPLATFORM rust:latest AS builder

# Install git (required for ccver to work)
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the Rust toolchain configuration
COPY rust-toolchain.toml .

# Install the nightly toolchain specified in rust-toolchain.toml
RUN rustup toolchain install nightly
RUN rustup default nightly

# Arguments for cross-compilation
ARG TARGETPLATFORM
ARG BUILDPLATFORM

# Install the appropriate Rust target based on the target platform
RUN case "$TARGETPLATFORM" in \
    "linux/amd64") echo "x86_64-unknown-linux-gnu" > /rust_target.txt ;; \
    "linux/arm64") echo "aarch64-unknown-linux-gnu" > /rust_target.txt ;; \
    "linux/arm/v7") echo "armv7-unknown-linux-gnueabihf" > /rust_target.txt ;; \
    *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac && \
    export RUST_TARGET=$(cat /rust_target.txt) && \
    rustup target add $RUST_TARGET

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Copy the benches
COPY benches ./benches

# Copy the hooks
COPY bin ./bin

# Build the application in release mode for the specific target
RUN export RUST_TARGET=$(cat /rust_target.txt) && \
    cargo build --release --bin ccver --target $RUST_TARGET && \
    mkdir -p /usr/src/app/target/release && \
    cp /usr/src/app/target/$RUST_TARGET/release/ccver /usr/src/app/target/release/ccver

RUN ls -la /usr/src/app/target/release

# Start a new stage for the final image
FROM --platform=$TARGETPLATFORM ubuntu:latest AS runner

# Install git (Ubuntu doesn't include git by default)
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/ccver /ccver

# Copy SBOM files (directory contains at minimum a .gitkeep file)
COPY --chown=root:root sbom /usr/share/sbom

RUN ls -la /

ENV PATH="/usr/local/bin:${PATH}"

# Add labels for SBOM and supply chain metadata
LABEL org.opencontainers.image.sbom.location="/usr/share/sbom"
LABEL org.opencontainers.image.sbom.formats="CycloneDX,SPDX"

# Set the working directory to the workspace
WORKDIR /github/workspace


# Set the entrypoint to the ccver binary (can be overridden by action.yml)
ENTRYPOINT ["/ccver"]
