# hawk 🦅

Modern data analysis tool for structured data and text files (JSON, YAML, CSV, Text)

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/hawk-data.svg)](https://crates.io/crates/hawk-data)

**hawk** combines the simplicity of `awk` with the power of `pandas` for data exploration. Unlike traditional text tools that work line-by-line, hawk understands both structured data and plain text natively. Unlike heavy data science tools that require complex setup, hawk brings analytics to your terminal with a single command.

**Perfect for**:

- 📊 **Data Scientists**: Quick CSV/JSON analysis without Python overhead
- 🔧 **DevOps Engineers**: Kubernetes YAML, Docker Compose, log analysis
- 🌐 **API Developers**: REST response exploration and validation
- 📈 **Business Analysts**: Instant insights from structured datasets
- 🔍 **System Administrators**: Log file analysis and text processing

## ✨ Features

- 🔍 **Universal format support**: JSON, YAML, CSV, **and plain text** with automatic detection
- 🐼 **Pandas-like operations**: Filtering, grouping, aggregation, **and string manipulation**
- 📊 **Smart output formatting**: Colored tables, lists, JSON based on data structure
- 🚀 **Fast and lightweight**: Built in Rust for performance
- 🔧 **Developer-friendly**: Perfect for DevOps, data analysis, and API exploration
- 🎯 **Type-aware**: Understands numbers, strings, booleans with intelligent conversion
- 🔄 **Unified syntax**: Same query language across all formats
- 🧵 **String operations**: Powerful text processing capabilities
- 📊 **Statistical functions**: Built-in median, stddev, unique, sort operations
- 🎨 **Beautiful output**: Automatic color coding with TTY detection

## 🚀 Quick Start

### Installation

```bash
# Install via Homebrew (macOS/Linux)
brew install kyotalab/tools/hawk

# Install via Cargo (if Rust is installed)
cargo install hawk-data

# Verify installation
hawk --version
```

### Basic Usage

```bash
# Explore data structure
hawk '. | info' data.json

# Access fields
hawk '.users[0].name' users.json
hawk '.users.name' users.csv

# Filter and aggregate
hawk '.users[] | select(.age > 30) | count' users.yaml
hawk '.sales | group_by(.region) | avg(.amount)' sales.csv

# Process text files (NEW in v0.2.0!)
hawk '. | select(. | contains("ERROR"))' app.log
hawk '. | map(. | substring(0, 19))' access.log
```

## 📖 Query Syntax

### Field Access

```bash
.field                    # Access field
.array[0]                 # Access array element
.array[]                  # Access all array elements
.nested.field             # Deep field access
.array[0].nested.field    # Complex nested access
.array[].nested[]         # Multi-level array expansion
```

### Text Processing (NEW in v0.2.0!)

```bash
# String operations
. | map(. | upper)                    # Convert to uppercase
. | map(. | lower)                    # Convert to lowercase
. | map(. | trim)                     # Remove whitespace (both ends)
. | map(. | trim_start)               # Remove leading whitespace
. | map(. | trim_end)                 # Remove trailing whitespace
. | map(. | length)                   # Get string length
. | map(. | reverse)                  # Reverse string

# String manipulation
. | map(. | replace("old", "new"))    # Replace text
. | map(. | substring(0, 10))         # Extract substring
. | map(. | split(","))               # Split by delimiter
.array[] | join(", ")                 # Join array elements

# String filtering
. | select(. | contains("ERROR"))     # Contains pattern
. | select(. | starts_with("INFO"))   # Starts with pattern
. | select(. | ends_with(".log"))     # Ends with pattern
```

### Statistical Operations (NEW in v0.2.0!)

```bash
. | unique                            # Remove duplicates
. | sort                              # Sort values
. | median                            # Calculate median
. | stddev                            # Calculate standard deviation
. | length                            # Get array length

# With field specification
.users[] | unique(.department)        # Unique departments
.scores[] | median(.value)            # Median of values
.data[] | sort(.timestamp)            # Sort by timestamp
```

### Filtering

```bash
. | select(.age > 30)           # Numeric comparison
. | select(.name == "Alice")    # String equality
. | select(.active == true)     # Boolean comparison
. | select(.status != "inactive") # Not equal
. | select(.State.Name == "running") # Nested field filtering

# Complex string filtering (NEW!)
. | select(. | upper | contains("ERROR"))  # Case-insensitive search
. | select(. | length > 50)                # Filter by string length
```

### Data Transformation (NEW in v0.2.0!)

```bash
# Transform data with map
.users[] | map(.email | lower)                    # Normalize emails
.logs[] | map(. | substring(0, 19))              # Extract timestamps
.data[] | map(.text | trim | upper)              # Clean and normalize

# Complex transformations
.files[] | map(.name | replace(".txt", ".bak"))  # Change extensions
.messages[] | map(. | split(" ") | length)       # Count words per message
```

### Field Selection

```bash
. | select_fields(name,age)     # Select multiple fields
. | select_fields(name,department,salary) # Custom field subset
```

### Aggregation

```bash
. | count                 # Count records
. | sum(.amount)          # Sum values
. | avg(.score)           # Average values
. | min(.price)           # Minimum value
. | max(.price)           # Maximum value
```

### Grouping

```bash
. | group_by(.category)               # Group by field
. | group_by(.department) | count     # Count by group
. | group_by(.region) | avg(.sales)   # Average by group
. | group_by(.type) | sum(.amount)    # Sum by group
```

### Complex Queries

```bash
# Multi-step analysis
.users[] | select(.age > 25) | group_by(.department) | avg(.salary)

# Multi-level array processing
.Reservations[].Instances[] | select(.State.Name == "running")

# Text processing pipeline (NEW!)
. | select(. | contains("ERROR")) | map(. | substring(11, 8)) | unique | sort

# Mixed data and text analysis
.logs[] | map(. | split(" ")[0]) | unique | sort  # Extract unique dates
```

## 🎯 Use Cases

### Log File Analysis (NEW in v0.2.0!)

```bash
# Extract error logs with timestamps
hawk '. | select(. | contains("ERROR")) | map(. | substring(0, 19))' app.log

# Analyze log levels
hawk '. | map(. | split(" ")[2]) | group_by(.) | count' app.log

# Find unique IP addresses in access logs
hawk '. | map(. | split(" ")[0]) | unique | sort' access.log

# Count warnings by hour
hawk '. | select(. | contains("WARN")) | map(. | substring(11, 2)) | group_by(.) | count' app.log
```

### Text Data Processing

```bash
# Clean and normalize text data
hawk '. | map(. | trim | lower)' names.txt

# Remove different types of whitespace
hawk '. | map(. | trim_start)' indented_text.txt    # Remove leading spaces
hawk '. | map(. | trim_end)' trailing_spaces.txt    # Remove trailing spaces

# Extract file extensions
hawk '. | map(. | split(".") | last)' filelist.txt

# Join processed data
hawk '. | map(. | split(",")) | map(. | join(" | "))' data.txt

# Count words in documents
hawk '. | map(. | split(" ") | length) | avg' documents.txt

# Find long lines
hawk '. | select(. | length > 80)' code.txt
```

### API Response Analysis

```bash
# Analyze GitHub API response
curl -s "https://api.github.com/users/kyotalab/repos" | hawk '.[] | select(.language == "Rust") | count'

# Extract specific fields
curl -s "https://api.github.com/users/kyotalab/repos" | hawk '.[] | .name' --format list
```

### DevOps & Infrastructure

```bash
# Kubernetes resource analysis
hawk '.items[] | select(.status.phase == "Running")' pods.json
hawk '.spec.template.spec.containers[0].image' deployment.yaml

# AWS EC2 analysis
hawk '.Reservations[].Instances[] | select(.State.Name == "running")' describe-instances.json
hawk '.Reservations[].Instances[] | group_by(.InstanceType) | count' ec2-data.json

# Docker Compose services
hawk '.services | info' docker-compose.yml
hawk '.services[] | select(.ports)' docker-compose.yml

# Configuration file analysis
hawk '. | select(. | starts_with("#") | not) | map(. | trim)' config.conf
```

### Data Analysis

```bash
# Sales data analysis
hawk '.sales | group_by(.region) | sum(.amount)' sales.csv
hawk '.transactions[] | select(.amount > 1000) | avg(.amount)' transactions.json

# Statistical analysis (NEW!)
hawk '.scores[] | median(.value)' scores.csv
hawk '.measurements[] | stddev(.temperature)' sensor_data.json

# Data cleaning and normalization
hawk '.users[] | map(.email | lower | trim)' users.csv
hawk '.products[] | map(.name | replace("_", " ") | upper)' inventory.json
```

## 📁 Supported Formats

### JSON

```json
{
  "users": [
    { "name": "Alice", "age": 30, "department": "Engineering" },
    { "name": "Bob", "age": 25, "department": "Marketing" }
  ]
}
```

### YAML

```yaml
users:
  - name: Alice
    age: 30
    department: Engineering
  - name: Bob
    age: 25
    department: Marketing
```

### CSV

```csv
name,age,department
Alice,30,Engineering
Bob,25,Marketing
```

### Plain Text (NEW in v0.2.0!)

```
2024-01-15 09:00:01 INFO Application started
2024-01-15 09:00:02 ERROR Failed to connect
2024-01-15 09:00:03 WARN High memory usage
```

All formats support the same query syntax!

## 🎨 Output Formats

### Smart Auto-Detection (default)

```bash
hawk '.users[0].name' data.json    # → Alice (list)
hawk '.users[]' data.json          # → Colored table format
hawk '.config' data.json           # → JSON format
```

### Explicit Format Control

```bash
hawk '.users[]' --format table     # Force table
hawk '.users[]' --format json      # Force JSON
hawk '.users.name' --format list   # Force list
```

### Colored Output (NEW in v0.2.0!)

- **Automatic TTY detection**: Colors in terminal, plain text in pipes
- **Beautiful tables**: Headers in blue, numbers in green, booleans in yellow
- **Readable JSON**: Syntax highlighting for better readability
- **NO_COLOR support**: Respects NO_COLOR environment variable

## 🛠️ Advanced Examples

### Complex Data Analysis

```bash
# Multi-step pipeline analysis
hawk '.employees[] | select(.salary > 50000) | group_by(.department) | avg(.salary)' payroll.csv

# Nested data exploration
hawk '.projects[].tasks[] | select(.status == "completed") | group_by(.assignee) | count' projects.json

# Cross-format analysis
hawk '.metrics[] | select(.value > 100) | sum(.value)' metrics.yaml
```

### Real-world Log Analysis (NEW!)

```bash
# Extract error timestamps and analyze patterns
hawk '. | select(. | contains("ERROR")) | map(. | substring(0, 10)) | group_by(.) | count' app.log

# Find most common error messages
hawk '. | select(. | contains("ERROR")) | map(. | split(":")[1] | trim) | group_by(.) | count' error.log

# Analyze response times from access logs
hawk '. | map(. | split(" ")[9]) | select(. | length > 0) | avg' access.log

# Extract unique user agents
hawk '. | map(. | split("\"")[5]) | unique | sort' access.log
```

### Text Processing Workflows

```bash
# 1. Clean configuration files
hawk '. | select(. | starts_with("#") | not) | map(. | trim)' config.txt

# 2. Analyze code files
hawk '. | select(. | trim | length > 0) | map(. | length) | avg' source.py

# 3. Process CSV-like text data
hawk '. | map(. | split(",")) | map(.[1] | trim)' data.txt

# 4. Extract and analyze patterns
hawk '. | select(. | contains("@")) | map(. | split("@")[1]) | group_by(.) | count' emails.txt
```

### Data Processing Workflows

```bash
# 1. Explore structure
hawk '. | info' unknown-data.json

# 2. Filter relevant data
hawk '.records[] | select(.timestamp >= "2024-01")' data.json

# 3. Clean and normalize (NEW!)
hawk '.users[] | map(.email | lower | trim)' users.csv

# 4. Statistical analysis (NEW!)
hawk '.sales[] | group_by(.region) | median(.amount)' sales.json

# 5. Export results
hawk '.summary[]' data.json --format csv > results.csv
```

## 🔧 Installation & Setup

### Homebrew (Recommended)

```bash
# Install via Homebrew
brew install kyotalab/tools/hawk

# Or install from the main repository
brew tap kyotalab/tools
brew install hawk
```

### Cargo (Rust Package Manager)

```bash
cargo install hawk-data
```

### Build from Source

```bash
# Prerequisites: Rust 1.70 or later
git clone https://github.com/kyotalab/hawk.git
cd hawk
cargo build --release

# Add to PATH
sudo cp target/release/hawk /usr/local/bin/
```

### Binary Releases

Download pre-built binaries from [GitHub Releases](https://github.com/kyotalab/hawk/releases)

- Linux (x86_64)
- macOS (Intel & Apple Silicon)

## 📚 Documentation

### Command Line Options

```bash
hawk --help              # Show help
hawk --version           # Show version
hawk '.query' file.json  # Basic usage
hawk '.query' --format json  # Specify output format
```

### Query Language Reference

| Operation               | Syntax              | Example                         |
| ----------------------- | ------------------- | ------------------------------- | ------------------- | --------------------------------- | -------- | ------------------- |
| Field access            | `.field`            | `.name`                         |
| Array index             | `.array[0]`         | `.users[0]`                     |
| Array iteration         | `.array[]`          | `.users[]`                      |
| Multi-level arrays      | `.array[].nested[]` | `.Reservations[].Instances[]`   |
| **Text processing**     | `.                  | map(.                           | operation)`         | `.                                | map(.    | upper)`             |
| **String filtering**    | `.                  | select(.                        | contains("text"))`  | `.                                | select(. | contains("ERROR"))` |
| **String manipulation** | `.                  | map(.                           | replace("a", "b"))` | `.                                | map(.    | trim)`              |
| Field selection         | `                   | select_fields(field1,field2)`   | `                   | select_fields(name,age)`          |
| Filtering               | `                   | select(.field > value)`         | `                   | select(.age > 30)`                |
| Nested filtering        | `                   | select(.nested.field == value)` | `                   | select(.State.Name == "running")` |
| Grouping                | `                   | group_by(.field)`               | `                   | group_by(.department)`            |
| Counting                | `                   | count`                          | `.users             | count`                            |
| Aggregation             | `                   | sum/avg/min/max(.field)`        | `                   | avg(.salary)`                     |
| **Statistics**          | `                   | median/stddev/unique/sort`      | `                   | median`                           |
| Info                    | `                   | info`                           | `.                  | info`                             |

### String Operations (NEW in v0.2.0!)

| Operation               | Syntax    | Description |
| ----------------------- | --------- | ----------- | ---------------------------------- | ----------------------------- |
| `upper`                 | `.        | map(.       | upper)`                            | Convert to uppercase          |
| `lower`                 | `.        | map(.       | lower)`                            | Convert to lowercase          |
| `trim`                  | `.        | map(.       | trim)`                             | Remove whitespace (both ends) |
| `trim_start`            | `.        | map(.       | trim_start)`                       | Remove leading whitespace     |
| `trim_end`              | `.        | map(.       | trim_end)`                         | Remove trailing whitespace    |
| `length`                | `.        | map(.       | length)`                           | Get string length             |
| `reverse`               | `.        | map(.       | reverse)`                          | Reverse string                |
| `contains(pattern)`     | `.        | select(.    | contains("text"))`                 | Check if contains pattern     |
| `starts_with(pattern)`  | `.        | select(.    | starts_with("pre"))`               | Check if starts with pattern  |
| `ends_with(pattern)`    | `.        | select(.    | ends_with("suf"))`                 | Check if ends with pattern    |
| `replace(old, new)`     | `.        | map(.       | replace("a", "b"))`                | Replace text                  |
| `substring(start, len)` | `.        | map(.       | substring(0, 5))`                  | Extract substring             |
| `split(delimiter)`      | `.        | map(.       | split(","))`                       | Split by delimiter            |
| `join(delimiter)`       | `.array[] | join(", ")` | Join array elements with delimiter |

### Statistical Operations (NEW in v0.2.0!)

| Operation | Syntax | Description |
| --------- | ------ | ----------- | ---------------------------- |
| `unique`  | `.     | unique`     | Remove duplicates            |
| `sort`    | `.     | sort`       | Sort values                  |
| `median`  | `.     | median`     | Calculate median             |
| `stddev`  | `.     | stddev`     | Calculate standard deviation |
| `length`  | `.     | length`     | Get array length             |

### Supported Operators

- **Comparison**: `>`, `<`, `==`, `!=`
- **Aggregation**: `count`, `sum`, `avg`, `min`, `max`
- **Statistics**: `median`, `stddev`, `unique`, `sort`, `length`
- **Grouping**: `group_by`
- **Filtering**: `select`
- **Transformation**: `map`

## 🆕 What’s New in v0.2.0

### 🎉 Major Features

- **Plain Text Support**: Process log files, configuration files, and any text data
- **String Operations**: Complete set of string manipulation functions
- **Statistical Functions**: Built-in median, standard deviation, unique, and sort operations
- **Enhanced map() Function**: Transform data with powerful string operations
- **Colored Output**: Beautiful, readable output with automatic TTY detection

### 🔧 Improvements

- Better error messages with detailed context
- Improved pipeline processing with proper parentheses handling
- Enhanced type inference for CSV data
- More robust file format detection

### 📊 New Use Cases Enabled

- Log file analysis and monitoring
- Text data cleaning and normalization
- Statistical analysis of numeric data
- Complex data transformation pipelines
- Configuration file processing

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
git clone https://github.com/kyotalab/hawk.git
cd hawk
cargo build
cargo test
```

### Running Tests

```bash
cargo test                    # Run all tests
```

## 📄 License

This project is licensed under the MIT License - see the <LICENSE> file for details.

## 🙏 Acknowledgments

- Inspired by the simplicity of `awk` and the power of `pandas`
- Built with the amazing Rust ecosystem
- Special thanks to the `serde`, `clap`, `csv`, and `termcolor` crate maintainers

## 🔗 Related Tools & Comparison

| Tool         | Best For                     | Limitations                                 | hawk Advantage                                                 |
| ------------ | ---------------------------- | ------------------------------------------- | -------------------------------------------------------------- |
| **awk**      | Text processing, log parsing | Line-based, no JSON/YAML support            | Structured data focus, type-aware operations, string functions |
| **jq**       | JSON transformation          | JSON-only, complex syntax for data analysis | Multi-format, pandas-like analytics, text processing           |
| **pandas**   | Heavy data science           | Requires Python setup, overkill for CLI     | Lightweight, terminal-native, instant startup                  |
| **sed/grep** | Text manipulation            | No structured data understanding            | Schema-aware processing, statistical functions                 |

### Why Choose hawk?

**🎯 For structured data analysis**, hawk fills the gap between simple text tools and heavy data science frameworks:

```bash
# awk: Limited structured data support
awk -F',' '$3 > 30 {print $1}' data.csv

# jq: JSON-only, verbose for analytics
jq '.[] | select(.age > 30) | .name' data.json

# hawk: Unified, intuitive syntax across all formats
hawk '.[] | select(.age > 30) | .name' data.csv   # Same syntax for CSV
hawk '.[] | select(.age > 30) | .name' data.json  # Same syntax for JSON
hawk '.[] | select(.age > 30) | .name' data.yaml  # Same syntax for YAML
hawk '. | select(. | contains("age=30"))' data.txt # Even works for text!
```

**🚀 pandas power, awk simplicity**:

```bash
# Complex analytics made simple
hawk '.sales | group_by(.region) | median(.amount)' sales.csv
# vs pandas: requires Python script with imports, DataFrame setup, etc.
```

**🔧 DevOps & log analysis optimized**:

```bash
# Kubernetes config analysis (YAML native)
hawk '.spec.containers[] | select(.resources.limits.memory)' deployment.yaml

# Log analysis (NEW in v0.2.0!)
hawk '. | select(. | contains("ERROR")) | map(. | substring(0, 19)) | unique' app.log
```

---

**Happy data exploring with hawk!** 🦅

For questions, issues, or feature requests, please visit our [GitHub repository](https://github.com/kyotalab/hawk).
