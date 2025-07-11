# hawk 🦅

Modern data analysis tool for structured data (JSON, YAML, CSV)

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**hawk** combines the simplicity of `awk` with the power of `pandas` for data exploration. Unlike traditional text tools that work line-by-line, hawk understands structured data natively. Unlike heavy data science tools that require complex setup, hawk brings analytics to your terminal with a single command.

**Perfect for**:
- 📊 **Data Scientists**: Quick CSV/JSON analysis without Python overhead
- 🔧 **DevOps Engineers**: Kubernetes YAML, Docker Compose, Terraform analysis
- 🌐 **API Developers**: REST response exploration and validation
- 📈 **Business Analysts**: Instant insights from structured datasets

## ✨ Features

- 🔍 **Multi-format support**: JSON, YAML, CSV with automatic detection (vs jq's JSON-only)
- 🐼 **Pandas-like operations**: Filtering, grouping, aggregation (vs awk's line-based processing)
- 📊 **Smart output formatting**: Tables, lists, JSON based on data structure
- 🚀 **Fast and lightweight**: Built in Rust for performance (vs pandas' Python overhead)
- 🔧 **Developer-friendly**: Perfect for DevOps, data analysis, and API exploration
- 🎯 **Type-aware**: Understands numbers, strings, booleans (vs text tools' string-only approach)
- 🔄 **Unified syntax**: Same query language across all formats (vs format-specific tools)

## 🚀 Quick Start

### Installation

```bash
# Clone and build
git clone https://github.com/kyotalab/hawk.git
cd hawk
cargo build --release

# Add to PATH
cp target/release/hawk /usr/local/bin/
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
```

## 📖 Query Syntax

### Field Access
```bash
.field                    # Access field
.array[0]                 # Access array element
.array[]                  # Access all array elements
.nested.field             # Deep field access
.array[0].nested.field    # Complex nested access
```

### Filtering
```bash
. | select(.age > 30)           # Numeric comparison
. | select(.name == "Alice")    # String equality
. | select(.active == true)     # Boolean comparison
. | select(.status != "inactive") # Not equal
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

# Data exploration workflow
.data | info                          # Understand structure
.data[] | select(.status == "active") # Filter active records
.data[] | group_by(.category) | count # Count by category
```

## 🎯 Use Cases

### API Response Analysis
```bash
# Analyze GitHub API response
curl -s "https://api.github.com/users/octocat/repos" | hawk '.[] | select(.language == "JavaScript") | count'

# Extract specific fields
curl -s "https://api.github.com/users/octocat/repos" | hawk '.[] | .name' --format list
```

### DevOps & Infrastructure
```bash
# Kubernetes resource analysis
hawk '.items[] | select(.status.phase == "Running")' pods.json
hawk '.spec.template.spec.containers[0].image' deployment.yaml

# Docker Compose services
hawk '.services | info' docker-compose.yml
hawk '.services[] | select(.ports)' docker-compose.yml
```

### Data Analysis
```bash
# Sales data analysis
hawk '.sales | group_by(.region) | sum(.amount)' sales.csv
hawk '.transactions[] | select(.amount > 1000) | avg(.amount)' transactions.json

# Log analysis
hawk '.logs[] | select(.level == "ERROR") | count' app-logs.json
hawk '.events | group_by(.source) | count' events.yaml
```

### Configuration Management
```bash
# Ansible inventory analysis
hawk '.all.children | info' inventory.yml
hawk '.all.hosts[] | select(.ansible_host)' inventory.yml

# Terraform state analysis
hawk '.resources[] | group_by(.type) | count' terraform.tfstate
```

## 📁 Supported Formats

### JSON
```json
{
  "users": [
    {"name": "Alice", "age": 30, "department": "Engineering"},
    {"name": "Bob", "age": 25, "department": "Marketing"}
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

All formats support the same query syntax!

## 🎨 Output Formats

### Smart Auto-Detection (default)
```bash
hawk '.users[0].name' data.json    # → Alice (list)
hawk '.users[]' data.json          # → Table format
hawk '.config' data.json           # → JSON format
```

### Explicit Format Control
```bash
hawk '.users[]' --format table     # Force table
hawk '.users[]' --format json      # Force JSON
hawk '.users.name' --format list   # Force list
```

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

### Real-world DevOps Scenarios
```bash
# Find all running containers with high memory usage
hawk '.containers[] | select(.memory_usage > 512) | .name' docker-stats.json

# Analyze Kubernetes deployments by namespace
hawk '.items[] | group_by(.metadata.namespace) | count' deployments.json

# Extract configuration errors from logs
hawk '.logs[] | select(.level == "ERROR" && .source == "config")' app.json
```

### Data Processing Workflows
```bash
# 1. Explore structure
hawk '. | info' unknown-data.json

# 2. Filter relevant data
hawk '.records[] | select(.timestamp >= "2024-01")' data.json

# 3. Group and analyze
hawk '.records[] | group_by(.category) | avg(.score)' data.json

# 4. Export results
hawk '.summary[]' data.json --format csv > results.csv
```

## 🔧 Installation & Setup

### Prerequisites
- Rust 1.70 or later
- Git

### Build from Source
```bash
git clone https://github.com/kyotalab/hawk.git
cd hawk
cargo build --release
```

### Add to PATH
```bash
# Linux/macOS
sudo cp target/release/hawk /usr/local/bin/

# Or add to your shell profile
echo 'export PATH="$PATH:/path/to/hawk/target/release"' >> ~/.bashrc
```

## 📚 Documentation

### Command Line Options
```bash
hawk --help              # Show help
hawk --version           # Show version
hawk '.query' file.json  # Basic usage
hawk '.query' --format json  # Specify output format
```

### Query Language Reference

| Operation | Syntax | Example |
|-----------|--------|---------|
| Field access | `.field` | `.name` |
| Array index | `.array[0]` | `.users[0]` |
| Array iteration | `.array[]` | `.users[]` |
| Filtering | `\| select(.field > value)` | `\| select(.age > 30)` |
| Grouping | `\| group_by(.field)` | `\| group_by(.department)` |
| Counting | `\| count` | `.users \| count` |
| Aggregation | `\| sum/avg/min/max(.field)` | `\| avg(.salary)` |
| Info | `\| info` | `. \| info` |

### Supported Operators
- **Comparison**: `>`, `<`, `==`, `!=`
- **Aggregation**: `count`, `sum`, `avg`, `min`, `max`
- **Grouping**: `group_by`
- **Filtering**: `select`

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
cargo test --test integration # Run integration tests
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by the simplicity of `awk` and the power of `pandas`
- Built with the amazing Rust ecosystem
- Special thanks to the `serde`, `clap`, and `csv` crate maintainers

## 🔗 Related Tools & Comparison

| Tool | Best For | Limitations | hawk Advantage |
|------|----------|-------------|----------------|
| **awk** | Text processing, log parsing | Line-based, no JSON/YAML support | Structured data focus, type-aware operations |
| **jq** | JSON transformation | JSON-only, complex syntax for data analysis | Multi-format, pandas-like analytics |
| **pandas** | Heavy data science | Requires Python setup, overkill for CLI | Lightweight, terminal-native |
| **sed/grep** | Text manipulation | No structured data understanding | Schema-aware processing |

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
```

**🚀 pandas power, awk simplicity**:
```bash
# Complex analytics made simple
hawk '.sales | group_by(.region) | avg(.amount)' sales.csv
# vs pandas: requires Python script with imports, DataFrame setup, etc.
```

**🔧 DevOps & IaC optimized**:
```bash
# Kubernetes config analysis (YAML native)
hawk '.spec.containers[] | select(.resources.limits.memory)' deployment.yaml
# vs jq: requires YAML→JSON conversion first
```

---

**Happy data exploring with hawk!** 🦅

For questions, issues, or feature requests, please visit our [GitHub repository](https://github.com/kyotalab/hawk).