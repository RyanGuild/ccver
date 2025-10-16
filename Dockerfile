# Use the official Rust image as the base image
FROM --platform=$BUILDPLATFORM rust:latest AS builder

# Install git (required for ccver to work)
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*

# Set the working directory inside the container
WORKDIR /usr/src/app

# Arguments for cross-compilation
ARG TARGETPLATFORM
ARG BUILDPLATFORM

# Install only the specific Rust target needed for the target platform
# Don't copy rust-toolchain.toml to avoid installing all targets
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

# Copy the benches (required by Cargo.toml, but won't be built)
COPY benches ./benches

# Copy the hooks
COPY bin ./bin

# Build the application in release mode for the specific target
# Using --bin ccver ensures we only build the main binary, not benches or other bins
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
