# Hawk Examples

This directory contains sample data and query examples to learn and explore all hawk features.

## 🚀 Quick Start

Lightweight sample data (~200KB total) ready to use immediately after git clone:

```bash
cd examples/small

# Explore data structure
hawk '. | info' customers.json

# Basic filtering
hawk '.[] | select(.status == "active")' customers.json

# New feature: NOT operator
hawk '.[] | select(not (.segment == "enterprise"))' customers.json

# New feature: OR operator
hawk -t '. | select(. | contains("ERROR|WARN"))' application.log

# New feature: Array slicing
hawk '.[]' customers.json | hawk '.[0:3]'
```

## 📁 Dataset Overview

### small/ - Lightweight Learning Data

| File                   | Size | Records  | Format | Use Case                                 |
| ---------------------- | ---- | -------- | ------ | ---------------------------------------- |
| `customers.json`       | ~2KB | 10       | JSON   | Customer management, basic queries       |
| `orders.csv`           | ~1KB | 25       | CSV    | Sales analysis, JOIN operations          |
| `products.yaml`        | ~1KB | 8        | YAML   | Product catalog, price analysis          |
| `employees.json`       | ~2KB | 15       | JSON   | HR data, grouping operations             |
| `ec2_instances.json`   | ~2KB | 5        | JSON   | AWS resources, infrastructure monitoring |
| `user_behavior.json`   | ~3KB | 20       | JSON   | Analytics, statistical processing        |
| `survey_responses.csv` | ~2KB | 30       | CSV    | Survey analysis, aggregation             |
| `application.log`      | ~3KB | 50 lines | TEXT   | Log analysis, error extraction           |
| `nginx_access.log`     | ~2KB | 30 lines | TEXT   | Web server logs, IP analysis             |
| `urls.txt`             | ~1KB | 20 lines | TEXT   | URL processing, domain extraction        |
| `error_messages.txt`   | ~1KB | 15 lines | TEXT   | Error categorization, pattern extraction |
| `nginx.conf`           | ~2KB | -        | TEXT   | Configuration file analysis              |

## 🎯 Learning Path

### Level 1: Basic Operations

```bash
# Understanding data structure
hawk '. | info' customers.json
hawk '.[] | count' customers.json

# Simple filtering
hawk '.[] | select(.country == "USA")' customers.json
hawk '.[] | select(.price > 100)' products.yaml
```

### Level 2: Aggregation and Grouping

```bash
# Aggregation functions
hawk '.[] | sum(.lifetime_value)' customers.json
hawk '.[] | avg(.price)' products.yaml

# Grouping
hawk '.[] | group_by(.country) | count' customers.json
hawk '.[] | group_by(.department) | avg(.salary)' employees.json
```

### Level 3: New Features (Logical Operations & Slicing)

```bash
# NOT operator
hawk '.[] | select(not (.status == "inactive"))' customers.json
hawk -t '. | select(not (. | contains("DEBUG")))' application.log

# OR operator
hawk '.[] | select(.segment | contains("enterprise|business"))' customers.json
hawk -t '. | select(. | contains("ERROR|FATAL|CRITICAL"))' application.log

# Array slicing
hawk '.[]' customers.json | hawk '.[0:5]'        # First 5 records
hawk '.[]' customers.json | hawk '.[-3:]'        # Last 3 records
```

### Level 4: Complex Text Processing

```bash
# Log analysis
hawk -t '. | map(. | split(" ")[0:3] | join(" "))' application.log
hawk -t '. | select(. | contains("ERROR")) | map(. | split(" ")[-1])' application.log

# URL processing
hawk -t '. | map(. | split("://")[1] | split("/")[0])' urls.txt
hawk -t '. | select(not (. | starts_with("https://")))' urls.txt

# Configuration file analysis
hawk -t '. | select(not (. | starts_with("#"))) | select(. | contains("="))' nginx.conf
```

### Level 5: Advanced Queries

```bash
# Multiple condition combinations
hawk '.[] | select(.status == "active") | select(not (.segment == "test")) | group_by(.country) | count' customers.json

# String operations with complex logic
hawk -t '. | select(. | contains("ERROR|WARN")) | map(. | split(" ")[0:2] | join(" ")) | unique' application.log

# Slicing with aggregation
hawk '.[]' user_behavior.json | hawk '.[0:10] | avg(.duration_seconds)'
```

## 🛠️ Larger Datasets

After mastering the basics with small sample data, practice with larger datasets:

```bash
# Generate large sample datasets (1000-10000 records)
./scripts/generate_large.sh

# Download real-world open datasets
./scripts/download_datasets.sh

# Practice with generated data
hawk '.[] | group_by(.country) | count' large/customers_large.json
```

## 📊 Practical Use Cases

### Business Analytics

```bash
# Customer segment analysis
hawk '.[] | group_by(.segment) | {segment: .group, avg_value: (.items | avg(.lifetime_value)), count: (.items | count)}' customers.json

# Sales trends (by month)
hawk '.[] | map(.order_date | split("-")[0:2] | join("-")) | group_by(.) | sum(.price)' orders.csv
```

### Infrastructure Monitoring

```bash
# Identify high-load instances
hawk '.[] | select(.cpu_utilization > 80)' ec2_instances.json

# Time-series error log analysis
hawk -t '. | select(. | contains("ERROR")) | map(. | split(" ")[0:2] | join(" ")) | group_by(.) | count' application.log
```

### Data Cleaning

```bash
# Detect duplicate email addresses
hawk '.[] | group_by(.email) | select(.items | count > 1)' customers.json

# Filter out invalid data
hawk '.[] | select(not (.email | contains("test|demo|temp"))) | select(.lifetime_value > 0)' customers.json
```

## 🔧 Scripts

### scripts/generate_large.sh

Generate larger sample datasets:

- `--size N`: Specify number of records to generate
- `--type TYPE`: Specify data type to generate
- `--format FORMAT`: Specify output format

### scripts/download_datasets.sh

Download real-world open datasets:

- GitHub API responses
- Public API sample data
- Real log file examples

See [scripts/README.md](scripts/README.md) for detailed documentation.

## 🎓 Next Steps

1. **Master the basics**: Try all features with small/ data
2. **Practice with real data**: Use scripts/ to generate larger datasets
3. **Apply to your projects**: Use hawk with your own data files

## 💡 Tips

- **Performance**: Apply filters early for large datasets
- **Debugging**: Use `| info` to inspect data structure
- **Incremental building**: Build complex queries step by step
- **Formatting**: Use `--format table` for readable output

## 🤝 Contributing

New sample data and query examples are welcome!

---

**Related Documentation:**

- [Query Language Reference](../docs/query-language.md)
- [Getting Started Guide](../docs/getting-started.md)
- [String Operations](../docs/string-operations.md)
