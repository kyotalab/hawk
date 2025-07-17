# Text Processing Guide

Comprehensive guide to text and log processing with hawk.

## 📖 Table of Contents

- [Text Processing Fundamentals](#text-processing-fundamentals)
- [Log File Analysis](#log-file-analysis)
- [String Operations](#string-operations)
- [Pattern Matching and Filtering](#pattern-matching-and-filtering)
- [Text Transformation](#text-transformation)
- [Data Extraction](#data-extraction)
- [Advanced Text Patterns](#advanced-text-patterns)
- [Performance Optimization](#performance-optimization)
- [Real-world Examples](#real-world-examples)

## Text Processing Fundamentals

### Understanding Text Mode

hawk processes text files line-by-line when using the `--text` flag, treating each line as a string element in an array.

```bash
# Force text processing mode
hawk --text 'query' file.txt
hawk -t 'query' file.txt

# When to use --text flag
hawk -t '. | select(. | contains("ERROR"))' app.log
```

### Text vs Structured Data

| Mode               | Use Case              | Example                                    |
| ------------------ | --------------------- | ------------------------------------------ |
| **Auto-detect**    | JSON, YAML, CSV files | `hawk '.field' data.json`                  |
| **Text mode (-t)** | Log files, plain text | `hawk -t '. \| contains("ERROR")' app.log` |
| **Force text**     | Ambiguous files       | `hawk -t 'query' structured.log`           |

### Basic Text Processing Workflow

```bash
1. Read text file    → hawk -t '. | length' file.txt
2. Filter lines      → hawk -t '. | select(condition)' file.txt
3. Transform text    → hawk -t '. | map(operation)' file.txt
4. Extract data      → hawk -t '. | map(. | split(" ")[0])' file.txt
5. Analyze results   → hawk -t '. | unique | count' file.txt
```

## Log File Analysis

### Common Log Formats

#### Application Logs

```
2024-01-15 09:00:01 INFO Application started successfully
2024-01-15 09:00:02 DEBUG Loading configuration from /etc/app/config.json
2024-01-15 09:01:23 ERROR Failed to process user request: connection timeout
2024-01-15 09:01:24 INFO Retrying connection...
2024-01-15 09:02:45 WARN High memory usage detected: 85%
```

**Analysis Examples:**

```bash
# Find all error messages
hawk -t '. | select(. | contains("ERROR"))' app.log

# Extract timestamps
hawk -t '. | map(. | split(" ")[0])' app.log

# Count log levels
hawk -t '. | map(. | split(" ")[2]) | unique | count' app.log

# Get unique dates
hawk -t '. | map(. | substring(0, 10)) | unique | sort' app.log
```

#### Docker Container Logs

```
2024-01-15T10:30:45Z web_server GET /api/users 200 0.045s
2024-01-15T10:30:46Z database_service Connected to MySQL
2024-01-15T10:30:47Z web_server POST /api/auth 401 0.012s
2024-01-15T10:30:48Z cache_service Redis cache miss for key:user:123
```

**Analysis Examples:**

```bash
# Extract service names
hawk -t '. | map(. | split(" ")[1]) | unique' docker.log

# HTTP status code analysis
hawk -t '. | select(. | contains("GET|POST")) | map(. | split(" ")[4]) | group_by(.) | count' docker.log

# Service activity timeline
hawk -t '. | map(. replace("T", " ")) | map(. | split(" ")[0:2] | map(. | join("-"))' docker.log
```

#### Nginx/Apache Access Logs

```
192.168.1.100 - - [15/Jan/2024:10:30:45 +0000] "GET /api/users HTTP/1.1" 200 1234 "https://example.com" "Mozilla/5.0"
192.168.1.101 - - [15/Jan/2024:10:30:46 +0000] "POST /api/auth HTTP/1.1" 401 567 "-" "curl/7.68.0"
192.168.1.102 - - [15/Jan/2024:10:30:47 +0000] "GET /favicon.ico HTTP/1.1" 404 0 "https://example.com" "Mozilla/5.0"
```

**Analysis Examples:**

```bash
# Extract IP addresses
hawk -t '. | map(. | split(" ")[0]) | unique | sort' access.log

# Status code distribution
hawk -t '. | map(. | split("\"")[2] | split(" ")[1]) | group_by(.) | count' access.log

# Find 4xx and 5xx errors
hawk -t '. | select(. | contains("\" 4") | . | contains("\" 5"))' access.log

# Top user agents
hawk -t '. | map(. | split("\"")[5]) | group_by(.) | count | sort' access.log

# Requests per hour
hawk -t '. | map(. | split("[")[1] | split(":")[1]) | group_by(.) | count' access.log
```

#### System Logs (syslog format)

```
Jan 15 10:30:45 server01 kernel: [12345.678] TCP: Peer 192.168.1.100:443 unexpectedly shrunk window
Jan 15 10:30:46 server01 sshd[1234]: Accepted password for user from 192.168.1.200 port 22 ssh2
Jan 15 10:30:47 server01 systemd[1]: Started User Manager for UID 1000.
```

**Analysis Examples:**

```bash
# Extract service names
hawk -t '. | map(. | split(" ")[3] | split("[")[0]) | unique' syslog

# SSH connection analysis
hawk -t '. | select(. | contains("sshd")) | map(. | split(" from ")[1] | split(" ")[0]) | unique' syslog

# System service events
hawk -t '. | select(. | contains("systemd")) | map(. | split(": ")[1])' syslog

# Error pattern analysis
hawk -t '. | select(. | contains("error\|Error\|ERROR")) | map(. | split(" ")[3])' syslog
```

## String Operations

### Basic String Transformations

```bash
# Case conversion
hawk -t '. | map(. | upper)' text.txt           # Convert to uppercase
hawk -t '. | map(. | lower)' text.txt           # Convert to lowercase

# Whitespace management
hawk -t '. | map(. | trim)' text.txt            # Remove leading/trailing spaces
hawk -t '. | map(. | trim_start)' text.txt      # Remove leading spaces only
hawk -t '. | map(. | trim_end)' text.txt        # Remove trailing spaces only

# String analysis
hawk -t '. | map(. | length)' text.txt          # Get line lengths
hawk -t '. | map(. | reverse)' text.txt         # Reverse each line
```

### Advanced String Operations

```bash
# Text replacement
hawk -t '. | map(. | replace("old", "new"))' text.txt

# Substring extraction
hawk -t '. | map(. | substring(0, 10))' text.txt        # First 10 characters
hawk -t '. | map(. | substring(5))' text.txt            # From 5th character to end

# String splitting with array access (NEW!)
hawk -t '. | map(. | split(" ")[0])' text.txt           # First word
hawk -t '. | map(. | split(",")[2])' csv_like.txt       # Third CSV column
hawk -t '. | map(. | split(":")[1] | trim)' key_value.txt # Extract values
```

### Multiple Field String Operations (NEW!)

```bash
# Apply same operation to multiple fields in structured data
hawk '.users[] | map(.first_name, .last_name | upper)' users.json
hawk '.posts[] | map(.title, .content | trim)' posts.json
hawk '.logs[] | map(.message, .details | lower)' structured_logs.json
```

## Pattern Matching and Filtering

### Basic Pattern Matching

```bash
# Contains pattern
hawk -t '. | select(. | contains("ERROR"))' logs.txt

# Case-insensitive search
hawk -t '. | select(. | upper | contains("ERROR"))' logs.txt

# Multiple patterns (OR logic)
hawk -t '. | select(. | contains("ERROR") | . | contains("WARN"))' logs.txt

# Exclude patterns
hawk -t '. | select(. | contains("INFO") | not)' logs.txt
```

### Advanced Pattern Matching

```bash
# String starts/ends with pattern
hawk -t '. | select(. | starts_with("[INFO]"))' logs.txt
hawk -t '. | select(. | ends_with(".log"))' filenames.txt

# Length-based filtering
hawk -t '. | select(. | length > 100)' long_lines.txt
hawk -t '. | select(. | length < 20)' short_lines.txt

# Complex conditions
hawk -t '. | select(. | contains("HTTP") && . | contains("200"))' access.log
hawk -t '. | select(. | starts_with("2024") && . | contains("ERROR"))' timestamped.log
```

### Log Level Filtering

```bash
# Standard log levels
hawk -t '. | select(. | contains("DEBUG"))' app.log
hawk -t '. | select(. | contains("INFO"))' app.log
hawk -t '. | select(. | contains("WARN"))' app.log
hawk -t '. | select(. | contains("ERROR"))' app.log
hawk -t '. | select(. | contains("FATAL"))' app.log

# Severity filtering (ERROR and above)
hawk -t '. | select(. | contains("ERROR") | . | contains("FATAL"))' app.log

# Time-based filtering
hawk -t '. | select(. | starts_with("2024-01-15"))' dated_logs.txt
hawk -t '. | select(. | substring(11, 2) == "09")' hourly_filter.log  # 9 AM only
```

## Text Transformation

### Data Extraction

```bash
# Extract timestamps from logs
hawk -t '. | map(. | split(" ")[0])' timestamped.log

# Extract IP addresses from access logs
hawk -t '. | map(. | split(" ")[0])' access.log

# Extract HTTP methods
hawk -t '. | map(. | split("\"")[1] | split(" ")[0])' access.log

# Extract file paths
hawk -t '. | map(. | split("/") | last)' file_paths.txt

# Extract domains from URLs
hawk -t '. | map(. | split("://")[1] | split("/")[0])' urls.txt
```

### CSV-like Text Processing

```bash
# Process comma-separated values
hawk -t '. | map(. | split(",")[0])' csv_data.txt        # First column
hawk -t '. | map(. | split(",")[1] | trim)' csv_data.txt # Second column, trimmed

# Process tab-separated values
hawk -t '. | map(. | split("\t")[2])' tsv_data.txt

# Process pipe-separated values
hawk -t '. | map(. | split("|")[1])' pipe_data.txt

# Join processed data back
hawk -t '. | map(. | split(",") | join(" | "))' csv_data.txt
```

### Key-Value Extraction

```bash
# Extract values from key=value format
hawk -t '. | select(. | contains("=")) | map(. | split("=")[1])' config.txt

# Extract specific keys
hawk -t '. | select(. | starts_with("user=")) | map(. | split("=")[1])' key_value.txt

# Process JSON-like logs
hawk -t '. | select(. | contains("\"level\"")) | map(. | split("\"level\":\"")[1] | split("\"")[0])' json_logs.txt
```

## Data Extraction

### Email and URL Extraction

```bash
# Extract email addresses (basic pattern)
hawk -t '. | select(. | contains("@")) | map(. | split(" ") | select(. | contains("@")))' text.txt

# Extract domains from emails
hawk -t '. | select(. | contains("@")) | map(. | split("@")[1])' emails.txt

# Extract URLs (basic pattern)
hawk -t '. | select(. | contains("http")) | map(. | split(" ") | select(. | starts_with("http")))' text.txt
```

### Numeric Data Extraction

```bash
# Extract numbers from text
hawk -t '. | map(. | split(" ") | select(. | length > 0) | select(. | replace("[^0-9.]", "") | length > 0))' mixed.txt

# Extract percentages
hawk -t '. | select(. | contains("%")) | map(. | split("%")[0] | split(" ") | last)' percentages.txt

# Extract timestamps (ISO format)
hawk -t '. | map(. | substring(0, 19))' iso_timestamps.txt

# Extract version numbers
hawk -t '. | select(. | contains("v")) | map(. | split("v")[1] | split(" ")[0])' versions.txt
```

### Error Code and Status Extraction

```bash
# HTTP status codes
hawk -t '. | map(. | split(" ")[8])' access.log            # Standard access log format
hawk -t '. | select(. | split(" ")[8] >= "400")' access.log # 4xx and 5xx errors

# Exit codes from logs
hawk -t '. | select(. | contains("exit code")) | map(. | split("exit code ")[1] | split(" ")[0])' process.log

# Error numbers
hawk -t '. | select(. | contains("errno")) | map(. | split("errno=")[1] | split(" ")[0])' system.log
```

## Advanced Text Patterns

### Multi-line Log Processing

```bash
# Process stack traces (keep related lines together)
hawk -t '. | select(. | contains("Exception") | . | starts_with("\t"))' java.log

# Group by session ID
hawk -t '. | select(. | contains("session=")) | map(. | split("session=")[1] | split(" ")[0])' session.log

# Process multiline JSON logs (single line JSON per log entry)
hawk -t '. | select(. | starts_with("{") && . | ends_with("}"))' json.log
```

### Performance Log Analysis

```bash
# Response time analysis
hawk -t '. | select(. | contains("ms")) | map(. | split(" ") | select(. | ends_with("ms")) | replace("ms", ""))' perf.log

# Memory usage tracking
hawk -t '. | select(. | contains("memory")) | map(. | split("memory: ")[1] | split(" ")[0])' memory.log

# CPU usage extraction
hawk -t '. | select(. | contains("cpu")) | map(. | split("cpu: ")[1] | split("%")[0])' cpu.log
```

### Security Log Analysis

```bash
# Failed login attempts
hawk -t '. | select(. | contains("failed login")) | map(. | split("from ")[1] | split(" ")[0])' auth.log

# Suspicious activity patterns
hawk -t '. | select(. | contains("SUSPICIOUS") | . | contains("ANOMALY"))' security.log

# IP-based analysis
hawk -t '. | map(. | split(" ")[0]) | group_by(.) | count | sort' network.log
```

## Performance Optimization

### Efficient Text Processing

```bash
# ✅ Filter early in pipeline
hawk -t '. | select(. | contains("ERROR")) | map(. | split(" ")[0])' large.log

# ❌ Process everything then filter
hawk -t '. | map(. | split(" ")[0]) | select(. | contains("ERROR"))' large.log

# ✅ Use specific operations
hawk -t '. | map(. | split(" ")[0])' log.txt

# ❌ Use complex operations when simple ones suffice
hawk -t '. | map(. | replace(...) | substring(...) | split(...))' log.txt
```

### Memory Management

```bash
# ✅ Process in chunks for large files
hawk -t '.[0:10000] | select(. | contains("ERROR"))' huge.log

# ✅ Sample large datasets
hawk -t '.[::100] | map(. | split(" ")[0])' massive.log  # Every 100th line

# ✅ Use appropriate data types
hawk -t '. | select(. | length > 0)' text.txt           # String operations
```

### Slicing for Performance (NEW!)

```bash
# ✅ Process recent logs only
hawk -t '.[-1000:] | select(. | contains("ERROR"))' app.log    # Last 1000 lines

# ✅ Sample from different time periods
hawk -t '.[0:100] | .[500:600] | .[1000:1100]' distributed_sample.log

# ✅ Top/bottom analysis
hawk -t '. | sort | .[0:10]' values.txt                # Bottom 10
hawk -t '. | sort | .[-10:]' values.txt                # Top 10
```

## Real-world Examples

### Complete Log Analysis Workflows

#### Web Server Log Analysis

```bash
# 1. Overview of traffic
hawk -t '. | count' access.log                          # Total requests
hawk -t '. | map(. | split(" ")[0]) | unique | count' access.log  # Unique IPs

# 2. Error analysis
hawk -t '. | select(. | contains("\" 4") | . | contains("\" 5")) | count' access.log

# 3. Top pages
hawk -t '. | map(. | split("\"")[1] | split(" ")[1]) | group_by(.) | count | sort' access.log

# 4. Traffic patterns by hour
hawk -t '. | map(. | split("[")[1] | split(":")[1]) | group_by(.) | count' access.log

# 5. User agent analysis
hawk -t '. | map(. | split("\"")[5]) | group_by(.) | count | .[-10:]' access.log
```

#### Application Error Investigation

```bash
# 1. Error trend analysis
hawk -t '. | select(. | contains("ERROR")) | map(. | substring(0, 13)) | group_by(.) | count' app.log

# 2. Error types
hawk -t '. | select(. | contains("ERROR")) | map(. | split("ERROR ")[1] | split(":")[0]) | count' app.log

# 3. Related warnings
hawk -t '. | select(. | contains("WARN")) | select(. | contains("connection\|timeout\|retry"))' app.log
```

#### System Performance Monitoring

```bash
# 1. Memory usage trends
hawk -t '. | select(. | contains("memory")) | map(. | split("memory: ")[1] | split(" ")[0])' system.log

# 2. Disk space monitoring
hawk -t '. | select(. | contains("disk")) | map(. | split("usage: ")[1] | split("%")[0])' disk.log

# 3. Network activity
hawk -t '. | select(. | contains("bytes")) | map(. | split("bytes: ")[1] | split(" ")[0])' network.log

# 4. Process analysis
hawk -t '. | select(. | contains("process")) | map(. | split(" ")[3]) | group_by(.) | count' process.log
```

#### Security Log Analysis

```bash
# 1. Authentication failures
hawk -t '. | select(. | contains("authentication failed")) | map(. | split("from ")[1] | split(" ")[0]) | group_by(.) | count' security.log

# 2. Unusual access patterns
hawk -t '. | select(. | contains("GET") && . | contains("admin")) | map(. | split(" ")[0])' access.log

# 3. Brute force detection
hawk -t '. | select(. | contains("failed password")) | map(. | split(" ")[0]) | group_by(.) | count | select(. > 10)' auth.log

# 4. Geographic analysis (if GeoIP data available)
hawk -t '. | map(. | split(" ")[0]) | unique' access.log  # Extract IPs for GeoIP lookup
```

#### DevOps Pipeline Logs

```bash
# 1. Build success/failure rates
hawk -t '. | select(. | contains("BUILD")) | map(. | split("BUILD ")[1] | split(" ")[0]) | group_by(.) | count' ci.log

# 2. Deployment timing
hawk -t '. | select(. | contains("DEPLOY")) | map(. | split(" ")[0])' deploy.log

# 3. Test results analysis
hawk -t '. | select(. | contains("TEST")) | map(. | split("TEST ")[1]) | group_by(.) | count' test.log

# 4. Resource usage during builds
hawk -t '. | select(. | contains("CPU\|MEMORY")) | map(. | split(": ")[1])' resource.log
```

## Best Practices

### Text Processing Guidelines

1. **Always use --text flag for log files**: Prevents YAML/JSON misdetection
2. **Filter early**: Apply `select()` before expensive operations
3. **Use specific extractors**: Prefer `split()[index]` over complex regex alternatives
4. **Handle edge cases**: Check for empty results and missing fields
5. **Sample large files**: Use slicing for performance with huge datasets

### Common Patterns

```bash
# ✅ Good: Extract then analyze
hawk -t '. | map(. | split(" ")[0]) | unique | count' log.txt

# ✅ Good: Filter then transform
hawk -t '. | select(. | contains("ERROR")) | map(. | split(" ")[1])' log.txt

# ✅ Good: Use appropriate data types
hawk -t '. | select(. | length > 0) | map(. | trim)' text.txt

# ✅ Good: Handle missing data
hawk -t '. | select(. | contains(" ")) | map(. | split(" ")[1])' structured.txt
```

### Debugging Text Processing

```bash
# Check data structure
hawk -t '. | .[0:5]' file.txt                          # Sample first 5 lines

# Validate operations step by step
hawk -t '. | map(. | split(" "))' file.txt             # Step 1: split
hawk -t '. | map(. | split(" ")[0])' file.txt          # Step 2: index access

# Check for empty or problematic lines
hawk -t '. | select(. | length == 0)' file.txt         # Find empty lines
hawk -t '. | select(. | contains("\t"))' file.txt      # Find tab characters
```

---

**Related Documentation:**

- [Getting Started](getting-started.md) - Basic hawk introduction
- [String Operations](string-operations.md) - Detailed string processing reference
- [Query Language](query-language.md) - Complete syntax guide
- [Log Analysis Examples](examples/log-analysis.md) - Real-world log processing cases
