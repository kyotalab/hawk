# Sample Data Generation Scripts

This directory contains scripts to generate larger sample datasets for hawk learning and testing.

## 📋 Scripts Overview

### generate_large.sh

Generate larger sample datasets (1,000-10,000 records) for performance testing and advanced learning.

**Usage:**

```bash
# Generate all datasets with default settings
./scripts/generate_large.sh

# Specify custom size
./scripts/generate_large.sh --size 5000

# Generate specific data types only
./scripts/generate_large.sh --type customers
./scripts/generate_large.sh --type logs
./scripts/generate_large.sh --type metrics

# Specify output directory
./scripts/generate_large.sh --output examples/large
```

**Options:**

- `--size N`: Number of records to generate (default: 1000)
- `--type TYPE`: Data type to generate (customers, orders, employees, logs, metrics, all)
- `--output DIR`: Output directory (default: examples/large)
- `--format FORMAT`: Output format (json, csv, yaml)
- `--parallel`: Enable parallel generation for faster processing
- `--help`: Show help message

### download_datasets.sh

Download real-world open datasets for practicing with actual data patterns.

**Usage:**

```bash
# Download all available datasets
./scripts/download_datasets.sh

# Download specific datasets only
./scripts/download_datasets.sh --dataset github
./scripts/download_datasets.sh --dataset logs
```

**Available Datasets:**

- `github`: GitHub API responses (repositories, users, issues)
- `apis`: Public API samples (REST, GraphQL responses)
- `logs`: Real application logs from open source projects
- `configs`: Configuration file samples (nginx, docker, k8s)

### cleanup.sh

Clean up generated data files and temporary files.

```bash
# Clean all generated files with confirmation
./scripts/cleanup.sh

# Clean specific targets
./scripts/cleanup.sh --target large
./scripts/cleanup.sh --target external

# Preview what would be deleted (dry run)
./scripts/cleanup.sh --dry-run
```

## 🎯 Generated Data

### Large Dataset (examples/large/)

| File                       | Records              | Size (approx) | Use Case                              |
| -------------------------- | -------------------- | ------------- | ------------------------------------- |
| `customers_large.json`     | 1,000-10,000         | 200KB-2MB     | Customer analysis, segmentation       |
| `orders_large.csv`         | 5,000-50,000         | 500KB-5MB     | Sales analysis, trend analysis        |
| `employees_large.json`     | 500-5,000            | 100KB-1MB     | HR analysis, organizational analysis  |
| `logs_large.log`           | 10,000-100,000 lines | 1MB-10MB      | Log analysis, error analysis          |
| `metrics_large.csv`        | 1,440-14,400         | 100KB-1MB     | Time series analysis, monitoring data |
| `user_behavior_large.json` | 10,000-100,000       | 2MB-20MB      | Behavior analysis, A/B testing        |

### External Dataset (examples/external/)

| File                    | Source                | Size   | Use Case                            |
| ----------------------- | --------------------- | ------ | ----------------------------------- |
| `github_repos.json`     | GitHub API            | ~50KB  | Real API response processing        |
| `public_apis.json`      | Public APIs Directory | ~100KB | API data analysis                   |
| `real_logs.log`         | Open source projects  | ~500KB | Real log analysis                   |
| `config_samples.tar.gz` | Configuration samples | ~200KB | Config analysis, pattern extraction |

## 🚀 Usage Examples

### Performance Testing with Large Data

```bash
# Generate large customer dataset for aggregation testing
./scripts/generate_large.sh --type customers --size 10000
hawk '.[] | group_by(.country) | count' large/customers_large.json

# Test filtering performance with large logs
./scripts/generate_large.sh --type logs --size 100000
hawk -t '. | select(. | contains("ERROR|CRITICAL")) | count' large/logs_large.log

# Time series analysis with metrics data
./scripts/generate_large.sh --type metrics --size 14400
hawk '.[] | group_by(.hour) | avg(.cpu_usage)' large/metrics_large.csv
```

### Practicing with Real Data

```bash
# Practice with GitHub data
./scripts/download_datasets.sh --dataset github
hawk '.items[] | select(.language == "Rust") | group_by(.owner.login) | count' external/github_repos.json

# Real log error analysis
./scripts/download_datasets.sh --dataset logs
hawk -t '. | select(. | contains("ERROR|FATAL")) | map(. | split(" ")[0:3] | join(" ")) | unique' external/real_logs.log
```

## ⚙️ Script Details

### Data Generation Algorithms

**customers_large.json:**

- Random but realistic names, emails, countries
- Realistic company names and segment distribution
- Regional purchasing power reflected in lifetime_value
- Distribution adjusted by country population

**logs_large.log:**

- Chronological natural log generation
- Realistic ERROR/WARN/INFO ratios (1:5:20)
- Correlated IP addresses, URLs, response codes
- Mimics real application patterns

**metrics_large.csv:**

- 24 hours × days of time series data
- CPU, memory, network correlation relationships
- Load variation patterns by time of day
- Weekend/weekday differences reflected

### Performance Considerations

- **Parallel Generation**: Simultaneous generation of multiple files for speed
- **Memory Efficiency**: Streaming generation for large capacity support
- **Progress Display**: Real-time progress indication
- **Error Handling**: Proper handling of generation failures

## 🛠️ Customization

### Custom Data Generation

Create your own data patterns based on the scripts:

```bash
# Copy template
cp scripts/generate_large.sh scripts/generate_custom.sh

# Implement custom data patterns
# Add generate_custom_dataset() function
```

### Configuration File

Customize generation parameters with `scripts/config.yaml`:

```yaml
generation:
  default_size: 1000
  output_dir: "examples/large"

datasets:
  customers:
    countries: ["USA", "Canada", "UK", "Germany", "Japan"]
    segments: ["enterprise", "business", "small"]

  logs:
    log_levels: ["ERROR", "WARN", "INFO", "DEBUG"]
    level_ratios: [1, 5, 20, 50]
```

## 🧹 Cleanup

### Safe Deletion

```bash
# Deletion with confirmation
./scripts/cleanup.sh --interactive

# Specific files only
./scripts/cleanup.sh --pattern "*.log"

# Size-limited deletion
./scripts/cleanup.sh --size-limit 10MB
```

### Automated Cleanup

```bash
# Periodic cleanup of old files (cron example)
0 2 * * * /path/to/scripts/cleanup.sh --older-than 7days
```

## 💡 Tips

1. **Progressive Learning**: Learn in order: small → large → external
2. **Memory Monitoring**: Watch memory usage when generating large datasets
3. **Disk Space**: 10,000 records require approximately 10-50MB
4. **Parallel Processing**: Test performance with multiple concurrent queries

## 🐛 Troubleshooting

### Common Issues

**Slow generation:**

```bash
# Enable parallel generation
./scripts/generate_large.sh --parallel 4

# Adjust size
./scripts/generate_large.sh --size 1000
```

**Out of memory errors:**

```bash
# Use streaming mode
./scripts/generate_large.sh --streaming

# Adjust batch size
./scripts/generate_large.sh --batch-size 100
```

**Download failures:**

```bash
# Enable retry
./scripts/download_datasets.sh --retry 3

# Set proxy
export https_proxy=http://proxy.company.com:8080
./scripts/download_datasets.sh
```

---

**Related Documentation:**

- [Main Examples README](../README.md)
- [Query Language Reference](../../docs/query-language.md)

## 🛠️ Customization

### Custom Data Generation

スクリプトをベースに独自のデータパターンを作成：

```bash
# Copy template
cp scripts/generate_large.sh scripts/generate_custom.sh

# Add custom data patterns
# generate_custom_dataset() 関数を実装
```

### Configuration File

Generate custom parameters in `scripts/config.yaml`：

```yaml
generation:
  default_size: 1000
  output_dir: "examples/large"

datasets:
  customers:
    countries: ["USA", "Canada", "UK", "Germany", "Japan"]
    segments: ["enterprise", "business", "small"]

  logs:
    log_levels: ["ERROR", "WARN", "INFO", "DEBUG"]
    level_ratios: [1, 5, 20, 50]
```

## 🧹 Cleanup

### Safe Deletion

```bash
# Deletion with confirmation
./scripts/cleanup.sh --interactive

# Specific files only
./scripts/cleanup.sh --pattern "*.log"

# Size-limited deletion
./scripts/cleanup.sh --size-limit 10MB
```

### Automated Cleanup

```bash
# Periodic cleanup of old files (cron example)
0 2 * * * /path/to/scripts/cleanup.sh --older-than 7days
```

## 💡 Tips

1. **Progressive Learning**: Learn in order: small → large → external
2. **Memory Monitoring**: Watch memory usage when generating large datasets
3. **Disk Space**: 10,000 records require approximately 10-50MB
4. **Parallel Processing**: Test performance with multiple concurrent queries

## 🐛 Troubleshooting

### Common Issues

**Slow generation:**

```bash
# Enable parallel generation
./scripts/generate_large.sh --parallel 4

# Adjust size
./scripts/generate_large.sh --size 1000
```

**Out of memory errors:**

```bash
# Use streaming mode
./scripts/generate_large.sh --streaming

# Adjust batch size
./scripts/generate_large.sh --batch-size 100
```

**Download failures:**

```bash
# Enable retry
./scripts/download_datasets.sh --retry 3

# Set proxy
export https_proxy=http://proxy.company.com:8080
./scripts/download_datasets.sh
```

---

**Related Documentation:**

- [Main Examples README](../README.md)
- [Query Language Reference](../../docs/query-language.md)
