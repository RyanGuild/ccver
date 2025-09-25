#!/bin/bash
set -e

# Build the command arguments
ARGS=()

if [ -n "$INPUT_PATH" ]; then
  ARGS+=(--path="$INPUT_PATH")
fi

if [ -n "$INPUT_FORMAT" ]; then
  ARGS+=(--format="$INPUT_FORMAT")
fi

if [ "$INPUT_NO_PRE" == "true" ]; then
  ARGS+=(--no-pre)
fi

if [ "$INPUT_CI" == "true" ]; then
  ARGS+=(--ci)
fi

if [ -n "$INPUT_COMMAND" ]; then
  ARGS+=("$INPUT_COMMAND")
fi

# Run ccver with the constructed arguments
OUTPUT=$(ccver "${ARGS[@]}")

# Set the output for GitHub Actions
echo "version=$OUTPUT" >> $GITHUB_OUTPUT

# If the command was changelog, also set the changelog output
if [ "$INPUT_COMMAND" == "changelog" ]; then
  echo "changelog=$OUTPUT" >> $GITHUB_OUTPUT
fi

# Print the output for logging
echo "$OUTPUT"
