# String Operations Guide

Comprehensive guide to hawk's text processing capabilities.

## 📖 Table of Contents

- [Basic Operations](#basic-operations)
- [Advanced Operations](#advanced-operations)
- [Array Operations](#array-operations)
- [Multi-field Operations](#multi-field-operations)
- [Practical Examples](#practical-examples)
- [Performance Tips](#performance-tips)

## Basic Operations

### Case Conversion

```bash
# Convert to uppercase
hawk '. | map(. | upper)' names.txt

# Convert to lowercase
hawk '.users[] | map(.email | lower)' users.json

# Example
"Hello World" | upper  → "HELLO WORLD"
"Hello World" | lower  → "hello world"
```

### Whitespace Management

```bash
# Remove all whitespace
hawk '. | map(. | trim)' messy-data.txt

# Remove leading whitespace
hawk '. | map(. | trim_start)' indented.txt

# Remove trailing whitespace
hawk '. | map(. | trim_end)' data.txt

# Examples
"  hello  " | trim       → "hello"
"  hello  " | trim_start → "hello  "
"  hello  " | trim_end   → "  hello"
```

### String Analysis

```bash
# Get string length
hawk '. | map(. | length)' text.txt

# Reverse strings
hawk '. | map(. | reverse)' data.txt

# Examples
"hello" | length  → 5
"hello" | reverse → "olleh"
```

## Advanced Operations

### Pattern Matching

```bash
# Check if string contains pattern
hawk '. | select(. | contains("ERROR"))' logs.txt

# Check string start/end
hawk '. | select(. | starts_with("2024"))' timestamps.txt
hawk '. | select(. | ends_with(".log"))' filenames.txt

# Examples
"Hello World" | contains("World")     → true
"Hello World" | starts_with("Hello") → true
"Hello World" | ends_with("World")   → true
```

### Text Transformation

```bash
# Replace text
hawk '. | map(. | replace("old", "new"))' text.txt

# Extract substrings
hawk '. | map(. | substring(0, 10))' long-text.txt
hawk '. | map(. | substring(5))' text.txt  # from index 5 to end

# Examples
"Hello World" | replace("World", "Rust") → "Hello Rust"
"Hello World" | substring(0, 5)          → "Hello"
"Hello World" | substring(6)             → "World"
```

## Array Operations

### String Splitting

```bash
# Split into array
hawk '. | map(. | split(","))' csv-lines.txt
hawk '. | map(. | split(" "))' sentences.txt

# Split with index access (NEW in v0.2.2!)
hawk '. | map(. | split(" ")[0])' space-separated.txt
hawk '. | map(. | split(",")[2])' csv-data.txt

# Examples
"apple,banana,cherry" | split(",")    → ["apple", "banana", "cherry"]
"apple,banana,cherry" | split(",")[0] → "apple"
"apple,banana,cherry" | split(",")[1] → "banana"
```

### Array Joining

```bash
# Join array elements
hawk '.tags[] | join(",")' data.json
hawk '.words[] | join(" ")' word-lists.json

# Examples
["apple", "banana"] | join(",") → "apple,banana"
["hello", "world"] | join(" ")  → "hello world"
```

## Multi-field Operations

Process multiple fields with the same operation (NEW in v0.2.2!):

```bash
# Apply join to multiple array fields
hawk '.users[] | map(.skills, .projects | join(","))' users.json

# Convert multiple fields to uppercase
hawk '.users[] | map(.first_name, .last_name | upper)' users.json

# Get length of multiple string fields
hawk '.posts[] | map(.title, .content | length)' posts.json
```

### Example: User Data Processing

```json
// Input
{
  "users": [
    {
      "name": "alice",
      "skills": ["python", "rust"],
      "projects": ["web-app", "cli-tool"],
      "department": "engineering"
    }
  ]
}
```

```bash
# Process multiple fields simultaneously
hawk '.users[] | map(.name, .department | upper)' users.json

# Result
{
  "users": [
    {
      "name": "ALICE",           // ← converted
      "skills": ["python", "rust"],
      "projects": ["web-app", "cli-tool"],
      "department": "ENGINEERING" // ← converted
    }
  ]
}
```

## Practical Examples

### Log File Processing

```bash
# Extract timestamps from logs
hawk -t '. | map(. | split(" ")[0])' app.log

# Find unique IP addresses
hawk -t '. | map(. | split(" ")[0]) | unique' access.log

# Extract error messages
hawk -t '. | select(. | contains("ERROR")) | map(. | split(": ")[1])' error.log
```

### Data Cleaning

```bash
# Normalize email addresses
hawk '.users[] | map(.email | lower | trim)' users.csv

# Clean phone numbers
hawk '.contacts[] | map(.phone | replace("-", "") | replace("(", "") | replace(")", ""))' contacts.json

# Standardize names
hawk '.people[] | map(.name | trim | upper)' people.csv
```

### CSV Processing

```bash
# Extract specific columns from CSV-like text
hawk -t '. | map(. | split(",")[1])' data.txt

# Process headers and data separately
hawk -t '.[0] | split(",")' data.txt  # headers
hawk -t '.[1:] | map(. | split(",")[2])' data.txt  # data column
```

### Docker/Container Logs

```bash
# Extract container names
hawk -t '. | map(. | split(" ")[1]) | unique' docker.log

# Get timestamps and services
hawk -t '. | map(. replace("T", " ")) | map(. | split(" ")[0:2] | map(. | join("-"))' docker.log

# Filter by service and extract messages
hawk -t '. | select(. | contains("web_server")) | map(. | split(" ")[3:] | join(" "))' docker.log
```

## Performance Tips

### Efficient Patterns

```bash
# ✅ Good: Filter first, then transform
hawk '. | select(. | contains("ERROR")) | map(. | upper)' logs.txt

# ❌ Avoid: Transform everything, then filter
hawk '. | map(. | upper) | select(. | contains("ERROR"))' logs.txt
```

### Memory Considerations

```bash
# ✅ Process in chunks for large files
hawk '. | select(. | length > 100) | map(. | substring(0, 50))' large.txt

# ✅ Use specific operations instead of general ones
hawk '. | map(. | split(" ")[0])' data.txt  # Better than complex regex
```

### Text Format Detection

```bash
# ✅ Use --text flag for ambiguous files
hawk -t '. | map(. | split(" ")[0])' structured.log

# ✅ Especially important for logs that might be detected as YAML
hawk --text '. | select(. | contains("GC"))' gc.log
```

## Error Handling

### Common Issues

```bash
# Array index out of bounds → returns empty string
"a,b" | split(",")[5]  → ""

# Missing fields → error (use select to filter first)
hawk '.users[] | select(.email) | map(.email | lower)' users.json
```

### Debugging Tips

```bash
# Check data structure first
hawk '. | info' unknown-data.json

# Test operations step by step
hawk '. | map(. | split(" "))' data.txt        # Step 1: split
hawk '. | map(. | split(" ")[0])' data.txt     # Step 2: index access
```

## Chaining Operations

### Pipeline Examples

```bash
# Complex text processing pipeline
hawk -t '. | select(. | length > 10) | map(. | trim | upper | substring(0, 20))' text.txt

# Multi-step data cleaning
hawk '.users[] | map(.email | lower | trim) | select(. | ends_with(".com"))' users.json

# Log analysis workflow
hawk -t '. | select(. | contains("ERROR")) | map(. | split("][")[1] | split(" ")[0]) | unique | sort' app.log
```

---

**Next Steps:**

- [Data Analysis Guide](data-analysis.md) - Statistical operations and aggregation
- [Log Analysis Examples](examples/log-analysis.md) - Real-world log processing
- [Query Language Reference](query-language.md) - Complete syntax guide
