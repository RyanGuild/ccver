# Tracing and Logging in ccver

This document describes the tracing and logging capabilities added to ccver for better observability and debugging.

## Features

### Structured Logging
- All major operations are instrumented with structured logging
- Performance metrics are captured for critical operations
- Error context is enhanced with tracing information

### Log Levels
- `TRACE`: Very detailed execution information
- `DEBUG`: Debug information including intermediate values
- `INFO`: General information about operations (default for ccver modules)
- `WARN`: Warning messages
- `ERROR`: Error messages with context

### Environment Variables

#### RUST_LOG
Control logging levels using the `RUST_LOG` environment variable:

```bash
# Default: info level for most, debug for ccver modules
export RUST_LOG="info,ccver=debug"

# Debug everything
export RUST_LOG="debug"

# Only errors
export RUST_LOG="error"

# Specific module debugging
export RUST_LOG="info,ccver::git=trace,ccver::graph=debug"
```

#### Examples

```bash
# Run with detailed logging
RUST_LOG=debug ccver --format "v{major}.{minor}.{patch}" tag

# Run with minimal logging
RUST_LOG=error ccver --format "v{major}.{minor}.{patch}" tag

# Run with performance metrics
RUST_LOG=info,ccver::git=debug,ccver::graph=debug ccver tag
```

## Performance Metrics

The following operations include timing information:

- **Git log retrieval**: Time to fetch git logs and data size
- **Commit graph building**: Time to parse logs and build graph structure
- **Version map creation**: Time to calculate version mappings

Example output:
```
2024-01-01T12:00:00.000000Z  INFO ccver::git: Retrieved git log data duration_ms=45 log_size=12847
2024-01-01T12:00:00.100000Z  INFO ccver::graph: Commit graph built successfully duration_ms=23 node_count=156 edge_count=155
2024-01-01T12:00:00.150000Z  INFO ccver::version_map: Version map created successfully duration_ms=12 map_entries=156
```

## Structured Fields

Log entries include structured fields for better parsing and analysis:

- `duration_ms`: Operation duration in milliseconds
- `log_size`: Size of git log data in characters
- `node_count`: Number of nodes in commit graph
- `edge_count`: Number of edges in commit graph
- `map_entries`: Number of version map entries
- `command`: Command being executed
- `path`: Repository path
- `format`: Version format string
- `error`: Error information when operations fail

## Integration with External Tools

The structured logging output can be easily integrated with log analysis tools:

### JSON Output (if needed)
While not enabled by default, you can modify the logging configuration to output JSON for machine parsing.

### Log Aggregation
The structured fields make it easy to aggregate metrics and monitor performance across different repositories and operations.
