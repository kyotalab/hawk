# Getting Started with hawk 🦅

**5-minute introduction to hawk's data processing capabilities**

hawk is a command-line tool that lets you explore and analyze data using a simple, unified query language. Whether you're working with JSON APIs, CSV files, YAML configs, or log files, hawk uses the same intuitive syntax.

## 📦 Installation

Choose your preferred installation method:

### Homebrew (Recommended)

```bash
brew install kyotalab/tools/hawk
```

### Cargo (Rust)

```bash
cargo install hawk-data
```

### Verify Installation

```bash
hawk --version
# Output: hawk 0.2.2
```

## 🎯 Your First hawk Command

Let's start with a simple example. Create a test file:

```bash
cat << 'EOF' > users.json
{
  "users": [
    {"name": "Alice", "age": 30},
    {"name": "Bob", "age": 25}
  ]
}
EOF
```

Now run your first hawk command:

```bash
hawk '.users[0].name' users.json
```

**Output:** `Alice`

**What happened?**

- `.users` → access the "users" field
- `[0]` → get the first element of the array
- `.name` → get the "name" field from that element

## 🏗️ Basic Building Blocks

### 1. Field Access

```bash
# Access a field
hawk '.name' data.json

# Access nested fields
hawk '.user.profile.email' data.json

# Access array elements
hawk '.items[0]' data.json
```

### 2. Array Operations

```bash
# Get all array elements
hawk '.users[]' users.json

# Access specific fields from all elements
hawk '.users[].name' users.json
```

### 3. Filtering with select()

```bash
# Find users older than 25
hawk '.users[] | select(.age > 25)' users.json

# Find users named "Alice"
hawk '.users[] | select(.name == "Alice")' users.json
```

### 4. Counting and Aggregation

```bash
# Count total users
hawk '.users | count' users.json

# Average age
hawk '.users[] | avg(.age)' users.json
```

## 🧪 Hands-on Examples

Let's work through progressively complex examples with sample data.

### Example 1: JSON Data Analysis

Create a sample dataset:

```bash
cat > sales.json << 'EOF'
{
  "sales": [
    {"product": "Laptop", "price": 1200, "quantity": 3, "region": "North"},
    {"product": "Mouse", "price": 25, "quantity": 50, "region": "South"},
    {"product": "Keyboard", "price": 80, "quantity": 20, "region": "North"},
    {"product": "Monitor", "price": 300, "quantity": 10, "region": "South"}
  ]
}
EOF
```

**Basic Operations:**

```bash
# See all products
hawk '.sales[].product' sales.json

# Find expensive items (>$100)
hawk '.sales[] | select(.price > 100)' sales.json

# Count items by region
hawk '.sales[] | group_by(.region) | count' sales.json

# Average price by region
hawk '.sales[] | group_by(.region) | avg(.price)' sales.json
```

### Example 2: CSV Data Processing

Create a CSV file:

```bash
cat > employees.csv << 'EOF'
name,age,department,salary
Alice,30,Engineering,95000
Bob,25,Marketing,75000
Carol,35,Engineering,105000
David,28,Sales,80000
EOF
```

**CSV Operations:**

```bash
# See all names
hawk '.[].name' employees.csv

# Find engineers
hawk '.[] | select(.department == "Engineering")' employees.csv

# Average salary by department
hawk '.[] | group_by(.department) | avg(.salary)' employees.csv

# Count employees by department
hawk '.[] | group_by(.department) | count' employees.csv
```

### Example 3: Text/Log Processing

Create a sample log file:

```bash
cat > app.log << 'EOF'
2024-01-15 09:00:01 INFO Application started
2024-01-15 09:00:15 ERROR Database connection failed
2024-01-15 09:00:16 INFO Retrying connection
2024-01-15 09:01:20 WARN High memory usage: 85%
2024-01-15 09:01:45 ERROR Timeout occurred
EOF
```

**Text Processing Operations:**

```bash
# Process as text (use -t flag for logs)
# Find all ERROR lines
hawk -t '. | select(. | contains("ERROR"))' app.log

# Extract timestamps
hawk -t '. | map(. | split(" ")[0])' app.log

# Extract log levels
hawk -t '. | map(. | split(" ")[2])' app.log
```

## 🔧 String Operations

hawk includes powerful string manipulation:

```bash
# Text transformation
echo '"  Hello World  "' | hawk '. | map(. | trim | upper)'

# String splitting with index access (NEW!)
echo '"apple banana cherry"' | hawk '. | map(. | split(" ")[1])'

# Multiple field processing
cat << 'EOF' | hawk '. | map(.first, .last | upper)'
{
"first": "john",
"last": "doe"
}
EOF

```

## 📊 Understanding Output Formats

hawk automatically chooses the best output format:

```bash
# Single value → simple output
hawk '.users[0].name' users.json
# Output: Alice

# Array of objects → table format
hawk '.users[]' users.json
# Output: Formatted table with columns
```

You can force specific formats:

```bash
hawk '.users[]' --format json users.json    # Force JSON
hawk '.users[]' --format table users.json   # Force table
hawk '.users[].name' --format list users.json  # Force list
```

## 🎯 Common Patterns

### Data Exploration

```bash
# Understand data structure
hawk '. | info' unknown-data.json

# Count total records
hawk '. | count' data.json

# See unique values
hawk '.field_name[] | unique' data.json
```

### Filtering and Aggregation

```bash
# Filter → count pattern
hawk '.items[] | select(.price > 100) | count' data.json

# Group → aggregate pattern
hawk '.sales[] | group_by(.category) | sum(.amount)' data.json

# Filter → group → aggregate pattern
hawk '.orders[] | select(.status == "completed") | group_by(.region) | avg(.total)' data.json
```

### Text Processing

```bash
# Extract → unique pattern
hawk -t '. | map(. | split(" ")[0]) | unique' logs.txt

# Filter → extract pattern
hawk -t '. | select(. | contains("ERROR")) | map(. | split(" ")[1])' logs.txt

# Clean → transform pattern
hawk '.users[] | map(.email | lower | trim)' users.json
```

## 🚨 When to Use --text Flag

Use the `--text` flag when processing files that might be misdetected:

```bash
# For log files that look like YAML
hawk --text '. | select(. | contains("ERROR"))' structured.log

# For any text file you want to process line-by-line
hawk -t '. | map(. | length) | avg' text-file.txt
```

## 🎓 Next Steps

Now that you know the basics, explore these guides:

### **Immediate Next Steps**

1. **[String Operations Guide](string-operations.md)** - Master text processing
2. **[Query Language Reference](query-language.md)** - Complete syntax guide
3. **[Log Analysis Examples](examples/log-analysis.md)** - Real-world log processing

### **By Use Case**

- **Data Analysis**: [Data Analysis Guide](data-analysis.md)
- **DevOps**: [DevOps Workflows](examples/devops-workflows.md)
- **API Work**: [API Exploration](examples/api-exploration.md)

### **Advanced Topics**

- **Performance**: [Optimization Tips](advanced/performance.md)
- **Complex Workflows**: [Custom Workflows](advanced/custom-workflows.md)

## 🔗 Quick Reference Card

### Essential Commands

```bash
# Field access
hawk '.field' data.json
hawk '.array[0]' data.json
hawk '.array[]' data.json

# Filtering
hawk '.array[] | select(.field > value)' data.json

# Aggregation
hawk '.array[] | count/sum/avg/min/max(.field)' data.json

# Grouping
hawk '.array[] | group_by(.field) | count' data.json

# Text processing
hawk -t '. | select(. | contains("pattern"))' file.txt
hawk -t '. | map(. | split(" ")[0])' file.txt

# String operations
hawk '.field | upper/lower/trim/length' data.json
hawk '.field | split(",")[0]' data.json
```

### Data Types

- **JSON**: `data.json` → auto-detected
- **YAML**: `config.yaml` → auto-detected
- **CSV**: `data.csv` → auto-detected
- **Text**: `file.txt` → use `-t` flag for line processing

## 💡 Pro Tips

1. **Start Simple**: Begin with basic field access, then add complexity
2. **Use `info`**: Always start data exploration with `hawk '. | info' file`
3. **Test in Steps**: Build complex queries incrementally
4. **Use `--text`**: When in doubt with text files, use the `-t` flag
5. **Read Error Messages**: hawk provides helpful error context

## 🎉 You're Ready!

You now know enough hawk to be productive! The key is to start with simple operations and gradually build more complex queries as you become comfortable with the syntax.

**Remember**: hawk uses the same syntax across all data formats, so skills learned with JSON work with CSV, YAML, and text files.

Happy data exploring! 🦅

---

**Quick Links:**

- [String Operations](string-operations.md) - Text processing guide
- [Examples](../examples/README.md) - Real-world use cases
