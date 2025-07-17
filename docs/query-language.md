# Query Language Reference

Complete reference for hawk's query syntax and operations.

## 📖 Table of Contents

- [Query Structure](#query-structure)
- [Field Access](#field-access)
- [Pipeline Operations](#pipeline-operations)
- [Filtering](#filtering)
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

## Pipeline Operations

### Pipeline Syntax

Operations are chained with the pipe operator `|`:

```bash
<input> | <operation1> | <operation2> | <operation3>
```

### Operation Categories

1. **Filtering**: `select()`, text filters
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

#### Multiple Field Transformation (NEW in v0.2.2!)

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

# Text processing
hawk -t '. | map(. | trim | upper)' text.txt
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
split("delimiter")[index]    # Split and access specific element (NEW!)
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

# Splitting with index access (NEW!)
"apple,banana,cherry" | split(",")[1]   # → "banana"
"2024-01-15 10:30:00" | split(" ")[0]  # → "2024-01-15"
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

### Complex Filtering

```bash
# Multiple conditions (AND)
select(.age > 18 and .status == "active")

# Filter by nested conditions
select(.profile.preferences.notifications == true)

# Filter by array presence
select(.skills | length > 0)
```

### Multi-step Transformations

```bash
# Filter then transform
.users[] | select(.age > 30) | map(.name | upper)

# Transform then filter
.users[] | map(.email | lower) | select(. | ends_with(".com"))

# Group then analyze
.sales[] | group_by(.region) | map(.items | avg(.amount))
```

### Text Processing Workflows

```bash
# Log analysis pipeline
. | select(. | contains("ERROR")) | map(. | split(" ")[1]) | unique

# Data extraction pipeline
. | map(. | split(",")[0]) | select(. | length > 0) | unique | sort

# Cleaning pipeline
. | map(. | trim | upper) | select(. | starts_with("A"))
```

### Combining Operations

```bash
# Filter → group → aggregate
.orders[] | select(.status == "completed") | group_by(.region) | sum(.total)

# Transform → filter → count
.users[] | map(.email | lower) | select(. | ends_with("@company.com")) | count

# Multi-field → string ops → analysis
.logs[] | map(.timestamp | split("T")[0]) | group_by(.) | count
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

# ✅ Solution: check array length first
.users | length
```

#### Type Mismatches

```bash
# ❌ Error: applying numeric operation to string
.users[].name | sum

# ✅ Solution: use appropriate operation
.users[].name | length
```

### Debugging Techniques

#### Data Structure Exploration

```bash
# Understand data structure
. | info

# Check array lengths
.array_field | length

# Examine first element
.array_field[0]
```

#### Step-by-step Building

```bash
# Build query incrementally
.users[]                     # Step 1: get all users
.users[] | select(.age > 30) # Step 2: add filter
.users[] | select(.age > 30) | count  # Step 3: add aggregation
```

## Query Examples by Use Case

### Data Exploration

```bash
# Quick overview
. | info

# Count records
. | count

# Sample data
.[0:5]  # First 5 records

# Unique values
.field[] | unique
```

### API Response Analysis

```bash
# Extract specific data
.data[].id
.response.results[].title

# Filter by status
.items[] | select(.status == "active")

# Aggregate metrics
.analytics[] | sum(.views)
```

### Log File Analysis

```bash
# Find errors
. | select(. | contains("ERROR"))

# Extract timestamps
. | map(. | split(" ")[0])

# Count by log level
. | map(. | split(" ")[1]) | group_by(.) | count
```

### CSV Data Processing

```bash
# Column analysis
.[].column_name | unique

# Filtering records
.[] | select(.status == "active")

# Grouping analysis
.[] | group_by(.category) | avg(.value)
```

### Configuration File Analysis

```bash
# Check settings
.config.database.enabled

# List services
.services[].name

# Find configurations
.environments[] | select(.name == "production")
```

## Performance Tips

### Efficient Query Patterns

```bash
# ✅ Filter early in pipeline
.large_array[] | select(.condition) | expensive_operation

# ❌ Filter late in pipeline
.large_array[] | expensive_operation | select(.condition)
```

### Memory Considerations

```bash
# ✅ Process in chunks
.data[] | select(.relevant) | map(.transform)

# ❌ Transform everything
.data[] | map(.expensive_transform) | select(.relevant)
```

### String Operation Efficiency

```bash
# ✅ Use specific operations
.text | split(" ")[0]

# ❌ Use complex operations when simple ones suffice
.text | replace(...) | substring(...) | split(...)
```

---

**Related Documentation:**

- [Getting Started Guide](getting-started.md) - Quick introduction
- [String Operations](string-operations.md) - Detailed text processing
- [Examples](examples/) - Real-world use cases
- [Advanced Topics](advanced/) - Performance and optimization
