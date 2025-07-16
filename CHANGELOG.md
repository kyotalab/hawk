# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2024-07-16

### 🐛 Bug Fixes
- Fixed single object field access (e.g., `.Parameters` in CloudFormation templates)
- Corrected info display for single objects ("Single Object" vs "Object Array")
- Enhanced support for YAML/JSON single object files

### 🔧 Improvements
- Better error messages for field access
- Improved CloudFormation, Docker Compose, Kubernetes manifest support

## [0.2.0] - 2025-07-16

### 🎉 Major Features Added

#### Plain Text Support

- **Universal file format support**: Now processes plain text files, log files, configuration files, and any text-based data
- **Automatic format detection**: Intelligently detects JSON, YAML, CSV, and plain text files
- **Unified query syntax**: Same query language works across all supported formats
- **Text-as-array processing**: Each line becomes a string element in an array for consistent processing

#### String Operations

- **Complete string manipulation suite**: `upper`, `lower`, `trim`, `trim_start`, `trim_end`
- **String analysis functions**: `length`, `reverse`
- **Pattern matching**: `contains(pattern)`, `starts_with(pattern)`, `ends_with(pattern)`
- **Text transformation**: `replace(old, new)`, `substring(start, length)`
- **String parsing**: `split(delimiter)` to convert strings to arrays
- **Array joining**: `join(delimiter)` to convert arrays back to strings

#### Enhanced map() Function

- **Data transformation pipeline**: Transform data elements with chained string operations
- **Type-safe operations**: Proper error handling for incompatible data types
- **Complex pipelines**: Support for multi-step transformations like `map(. | trim | upper | replace("old", "new"))`

#### Statistical Functions

- **Descriptive statistics**: `median`, `stddev` (standard deviation)
- **Data manipulation**: `unique` (remove duplicates), `sort` (sort values)
- **Array operations**: `length` for counting elements
- **Field-specific operations**: All statistical functions support field specification (e.g., `median(.price)`)

#### Colored Output

- **Automatic TTY detection**: Colors in terminal, plain text when piped or redirected
- **Beautiful syntax highlighting**:
  - Table headers in blue with bold formatting
  - Numbers in green
  - Boolean values in yellow
  - Null values in gray
  - JSON syntax highlighting with colored keys and values
- **Environment variable support**: Respects `NO_COLOR` environment variable
- **Multiple output formats**: Enhanced table, JSON, and list outputs with appropriate coloring

### 🔧 Improvements

#### Enhanced Error Handling

- **Detailed error messages**: Context-aware error reporting with specific field and operation information
- **Type-safe operations**: Better validation of operations on different data types
- **Pipeline debugging**: Improved error location reporting in complex query pipelines

#### Better File Format Detection

- **Robust detection algorithms**: Improved heuristics for distinguishing between formats
- **Edge case handling**: Better support for malformed or ambiguous files
- **Fallback mechanisms**: Graceful degradation to text processing when format detection fails

#### Performance Optimizations

- **Memory-efficient processing**: Optimized data structures for large datasets
- **Faster pipeline execution**: Improved query parsing and execution engine
- **Reduced startup time**: Optimized initialization and dependency loading

#### Pipeline Processing Improvements

- **Parentheses-aware parsing**: Proper handling of nested operations like `map(. | contains("text") | not)`
- **Complex query support**: Better support for multi-level operations and transformations
- **Operation chaining**: Improved reliability of long pipeline chains

### 📊 New Use Cases Enabled

#### Log File Analysis

```bash
# Extract error logs with timestamps
hawk '. | select(. | contains("ERROR")) | map(. | substring(0, 19))' app.log

# Count log levels
hawk '. | map(. | split(" ")[2]) | group_by(.) | count' application.log

# Find unique IP addresses
hawk '. | map(. | split(" ")[0]) | unique | sort' access.log
```

#### Text Data Processing

```bash
# Clean and normalize text
hawk '. | map(. | trim | lower)' names.txt

# Extract file extensions
hawk '. | map(. | split(".") | last)' filelist.txt

# Statistical text analysis
hawk '. | map(. | split(" ") | length) | median' documents.txt
```

#### Data Cleaning and Normalization

```bash
# Email normalization
hawk '.users[] | map(.email | lower | trim)' users.csv

# Complex string transformations
hawk '.products[] | map(.name | replace("_", " ") | upper)' inventory.json

# Data validation and cleaning
hawk '.records[] | select(.id | length == 8) | map(.status | upper)' data.csv
```

#### Advanced Analytics

```bash
# Statistical analysis
hawk '.measurements[] | group_by(.sensor) | stddev(.temperature)' sensor_data.json

# Median calculations
hawk '.sales[] | group_by(.region) | median(.amount)' sales_data.csv

# Unique value analysis
hawk '.users[] | unique(.department) | sort' employee_data.json
```

### 🛠️ Technical Improvements

#### New Dependencies

- `termcolor ^1.4`: For colored output support
- `is-terminal ^0.4`: For TTY detection

#### Code Architecture

- **New module `string_ops`**: Centralized string operation handling
- **New module `stats_ops`**: Statistical function implementations
- **Enhanced `filter.rs`**: Improved pipeline operation handling
- **Updated `output.rs`**: Comprehensive colored output support
- **Improved `setup.rs`**: Better file format detection and text processing

#### Testing

- **Comprehensive test suite**: Added tests for all new string operations
- **Statistical function testing**: Validation of median, stddev, and other statistical operations
- **Integration testing**: End-to-end testing of complex pipeline operations
- **Edge case coverage**: Testing of malformed inputs and error conditions

### 🔄 Breaking Changes

None. This release is fully backward compatible with v0.1.x.

### 📦 Migration Guide

No migration required. All existing queries and workflows continue to work unchanged.

### 🐛 Bug Fixes

- Fixed pipeline parsing issues with complex nested operations
- Improved CSV type inference accuracy
- Enhanced error reporting for malformed queries
- Fixed memory usage issues with large datasets

### 📖 Documentation Updates

- **Comprehensive README update**: Added extensive documentation for new features
- **String operations guide**: Complete reference for all string manipulation functions
- **Statistical functions documentation**: Usage examples and parameter descriptions
- **Text processing examples**: Real-world use cases for log analysis and text processing
- **Enhanced query syntax reference**: Updated with all new operations and examples

### 🙏 Acknowledgments

- Community feedback on string processing needs
- Performance suggestions from early adopters
- Documentation improvements from user contributions

## [0.1.0] - 2024-07-12

### 🎉 Initial Release

#### Core Features

- **Multi-format support**: JSON, YAML, CSV parsing and processing
- **Pandas-like query language**: Intuitive syntax for data analysis
- **Field access and navigation**: Deep nested field access with array expansion
- **Filtering operations**: `select()` with comparison operators
- **Aggregation functions**: `count`, `sum`, `avg`, `min`, `max`
- **Grouping operations**: `group_by()` with aggregation support
- **Multiple output formats**: Table, JSON, list with automatic format detection

#### Technical Foundation

- **Rust implementation**: Fast, memory-safe data processing
- **serde_json integration**: Robust JSON parsing and manipulation
- **Type-aware processing**: Intelligent handling of numbers, strings, booleans
- **Error handling**: Comprehensive error reporting with thiserror
- **CLI interface**: User-friendly command-line interface with clap

#### Supported Operations

- Field access: `.field`, `.array[0]`, `.array[]`, `.nested.field`
- Filtering: `select(.field > value)`, `select(.field == "value")`
- Aggregation: `sum(.field)`, `avg(.field)`, `min(.field)`, `max(.field)`, `count`
- Grouping: `group_by(.field)` with aggregation support
- Info: `. | info` for data structure exploration

#### Output Formats

- **Table format**: Structured table output for object arrays
- **JSON format**: Pretty-printed JSON output
- **List format**: Simple list output for array data
- **Auto format**: Intelligent format selection based on data structure

---

For more details about any release, please see the [GitHub releases page](https://github.com/kyotalab/hawk/releases).
