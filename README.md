# hawk 🦅

**Modern data analysis tool for JSON, YAML, CSV, and text files**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/hawk-data.svg)](https://crates.io/crates/hawk-data)

hawk combines the simplicity of `awk` with the power of `pandas`, bringing unified data processing to your command line. Process any data format with the same intuitive syntax.

## ⚡ Quick Start

### Installation

```bash
# Homebrew (macOS/Linux)
brew install kyotalab/tools/hawk

# Cargo (Rust)
cargo install hawk-data

# Verify installation
hawk --version
```

### 30-Second Demo

```bash
# JSON/CSV analysis - same syntax!
hawk '.users[] | select(.age > 30) | count' users.json
hawk '.[] | group_by(.department) | avg(.salary)' employees.csv

# Text/log processing with slicing (NEW!)
hawk -t '. | select(. | contains("ERROR|WARN")) | .[-100:]' app.log
hawk -t '. | map(. | split(" ")[0:3]) | unique' access.log

# Advanced string operations with multiple fields
hawk '.posts[] | map(.title, .content | trim | lower)' blog.json
hawk '.[] | group_by(.category) | .[0:10] | avg(.price)' products.json
```

## 🚀 Why hawk?

| Feature                  | hawk                       | jq               | awk           | pandas             |
| ------------------------ | -------------------------- | ---------------- | ------------- | ------------------ |
| **Multi-format**         | ✅ JSON, YAML, CSV, Text   | ❌ JSON only     | ❌ Text only  | ❌ Python required |
| **Unified syntax**       | ✅ Same queries everywhere | ❌ JSON-specific | ❌ Line-based | ❌ Complex setup   |
| **String operations**    | ✅ 14 built-in + slicing   | ⚠️ Limited       | ⚠️ Basic      | ✅ Extensive       |
| **Statistical analysis** | ✅ Built-in median, stddev | ❌ None          | ❌ None       | ✅ Full suite      |
| **Learning curve**       | 🟢 Familiar pandas-like    | 🟡 Steep         | 🟢 Simple     | 🔴 High            |

## 🎯 Key Features

### **Universal Data Processing**

Process any format with identical syntax:

```bash
hawk '.items[] | select(.price > 100)' data.json   # JSON
hawk '.items[] | select(.price > 100)' data.csv    # CSV
hawk '.items[] | select(.price > 100)' data.yaml   # YAML
hawk -t '. | select(. | contains("$"))' data.txt   # Text
```

### **Advanced Text Processing (NEW in v0.2.3!)**

```bash
# Split with slicing - extract exactly what you need
echo "2024-01-15 10:30:45 INFO message" | hawk -t '. | map(. | split(" ")[0:2])'
# → ["2024-01-15", "10:30:45"]

# OR conditions for flexible filtering
hawk -t '. | select(. | contains("GET|POST|PUT"))' access.log

# Powerful slicing for any operation result
hawk '.[] | sort(.revenue) | .[-10:]' companies.json  # Top 10
hawk '.[] | group_by(.category) | .[0:5]' products.json  # 5 from each group
```

### **Statistical Analysis Made Simple**

```bash
# Instant insights from your data
hawk '.sales[] | group_by(.region) | median(.amount)' sales.json
hawk '.users[] | select(.active) | stddev(.session_time)' analytics.json
hawk '.metrics[] | unique(.user_id) | count' engagement.json
```

## 📚 Documentation

### **Get Started in 5 Minutes**

- 🚀 [**Quick Start Guide**](docs/getting-started.md) - Essential basics
- 📖 [**Query Language Reference**](docs/query-language.md) - Complete syntax
- 🧵 [**String Operations**](docs/string-operations.md) - Text processing guide

### **Master Advanced Features**

- 📊 [**Data Analysis**](docs/data-analysis.md) - Statistical workflows
- 📄 [**Text Processing**](docs/text-processing.md) - Log analysis and text manipulation
- 💼 [**Real-world Examples**](docs/examples/) - Industry-specific use cases

### **Use Case Guides(In progress)**

- 🔍 [**Log Analysis**](docs/examples/log-analysis.md) - Docker, nginx, application logs
- ⚙️ [**DevOps Workflows**](docs/examples/devops-workflows.md) - Kubernetes, CI/CD, monitoring
- 📈 [**Data Science**](docs/examples/data-science.md) - CSV analysis, statistics, ML prep

## 🌟 Popular Workflows

### **Log Analysis**

```bash
# Find error patterns in application logs
hawk -t '. | select(. | contains("ERROR")) | map(. | split(" ")[0:2]) | unique' app.log

# Analyze Docker container performance
hawk -t '. | group_by(. | split(" ")[1]) | count' docker.log
```

### **Data Exploration**

```bash
# Quick dataset overview
hawk '. | info' unknown-data.json

# Statistical analysis
hawk '.users[] | group_by(.department) | median(.salary)' employees.csv
```

### **DevOps Automation**

```bash
# Kubernetes resource analysis
hawk '.items[] | select(.status.phase == "Running") | count' pods.json

# Performance monitoring
hawk '.metrics[] | group_by(.service) | avg(.response_time)' monitoring.json
```

## ⭐ What's New in v0.2.3

- **🎯 Advanced Slicing**: `.[0:10]`, `.[-5:]`, `group_by(.field) | .[0:3]`
- **✂️ Split with Slicing**: `split(" ")[0:3]`, `split(",")[-2:]`
- **🔍 OR Conditions**: `contains("GET|POST")`, `starts_with("ERROR|WARN")`
- **📊 Stratified Sampling**: Sample from each group for unbiased analysis
- **⚡ Performance**: Optimized for large datasets with efficient memory usage

## 🤝 Contributing

We welcome contributions! See our [Contributing Guide](CONTRIBUTING.md).

```bash
git clone https://github.com/kyotalab/hawk.git
cd hawk
cargo build --release
cargo test
```

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

**Ready to transform your data workflows?** Start with our [5-minute tutorial](docs/getting-started.md) 🚀
