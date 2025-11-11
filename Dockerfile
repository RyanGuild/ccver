# Builder stage - selects the appropriate binary
FROM alpine:latest AS builder

# Create directory for the binary
RUN mkdir -p /usr/src/app

# Copy pre-built binaries from the build workflow
COPY binaries/ /binaries/

# Auto-detect and select the appropriate binary
# Prefer arm64 if available (for Apple Silicon), otherwise use amd64
RUN if [ -f /binaries/ccver-linux-arm64 ]; then \
        cp /binaries/ccver-linux-arm64 /usr/src/app/ccver && \
        echo "Selected arm64 binary" && \
        ls -lh /usr/src/app/ccver; \
    elif [ -f /binaries/ccver-linux-amd64 ]; then \
        cp /binaries/ccver-linux-amd64 /usr/src/app/ccver && \
        echo "Selected amd64 binary" && \
        ls -lh /usr/src/app/ccver; \
    else \
        echo "Error: No suitable binary found in /binaries/" && \
        echo "Available files:" && \
        ls -la /binaries/ && \
        exit 1; \
    fi && \
    chmod +x /usr/src/app/ccver

# Final stage - use ubuntu base image
FROM ubuntu:latest AS runner

# Install git (Ubuntu doesn't include git by default)
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*

# Copy the pre-built binary from the builder stage
COPY --from=builder /usr/src/app/ccver /ccver

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
