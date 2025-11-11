# Use a minimal base image for the builder stage (just to organize binaries)
ARG BUILDPLATFORM=linux/amd64
FROM --platform=$BUILDPLATFORM alpine:latest AS builder

# Arguments for selecting the correct binary
ARG TARGETPLATFORM

# Create directory for the binary
RUN mkdir -p /usr/src/app

# Copy pre-built binaries from the build workflow
COPY binaries/ /binaries/

# Select the appropriate binary based on the target platform
RUN case "$TARGETPLATFORM" in \
    "linux/amd64") cp /binaries/ccver-linux-amd64 /usr/src/app/ccver ;; \
    "linux/arm64") cp /binaries/ccver-linux-arm64 /usr/src/app/ccver ;; \
    "linux/arm/v7") echo "ARM v7 not yet supported with pre-built binaries" && exit 1 ;; \
    *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac && \
    chmod +x /usr/src/app/ccver

# Start a new stage for the final image
FROM --platform=$TARGETPLATFORM ubuntu:latest AS runner

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
