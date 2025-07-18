# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2025-07-18

### 🎉 New Features

#### Logical Operations

- **NOT operator support**: Added `not` operator for negating conditions in `select()` statements

  - Syntax: `select(not (.condition))` - parentheses are required for clarity
  - Works with all comparison operators and string operations
  - Examples: `select(not (.age > 65))`, `select(not (.email | contains("spam")))`

- **OR operator support**: Added pipe-delimited pattern matching for OR conditions
  - Syntax: `select(.field | contains("pattern1|pattern2|pattern3"))`
  - Multi-pattern matching in a single operation
  - Examples: `select(.level | contains("ERROR|FATAL"))`, `select(.role | contains("admin|manager|supervisor"))`

#### Array Slicing Operations

- **Python-style slicing**: Complete slice notation support for arrays and string split results

  - Basic slicing: `.[start:end]`, `.[start:]`, `.[:end]`, `.[:]`
  - Negative index support: `.[-5:]`, `.[:-3]`, `.[-10:-5]`
  - Field-specific slicing: `.users[0:10]`, `.logs[-100:]`

- **String split slicing**: Advanced slicing support for string split operations

  - Direct slicing after split: `split(",")[1:3]`, `split("/")[-1]`
  - Complex path processing: `split("/")[1:-1]` for middle path components
  - CSV column extraction: `split(",")[2:5]` for specific column ranges

- **Negative indexing**: Support for negative array indices
  - Last element access: `.array[-1]`, `.users[-1]`
  - Reverse indexing: `.array[-3]` for third from last
  - Compatible with all array operations

#### Enhanced Filtering Capabilities

- **Complex logical combinations**: Combine NOT and OR operations for sophisticated filtering

  - Example: `select(not (.status | contains("deleted|suspended|inactive")))`
  - Multi-condition filtering: `select(not (.type | contains("debug|trace|verbose")))`

- **Pattern-based exclusion**: Exclude multiple patterns efficiently
  - File filtering: `select(not (.filename | contains(".tmp|.bak|.swp")))`
  - Content filtering: `select(not (. | contains("DEBUG|INFO|TRACE")))`

### 🔧 Improvements

#### Enhanced String Operations

- **Improved split operations**: Better integration with slicing for complex text processing
- **Optimized pattern matching**: More efficient OR pattern processing with pipe-delimited syntax
- **Better error handling**: Clearer error messages for logical operation syntax errors

#### Performance Optimizations

- **Slice operation efficiency**: Optimized memory usage for large array slicing operations
- **Pattern matching performance**: Improved performance for multi-pattern OR operations
- **Logical operation caching**: Better performance for complex NOT/OR combinations

#### Documentation and Examples

- **Comprehensive logical operation examples**: Real-world use cases for NOT and OR operations
- **Slicing operation guide**: Complete reference for all slicing capabilities
- **Advanced filtering patterns**: Examples combining multiple logical operations

### 📊 New Use Cases Enabled

#### Advanced Log Analysis

```bash
# Exclude multiple log levels efficiently
hawk '. | select(not (.level | contains("DEBUG|INFO|TRACE")))' app.log

# Get recent critical errors only
hawk '.logs[-1000:] | select(.level | contains("ERROR|FATAL|CRITICAL"))' system.log

# Extract specific time ranges with slicing
hawk '.logs[] | .timestamp | split("T")[0] | split("-")[1:3]' logs.json
```

#### Sophisticated Data Filtering

```bash
# Filter active users excluding test accounts
hawk '.users[] | select(not (.email | contains("test|demo|temp")))' users.json

# Find high-priority items excluding archived
hawk '.items[] | select(.priority | contains("high|urgent")) | select(not (.status | contains("archived|deleted")))' items.json

# Process middle sections of arrays
hawk '.data[100:200] | select(not (.type | contains("noise|test")))' dataset.json
```

#### Complex Text Processing

```bash
# Extract domain names efficiently
hawk '.urls[] | split("://")[1] | split("/")[0]' urls.txt

# Get file paths without extension
hawk '.files[] | .path | split(".")[:-1] | join(".")' filelist.json

# Process CSV columns with exclusions
hawk -t '. | split(",")[2:8] | select(not (.[0] | contains("null|empty|N/A")))' data.csv
```

#### Advanced Data Analysis Workflows

```bash
# Multi-step filtering with slicing
hawk '.events[0:5000] | select(not (.type | contains("debug|trace"))) | group_by(.service) | count' events.json

# Recent data analysis with pattern exclusion
hawk '.metrics[-500:] | select(not (.source | contains("test|staging"))) | avg(.value)' metrics.json

# Complex field extraction with logical operations
hawk '.users[] | select(.role | contains("admin|manager")) | select(not (.status | contains("inactive|suspended"))) | count' users.json
```

### 🛠️ Technical Improvements

#### Filter Module Enhancements

- **New function `parse_not_condition_with_parentheses`**: Robust NOT operator parsing with mandatory parentheses
- **Enhanced `apply_filter_with_string_operations`**: Support for NOT operator in string operation pipelines
- **Improved error handling**: Better error messages for missing parentheses and invalid syntax

#### Slicing Infrastructure

- **Universal slicing support**: `apply_universal_slice_operation` handles all data types
- **Negative index processing**: `parse_index_with_negative` for Python-style negative indexing
- **Data structure detection**: `detect_data_structure` for intelligent slicing behavior

#### Pattern Matching Optimization

- **OR pattern preprocessing**: Efficient pipe-delimited pattern parsing
- **Multi-pattern contains operations**: Optimized string matching for multiple patterns
- **Regex-free implementation**: Fast pattern matching without regex overhead

### 🔄 Breaking Changes

None. This release is fully backward compatible with v0.2.x.

### 🐛 Bug Fixes

- Fixed slice boundary checking for out-of-range indices
- Improved pattern matching edge cases with empty patterns
- Enhanced error reporting for malformed logical operations
- Fixed memory usage issues with large slice operations

### 📖 Documentation Updates

- **Complete logical operations reference**: Documentation for NOT and OR operators
- **Comprehensive slicing guide**: All slicing capabilities with examples
- **Advanced filtering patterns**: Real-world use case examples
- **Performance best practices**: Guidelines for efficient query construction

### 🚀 Migration Guide

#### For users upgrading from v0.2.1:

All existing queries continue to work without changes. New features are additive:

**New NOT operator usage:**

```bash
# Old approach (still works)
hawk '.users[] | select(.age <= 65)' users.json

# New approach with NOT operator
hawk '.users[] | select(not (.age > 65))' users.json
```

**New OR operator usage:**

```bash
# Old approach with multiple queries
hawk '.logs[] | select(.level == "ERROR")' logs.json
hawk '.logs[] | select(.level == "FATAL")' logs.json

# New approach with OR operator
hawk '.logs[] | select(.level | contains("ERROR|FATAL"))' logs.json
```

**New slicing capabilities:**

```bash
# Get last 10 users (new)
hawk '.users[-10:]' users.json

# Get middle section of data (new)
hawk '.data[100:200]' data.json

# Extract filename from path (new)
hawk '.files[] | .path | split("/")[-1]' files.json
```

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
