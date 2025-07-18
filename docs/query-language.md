# Query Language Reference

Complete reference for hawk's query syntax and operations.

## 📖 Table of Contents

- [Query Structure](#query-structure)
- [Field Access](#field-access)
- [Array Slicing](#array-slicing)
- [Pipeline Operations](#pipeline-operations)
- [Filtering](#filtering)
- [Logical Operations](#logical-operations)
- [Data Transformation](#data-transformation)
- [String Operations](#string-operations)
- [Statistical Operations](#statistical-operations)
- [Aggregation Functions](#aggregation-functions)
- [Grouping Operations](#grouping-operations)
- [Output Control](#output-control)
- [Advanced Patterns](#advanced-patterns)
- [Error Handling](#error-handling)

## Query Structure

### Basic Syntax

```
hawk '<query>' [file]
hawk '<query>' [options] [file]
```

### Pipeline Structure

```
<base_query> | <operation1> | <operation2> | ...
```

**Examples:**

```bash
# Simple field access
hawk '.users[0].name' data.json

# Pipeline with operations
hawk '.users[] | select(.age > 30) | count' data.json

# Complex pipeline
hawk '.logs[] | select(.level == "ERROR") | group_by(.service) | count' logs.json
```

## Field Access

### Object Fields

```bash
# Access top-level field
.field_name

# Access nested field
.parent.child.grandchild
```

**Examples:**

```bash
# Simple access
hawk '.name' user.json

# Nested access
hawk '.user.profile.email' data.json
```

### Array Access

#### Index Access

```bash
# Access specific element
.array[0]        # First element
.array[1]        # Second element
.array[-1]       # Last element (NEW!)
.array[-2]       # Second to last element (NEW!)
```

#### Array Iteration

```bash
# Access all elements
.array[]

# Access field from all elements
.array[].field_name

# Nested array access
.array[].nested_array[]
```

**Examples:**

```bash
# Get first user
hawk '.users[0]' data.json

# Get last user
hawk '.users[-1]' data.json

# Get all user names
hawk '.users[].name' data.json

# Get all project names from all users
hawk '.users[].projects[].name' data.json
```

### Root Access

```bash
# Access entire document
.

# Process each top-level element (for arrays)
.[]
```

## Array Slicing

### Basic Slicing Syntax

```bash
# Slice notation
.[start:end]     # Elements from start to end (exclusive)
.[start:]        # Elements from start to end
.[:end]          # Elements from beginning to end
.[:]             # All elements (copy)
```

### Negative Index Support

```bash
# Negative indices
.[-5:]           # Last 5 elements
.[:-3]           # All except last 3 elements
.[-10:-5]        # Elements from 10th-last to 5th-last
```

### Field-specific Slicing

```bash
# Slice specific arrays
.users[0:5]      # First 5 users
.logs[-100:]     # Last 100 log entries
.data[10:20]     # Elements 10-19
```

### Slicing with String Operations

```bash
# Split and slice results (NEW!)
.timestamp | split("-")[0:2]           # Get year and month
.path | split("/")[1:-1]               # Get middle path components
.csv_line | split(",")[2:5]            # Get columns 2-4
```

**Examples:**

```bash
# Basic slicing
hawk '.users[0:5]' users.json              # First 5 users
hawk '.logs[-50:]' logs.json               # Last 50 log entries
hawk '.data[10:20]' data.json              # Middle section

# Combined with operations
hawk '.scores[0:10] | avg(.)' scores.json  # Average of top 10 scores
hawk '.users[-5:] | count' users.json      # Count last 5 users

# String split slicing
hawk '.logs[] | .timestamp | split("T")[0]' logs.json          # Get date part
hawk '.files[] | .path | split("/")[-1]' files.json            # Get filename
hawk '.urls[] | split("://")[1] | split("/")[0]' urls.json     # Get domain

# Advanced slicing patterns
hawk '.data[] | .tags | split(",")[1:-1]' data.json            # Skip first and last tags
hawk '.logs[] | .message | split(" ")[2:] | join(" ")' logs.json  # Remove first 2 words
```

## Pipeline Operations

### Pipeline Syntax

Operations are chained with the pipe operator `|`:

```bash
<input> | <operation1> | <operation2> | <operation3>
```

### Operation Categories

1. **Filtering**: `select()`, text filters, logical operations
2. **Transformation**: `map()`, string operations
3. **Aggregation**: `count`, `sum()`, `avg()`, etc.
4. **Grouping**: `group_by()`
5. **Statistical**: `unique`, `sort`, `median`, `stddev`

## Filtering

### Basic Filtering with select()

```bash
# Numeric comparisons
select(.field > value)
select(.field < value)
select(.field == value)
select(.field != value)
select(.field >= value)
select(.field <= value)

# String comparisons
select(.field == "string")
select(.field != "string")

# Boolean comparisons
select(.field == true)
select(.field == false)
```

### Nested Field Filtering

```bash
# Filter by nested field
select(.parent.child > value)
select(.user.profile.age >= 18)
select(.config.database.enabled == true)
```

### String-based Filtering

```bash
# Text contains pattern
select(. | contains("pattern"))
select(.field | contains("text"))

# Text starts/ends with pattern
select(. | starts_with("prefix"))
select(.field | ends_with("suffix"))

# Case-insensitive filtering
select(. | upper | contains("PATTERN"))
```

**Examples:**

```bash
# Find users over 30
hawk '.users[] | select(.age > 30)' users.json

# Find active users
hawk '.users[] | select(.status == "active")' users.json

# Find users in Engineering
hawk '.users[] | select(.department == "Engineering")' users.json

# Find log entries containing "ERROR"
hawk -t '. | select(. | contains("ERROR"))' app.log

# Find files ending with .log
hawk '.files[] | select(.name | ends_with(".log"))' files.json
```

## Logical Operations

### NOT Operator

```bash
# NOT operator syntax (requires parentheses)
select(not (.condition))

# Examples
select(not (.age > 30))              # Users 30 or younger
select(not (.status == "active"))    # Inactive users
select(not (.email | contains("@gmail.com")))  # Non-Gmail users
```

### OR Operator (Pattern-based)

```bash
# OR using pipe-delimited patterns within contains()
select(.field | contains("pattern1|pattern2"))

# Multiple pattern matching
select(.status | contains("active|pending"))
select(.email | contains("@gmail.com|@company.com"))
select(.level | contains("ERROR|FATAL|CRITICAL"))
```

### Complex Logical Combinations

```bash
# NOT with string operations
select(not (.department | contains("Sales")))
select(not (.filename | ends_with(".tmp")))

# OR with pattern matching
select(.tag | contains("IMPORTANT|URGENT|CRITICAL"))
select(.file_type | contains("jpg|png|gif|svg"))
```

**Examples:**

```bash
# NOT operator examples
hawk '.users[] | select(not (.age > 65))' users.json           # Working age users
hawk '.files[] | select(not (.name | ends_with(".log")))' files.json  # Non-log files
hawk -t '. | select(not (. | contains("#")))' config.txt       # Non-comment lines

# OR operator examples
hawk '.users[] | select(.role | contains("admin|manager"))' users.json
hawk '.logs[] | select(.level | contains("ERROR|FATAL"))' logs.json
hawk '.files[] | select(.ext | contains("jpg|png|gif"))' files.json

# Combined logical operations
hawk '.users[] | select(not (.status | contains("deleted|suspended")))' users.json
hawk '.events[] | select(not (.type | contains("debug|trace")))' events.json

# Complex conditions with slicing
hawk '.logs[0:100] | select(not (.message | contains("INFO")))' logs.json
hawk '.users[-50:] | select(.email | contains("@company.com|@partner.com"))' users.json
```

## Data Transformation

### map() Function

#### Single Field Transformation

```bash
# Transform single field
map(.field | operation)

# Examples
map(.name | upper)           # Convert name to uppercase
map(.email | lower)          # Convert email to lowercase
map(.content | length)       # Get content length
```

#### Multiple Field Transformation

```bash
# Transform multiple fields with same operation
map(.field1, .field2 | operation)

# Examples
map(.first_name, .last_name | upper)      # Uppercase both names
map(.skills, .hobbies | join(","))        # Join both arrays
map(.title, .description | length)       # Get length of both fields
```

#### Root Element Transformation

```bash
# Transform entire element
map(. | operation)

# Examples for text processing
map(. | trim)                # Trim each line
map(. | split(" ")[0])       # Get first word from each line
map(. | upper)               # Convert each line to uppercase
```

**Examples:**

```bash
# Convert all names to uppercase
hawk '.users[] | map(.name | upper)' users.json

# Get email domains
hawk '.users[] | map(.email | split("@")[1])' users.json

# Process multiple fields
hawk '.users[] | map(.first_name, .last_name | upper)' users.json

# Text processing with slicing
hawk -t '. | map(. | split(" ")[1:] | join(" "))' text.txt    # Remove first word
hawk '.logs[] | map(.timestamp | split("T")[0])' logs.json   # Extract date part
```

### Field Selection

```bash
# Select specific fields
select_fields(field1,field2,field3)

# Examples
select_fields(name,age)              # Keep only name and age
select_fields(id,title,description)  # Keep only specified fields
```

## String Operations

### Case Conversion

```bash
upper                        # Convert to uppercase
lower                        # Convert to lowercase
```

### Whitespace Management

```bash
trim                         # Remove leading and trailing whitespace
trim_start                   # Remove leading whitespace only
trim_end                     # Remove trailing whitespace only
```

### String Analysis

```bash
length                       # Get string length
reverse                      # Reverse string
```

### Pattern Matching

```bash
contains("pattern")          # Check if string contains pattern
starts_with("prefix")        # Check if string starts with prefix
ends_with("suffix")          # Check if string ends with suffix
```

### Text Transformation

```bash
replace("old", "new")        # Replace text
substring(start, length)     # Extract substring
substring(start)             # Extract from start to end
```

### String Splitting and Joining

```bash
split("delimiter")           # Split string into array
split("delimiter")[index]    # Split and access specific element
split("delimiter")[start:end] # Split and slice result (NEW!)
join("delimiter")            # Join array elements into string
```

**Examples:**

```bash
# Basic string operations
"Hello World" | upper                    # → "HELLO WORLD"
"  text  " | trim                       # → "text"
"Hello World" | length                  # → 11

# Pattern matching
"Hello World" | contains("World")       # → true
"filename.txt" | ends_with(".txt")      # → true

# Text transformation
"Hello World" | replace("World", "Rust") # → "Hello Rust"
"Hello World" | substring(0, 5)         # → "Hello"

# Splitting with slicing (NEW!)
"apple,banana,cherry,date" | split(",")[1:3]    # → ["banana", "cherry"]
"2024-01-15 10:30:00" | split(" ")[0]          # → "2024-01-15"
"path/to/my/file.txt" | split("/")[-1]         # → "file.txt"
"one,two,three,four,five" | split(",")[::2]    # → ["one", "three", "five"] (future)
```

## Statistical Operations

### Basic Statistics

```bash
unique                       # Remove duplicates
sort                         # Sort values
length                       # Get array length
```

### Advanced Statistics

```bash
median                       # Calculate median
median(.field)              # Calculate median of field
stddev                      # Calculate standard deviation
stddev(.field)              # Calculate standard deviation of field
```

**Examples:**

```bash
# Get unique values
hawk '.users[].department | unique' users.json

# Sort values
hawk '.scores[] | sort' scores.json

# Calculate statistics
hawk '.measurements[] | median' data.json
hawk '.sales[] | stddev(.amount)' sales.json

# Combined with slicing
hawk '.scores[0:50] | median' scores.json        # Median of top 50 scores
hawk '.recent_data[-100:] | unique' data.json   # Unique values in last 100 entries
```

## Aggregation Functions

### Counting

```bash
count                        # Count elements
```

### Numeric Aggregation

```bash
sum(.field)                  # Sum numeric values
avg(.field)                  # Calculate average
min(.field)                  # Find minimum value
max(.field)                  # Find maximum value
```

### Field-specific Aggregation

```bash
# Apply to specific field
sum(.price)
avg(.score)
min(.temperature)
max(.response_time)
```

**Examples:**

```bash
# Count users
hawk '.users | count' users.json

# Calculate totals
hawk '.sales[] | sum(.amount)' sales.json

# Find averages
hawk '.students[] | avg(.grade)' grades.json

# Find extremes
hawk '.temperatures[] | min(.celsius)' weather.json
hawk '.response_times[] | max(.duration)' performance.json

# With slicing
hawk '.sales[0:30] | sum(.amount)' sales.json     # Sum first 30 sales
hawk '.scores[-100:] | avg(.)' scores.json        # Average of last 100 scores
```

## Grouping Operations

### Basic Grouping

```bash
group_by(.field)             # Group by field value
```

### Grouping with Aggregation

```bash
group_by(.field) | count     # Count items in each group
group_by(.field) | sum(.numeric_field)   # Sum by group
group_by(.field) | avg(.numeric_field)   # Average by group
group_by(.field) | min(.numeric_field)   # Minimum by group
group_by(.field) | max(.numeric_field)   # Maximum by group
```

**Examples:**

```bash
# Group users by department
hawk '.users[] | group_by(.department)' users.json

# Count by department
hawk '.users[] | group_by(.department) | count' users.json

# Average salary by department
hawk '.employees[] | group_by(.department) | avg(.salary)' employees.json

# Sales sum by region
hawk '.sales[] | group_by(.region) | sum(.amount)' sales.json

# Group with logical filtering
hawk '.users[] | select(not (.status == "deleted")) | group_by(.role) | count' users.json
```

## Output Control

### Format Options

```bash
--format auto               # Smart format detection (default)
--format table              # Force table output
--format json               # Force JSON output
--format list               # Force list output
```

### Text Processing Mode

```bash
--text, -t                  # Force text interpretation
```

**Examples:**

```bash
# Force specific output format
hawk '.users[]' --format table users.json
hawk '.users[].name' --format list users.json

# Process as text
hawk -t '. | select(. | contains("ERROR"))' app.log
```

## Advanced Patterns

### Complex Filtering with Logic

```bash
# Multiple NOT conditions
select(not (.age > 65)) and select(not (.status == "inactive"))

# OR with NOT combinations
select(.priority | contains("high|urgent")) and select(not (.archived == true))

# Complex string filtering with OR patterns
select(not (.filename | contains(".tmp|.bak|.swp")))
```

### Multi-step Transformations with Slicing

```bash
# Filter, slice, then transform
.users[] | select(.active == true) | .[0:10] | map(.name | upper)

# Transform, slice, then analyze
.logs[] | map(.timestamp | split("T")[0]) | .[-30:] | unique

# Slice grouped data
.sales[] | group_by(.region) | .[0:5] | sum(.total)
```

### Text Processing Workflows with Advanced Operations

```bash
# Complex log analysis
. | select(not (. | contains("DEBUG|INFO|TRACE"))) | map(. | split(" ")[1:] | join(" ")) | unique

# CSV processing with pattern matching
. | map(. | split(",")[2:5]) | select(not (.[0] | contains("null|empty|N/A")))

# Configuration analysis with OR patterns
. | select(not (. | starts_with("#"))) | select(. | contains("=|:")) | map(. | split("=|:")[0] | trim) | unique | sort
```

### Combining All Features

```bash
# Complex data pipeline
.events[-1000:] |
select(not (.type | contains("debug|trace|verbose"))) |
map(.timestamp | split("T")[0]) |
group_by(.) |
count

# Advanced text processing with OR patterns
.logs[] |
select(not (. | contains("INFO|DEBUG"))) |
map(. | split(" ")[2:] | join(" ") | substring(0, 100)) |
select(. | length > 10) |
unique[0:20]

# Multi-field analysis with pattern matching
.users[0:500] |
select(not (.role | contains("guest|inactive|test"))) |
map(.departments, .skills | join(",") | split(",") | length) |
group_by(.) |
count
```

## Error Handling

### Common Error Patterns

#### Field Not Found

```bash
# ❌ Error: field doesn't exist
.users[].nonexistent_field

# ✅ Solution: filter first
.users[] | select(.nonexistent_field) | .nonexistent_field
```

#### Index Out of Bounds

```bash
# ❌ Error: array index doesn't exist
.users[999].name

# ✅ Solution: use slicing safely
.users[0:1000] | .[999].name  # Returns empty if out of bounds
```

#### Slice Range Issues

```bash
# ❌ Error: invalid slice range
.array[10:5]   # End before start

# ✅ Solution: check bounds
.array | length  # Check array size first
.array[5:10]     # Ensure start < end
```

#### Logical Operation Errors

```bash
# ❌ Error: missing parentheses in NOT
select(not .field == "value")

# ✅ Solution: use proper syntax
select(not (.field == "value"))
```

### Debugging Techniques

#### Data Structure Exploration

```bash
# Understand data structure
. | info

# Check array lengths with slicing
.array_field | length
.array_field[0:5]    # Sample first 5 elements

# Examine specific ranges
.array_field[-10:]   # Last 10 elements
```

#### Step-by-step Building

```bash
# Build query incrementally
.users[]                                          # Step 1: get all users
.users[] | select(.age > 30)                     # Step 2: add filter
.users[] | select(.age > 30) | .[0:10]          # Step 3: add slicing
.users[] | select(.age > 30) | .[0:10] | count  # Step 4: add aggregation
```

## Query Examples by Use Case

### Data Exploration

```bash
# Quick overview
. | info

# Sample data with slicing
.[0:5]           # First 5 records
.[-3:]           # Last 3 records

# Unique values
.field[] | unique[0:10]  # Top 10 unique values
```

### API Response Analysis

```bash
# Extract specific data with limits
.data[0:100].id                    # First 100 IDs
.response.results[-50:].title      # Last 50 titles

# Filter by status with logic
.items[] | select(not (.status | contains("deleted|archived|suspended")))

# Aggregate metrics with slicing
.analytics[0:30] | sum(.views)     # Sum first 30 view counts
```

### Log File Analysis

```bash
# Find errors excluding debug info
. | select(.level | contains("ERROR|FATAL")) | select(not (. | contains("DEBUG|TRACE")))

# Extract timestamps with slicing
. | map(. | split(" ")[0:2] | join(" "))    # Get date and time parts

# Recent log analysis
.[-1000:] | select(. | contains("ERROR")) | count   # Count errors in last 1000 lines
```

### CSV Data Processing

```bash
# Column analysis with slicing
.[].column_name | unique[0:20]     # Top 20 unique values

# Filtering with logical operations
.[] | select(not (.status | contains("draft|deleted|suspended")))

# Multi-column processing
.[] | map(.data | split(",")[1:4] | join("|"))    # Extract columns 1-3, join with |
```

### Configuration File Analysis

```bash
# Non-comment, non-empty lines
. | select(not (. | starts_with("#"))) | select(not (. | trim | length == 0))

# Configuration sections
. | select(. | starts_with("[")) | map(. | substring(1, -1))

# Key-value extraction
. | select(. | contains("=")) | map(. | split("=")[0:2])
```

## Performance Tips

### Efficient Query Patterns

```bash
# ✅ Filter early, slice after
.large_array[] | select(.condition) | .[0:100] | expensive_operation

# ❌ Process everything then filter
.large_array[] | expensive_operation | select(.condition) | .[0:100]
```

### Memory Considerations with Slicing

```bash
# ✅ Process data in chunks
.data[0:1000] | select(.relevant) | map(.transform)
.data[1000:2000] | select(.relevant) | map(.transform)

# ❌ Load everything into memory
.data[] | map(.expensive_transform) | select(.relevant)
```

### Logical Operation Efficiency

```bash
# ✅ Use specific conditions early
select(.status == "active") | select(not (.archived == true))

# ❌ Complex logical operations on large datasets
select(not (.status | contains("deleted|archived|suspended|inactive")))
```

---

**Related Documentation:**

- [Getting Started Guide](getting-started.md) - Quick introduction
- [String Operations](string-operations.md) - Detailed text processing
- [Examples](examples/) - Real-world use cases
- [Advanced Topics](advanced/) - Performance and optimization
