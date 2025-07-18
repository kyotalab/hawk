# Data Analysis Guide

Comprehensive guide to data analysis workflows with hawk.

## 📖 Table of Contents

- [Data Analysis Fundamentals](#data-analysis-fundamentals)
- [Exploratory Data Analysis](#exploratory-data-analysis)
- [Statistical Operations](#statistical-operations)
- [Data Filtering and Selection](#data-filtering-and-selection)
- [Grouping and Aggregation](#grouping-and-aggregation)
- [Data Transformation](#data-transformation)
- [Time Series Analysis](#time-series-analysis)
- [Performance Analytics](#performance-analytics)
- [Business Intelligence](#business-intelligence)
- [Advanced Analytics Patterns](#advanced-analytics-patterns)

## Data Analysis Fundamentals

### The hawk Analytics Workflow

```bash
1. Data Exploration    → hawk '. | info' data.json
2. Data Cleaning      → hawk '.[] | select(.field) | map(.field | trim)'
3. Data Filtering     → hawk '.[] | select(.condition)'
4. Data Aggregation   → hawk '.[] | group_by(.field) | agg_function'
5. Results Export     → hawk '.results[]' --format csv > output.csv
```

### Understanding Your Data Structure

Before analysis, always understand your data:

```bash
# Get basic information
hawk '. | info' dataset.json

# Count total records
hawk '. | count' data.csv

# Sample the first few records
hawk '.[0:5]' data.json

# Check for missing values
hawk '.[] | select(.field == null) | count' data.json
```

## Exploratory Data Analysis

### Data Overview and Profiling

```bash
# Dataset summary
hawk '. | info' sales_data.json

# Record count by category
hawk '.[] | group_by(.category) | count' products.csv

# Unique values in a field
hawk '.[] | .department | unique' employees.json

# Data quality check
hawk '.[] | select(.email | contains("@")) | count' users.csv
```

### Sample Data Analysis Workflow

Let's work with a sample sales dataset:

```json
{
  "sales": [
    {
      "date": "2024-01-15",
      "product": "Laptop",
      "category": "Electronics",
      "amount": 1200,
      "quantity": 1,
      "region": "North",
      "salesperson": "Alice"
    },
    {
      "date": "2024-01-15",
      "product": "Mouse",
      "category": "Electronics",
      "amount": 25,
      "quantity": 3,
      "region": "South",
      "salesperson": "Bob"
    },
    {
      "date": "2024-01-16",
      "product": "Desk",
      "category": "Furniture",
      "amount": 300,
      "quantity": 2,
      "region": "North",
      "salesperson": "Alice"
    },
    {
      "date": "2024-01-16",
      "product": "Chair",
      "category": "Furniture",
      "amount": 150,
      "quantity": 4,
      "region": "South",
      "salesperson": "Carol"
    }
  ]
}
```

**Basic Analysis:**

```bash
# Total sales count
hawk '.sales[] | count' sales_data.json

# Total revenue
hawk '.sales[] | sum(.amount)' sales_data.json

# Average sale amount
hawk '.sales[] | avg(.amount)' sales_data.json

# Sales by category
hawk '.sales[] | group_by(.category) | sum(.amount)' sales_data.json

# Top performing regions
hawk '.sales[] | group_by(.region) | sum(.amount)' sales_data.json
```

## Statistical Operations

### Descriptive Statistics

```bash
# Central tendency
hawk '.[] | avg(.field)' data.json          # Mean
hawk '.[] | median(.field)' data.json       # Median

# Variability
hawk '.[] | min(.field)' data.json          # Minimum
hawk '.[] | max(.field)' data.json          # Maximum
hawk '.[] | stddev(.field)' data.json       # Standard deviation

# Distribution
hawk '.[] | .field | unique | sort' data.json  # Unique values
hawk '.[] | .field | sort' data.json           # All values sorted
```

### Advanced Statistical Analysis

```bash
# Quartile analysis (using slicing)
hawk '.[] | sort(.price) | length' products.json  # Get total count

# Range analysis
hawk '.[] | select(.price >= 100) | select(.price <= 500) | count' products.json

# Frequency analysis
hawk '.[] | group_by(.grade) | count' grades.json
```

### Statistical Comparisons

```bash
# Compare groups
hawk '.[] | group_by(.department) | avg(.salary)' employees.csv
hawk '.[] | group_by(.department) | stddev(.salary)' employees.csv

# Performance metrics
hawk '.[] | group_by(.team) | min(.response_time)' performance.json
hawk '.[] | group_by(.team) | max(.response_time)' performance.json

# Correlation analysis (manual)
hawk '.[] | select(.x > 0) | select(.y > 0) | count' correlation_data.json
```

## Data Filtering and Selection

### Conditional Filtering

```bash
# Numeric conditions
hawk '.[] | select(.age >= 18)' users.json              # Adults only
hawk '.[] | select(.price > 100) | select(.price <= 500)' products.json  # Price range

# String conditions
hawk '.[] | select(.status == "active")' accounts.json  # Active accounts
hawk '.[] | select(.email | ends_with(".com"))' users.json  # .com emails

# Date filtering (string-based)
hawk '.[] | select(.date >= "2024-01-01")' transactions.json
hawk '.[] | select(.date | starts_with("2024-01"))' logs.json
```

### Complex Multi-condition Filtering

```bash
# Multiple AND conditions
hawk '.[] | select(.age >= 18) | select(.status == "active") | select(.region == "North")' users.json

# Range filtering
hawk '.[] | select(.score >= 80) | select(.score <= 100)' grades.json

# Category filtering
hawk '.[] | select(.category == "Electronics") | select(.price < 1000)' products.json

# Data quality filtering
hawk '.[] | select(.email | contains("@")) | select(.phone | length == 10)' contacts.json
```

### Sampling and Data Selection

```bash
# Random sampling (using slicing)
hawk '.[] | .[0:100]' large_dataset.json              # First 100 records
hawk '.[] | .[1000:1100]' large_dataset.json          # Records 1000-1100

# Stratified sampling
hawk '.[] | group_by(.category) | .[0:10]' products.json  # 10 from each category

# Top/Bottom N
hawk '.[] | sort(.revenue) | .[-10:]' companies.json      # Top 10 by revenue
hawk '.[] | sort(.score) | .[0:5]' results.json           # Bottom 5 by score
```

## Grouping and Aggregation

### Basic Grouping Operations

```bash
# Group by single field
hawk '.[] | group_by(.department) | count' employees.json
hawk '.[] | group_by(.region) | sum(.sales)' sales.json
hawk '.[] | group_by(.category) | avg(.price)' products.json

# Group by multiple criteria (sequential)
hawk '.[] | group_by(.region) | group_by(.category) | sum(.amount)' sales.json
```

### Advanced Aggregation Patterns

```bash
# Multiple aggregations per group
hawk '.[] | group_by(.department)' employees.json  # Then analyze each group
hawk '.[] | group_by(.department) | count' employees.json     # Count per group
hawk '.[] | group_by(.department) | avg(.salary)' employees.json  # Average per group
hawk '.[] | group_by(.department) | sum(.salary)' employees.json  # Total per group

# Performance analytics
hawk '.[] | group_by(.server) | avg(.response_time)' performance.json
hawk '.[] | group_by(.server) | max(.memory_usage)' performance.json
hawk '.[] | group_by(.server) | min(.cpu_usage)' performance.json
```

### Business Intelligence Aggregations

```bash
# Sales analysis
hawk '.[] | group_by(.salesperson) | sum(.amount)' sales.json
hawk '.[] | group_by(.product) | avg(.rating)' reviews.json
hawk '.[] | group_by(.region) | count' customers.json

# Financial analysis
hawk '.[] | group_by(.quarter) | sum(.revenue)' financial.json
hawk '.[] | group_by(.cost_center) | sum(.expenses)' budget.json

# User behavior analysis
hawk '.[] | group_by(.user_type) | avg(.session_duration)' analytics.json
hawk '.[] | group_by(.device_type) | count' user_sessions.json
```

## Data Transformation

### Data Cleaning and Normalization

```bash
# Clean text data
hawk '.[] | map(.name | trim | upper)' contacts.json
hawk '.[] | map(.email | lower)' users.json

# Normalize numeric data
hawk '.[] | map(.amount | * 1.0)' transactions.json  # Ensure float

# Handle missing data
hawk '.[] | select(.field)' data.json                # Remove nulls
hawk '.[] | map(.field // "default_value")' data.json  # Replace nulls
```

### Feature Engineering

```bash
# Extract date components
hawk '.[] | map(.year | split("-")[0])' events.json
hawk '.[] | map(.month | split("-")[1])' events.json

# Categorize numeric data
hawk '.[] | select(.age >= 18) | select(.age < 65) | map(.age_group = "adult")' users.json
```

### Data Reshaping

```bash
# Extract specific fields
hawk '.[] | select_fields(id,name,email)' users.json
```

## Time Series Analysis

### Date-based Analysis

```bash
# Group by time periods
hawk '.[] | group_by(.date | split("-")[0])' time_series.json      # By year
hawk '.[] | group_by(.date | split("-")[1])' time_series.json      # By month
hawk '.[] | group_by(.date | substring(0, 7))' time_series.json    # By year-month

# Trend analysis
hawk '.[] | sort(.date) | .[0:10]' events.json     # First 10 chronologically
hawk '.[] | sort(.date) | .[-10:]' events.json     # Last 10 chronologically
```

### Sales Trend Analysis

```bash
# Monthly sales trends
hawk '.[] | group_by(.date | substring(0, 7)) | sum(.amount)' sales.json

# Daily transaction counts
hawk '.[] | group_by(.date) | count' transactions.json

# Seasonal analysis
hawk '.[] | group_by(.date | split("-")[1]) | avg(.temperature)' weather.json

# Growth analysis
hawk '.[] | sort(.date) | .[0:100]' historical_data.json  # Historical baseline
hawk '.[] | sort(.date) | .[-100:]' historical_data.json  # Recent data
```

### Performance Over Time

```bash
# System performance trends
hawk '.[] | group_by(.hour) | avg(.response_time)' performance_logs.json

# User engagement trends
hawk '.[] | group_by(.week) | sum(.active_users)' analytics.json

# Error rate analysis
hawk '.[] | group_by(.date) | select(.level == "ERROR") | count' error_logs.json
```

## Performance Analytics

### Application Performance Analysis

```bash
# Response time analysis
hawk '.[] | group_by(.endpoint) | avg(.response_time)' api_logs.json
hawk '.[] | group_by(.endpoint) | max(.response_time)' api_logs.json
hawk '.[] | group_by(.endpoint) | min(.response_time)' api_logs.json

# Error rate calculation
hawk '.[] | group_by(.service) | select(.status >= 400) | count' api_logs.json

# Throughput analysis
hawk '.[] | group_by(.hour) | count' requests.json
```

### System Resource Analysis

```bash
# Memory usage analysis
hawk '.[] | group_by(.server) | avg(.memory_usage)' system_metrics.json
hawk '.[] | group_by(.server) | max(.memory_usage)' system_metrics.json

# CPU utilization
hawk '.[] | group_by(.process) | avg(.cpu_percent)' process_metrics.json

# Disk usage trends
hawk '.[] | group_by(.mount_point) | max(.disk_usage)' disk_metrics.json
```

### User Performance Analysis

```bash
# Page load times
hawk '.[] | group_by(.page) | avg(.load_time)' user_metrics.json

# User session analysis
hawk '.[] | group_by(.user_id) | avg(.session_duration)' sessions.json

# Conversion rate analysis
hawk '.[] | group_by(.campaign) | select(.converted == true) | count' marketing.json
```

## Business Intelligence

### Sales Analytics

```bash
# Revenue analysis
hawk '.[] | group_by(.quarter) | sum(.revenue)' quarterly_sales.json
hawk '.[] | group_by(.product_line) | sum(.revenue)' product_sales.json
hawk '.[] | group_by(.region) | sum(.revenue)' regional_sales.json

# Profitability analysis
hawk '.[] | group_by(.product) | sum(.profit)' product_profitability.json
hawk '.[] | group_by(.customer_segment) | avg(.margin)' customer_analysis.json

# Sales performance
hawk '.[] | group_by(.salesperson) | sum(.deals_closed)' sales_performance.json
hawk '.[] | group_by(.salesperson) | avg(.deal_size)' sales_performance.json
```

### Customer Analytics

```bash
# Customer segmentation
hawk '.[] | group_by(.customer_type) | avg(.lifetime_value)' customers.json
hawk '.[] | group_by(.acquisition_channel) | count' customers.json

# Customer behavior
hawk '.[] | group_by(.customer_id) | sum(.total_spent)' transactions.json
hawk '.[] | group_by(.customer_id) | count' purchases.json

# Retention analysis
hawk '.[] | group_by(.cohort) | avg(.retention_rate)' retention.json
```

### Marketing Analytics

```bash
# Campaign performance
hawk '.[] | group_by(.campaign) | sum(.impressions)' marketing.json
hawk '.[] | group_by(.campaign) | avg(.click_through_rate)' marketing.json

# Channel effectiveness
hawk '.[] | group_by(.channel) | sum(.conversions)' marketing.json
hawk '.[] | group_by(.channel) | avg(.cost_per_acquisition)' marketing.json

# ROI analysis
hawk '.[] | group_by(.campaign) | sum(.revenue - .spend)' marketing.json
```

## Advanced Analytics Patterns

### Cohort Analysis

```bash
# User cohorts by signup month
hawk '.[] | group_by(.signup_month) | count' users.json
hawk '.[] | group_by(.signup_month) | avg(.lifetime_value)' users.json

# Retention by cohort
hawk '.[] | group_by(.cohort) | select(.active == true) | count' user_activity.json
```

### Funnel Analysis

```bash
# Conversion funnel
hawk '.[] | select(.stage == "awareness") | count' funnel.json
hawk '.[] | select(.stage == "consideration") | count' funnel.json
hawk '.[] | select(.stage == "purchase") | count' funnel.json

# Drop-off analysis
hawk '.[] | group_by(.exit_page) | count' user_sessions.json
```

### A/B Testing Analysis

```bash
# Test group comparison
hawk '.[] | group_by(.test_group) | avg(.conversion_rate)' ab_test.json
hawk '.[] | group_by(.test_group) | count' ab_test.json

# Statistical significance (basic)
hawk '.[] | group_by(.variant) | stddev(.metric)' ab_test.json
```

### Anomaly Detection (Basic)

```bash
# Outlier detection using statistical methods
hawk '.[] | sort(.value) | .[0:5]' data.json      # Bottom 5 (potential outliers)
hawk '.[] | sort(.value) | .[-5:]' data.json      # Top 5 (potential outliers)

# Threshold-based anomalies
hawk '.[] | avg(.response_time)' baseline.json    # Calculate baseline
hawk '.[] | select(.response_time > baseline * 2)' current.json  # 2x baseline
```

## Export and Reporting

### Data Export Formats

```bash
# Export to JSON
hawk '.summary' --format json > summary_report.json

# Export specific fields
hawk '.[] | select_fields(id,name,value)' --format table > report.txt
```

### Report Generation

```bash
# Summary statistics report
echo "=== Sales Summary ===" > report.txt
hawk '.[] | sum(.amount)' sales.json >> report.txt
hawk '.[] | avg(.amount)' sales.json >> report.txt
hawk '.[] | count' sales.json >> report.txt
```

## Best Practices

### Data Analysis Workflow

1. **Start with exploration**: Always use `hawk '. | info'` first
2. **Sample your data**: Use slicing `.[0:100]` for large datasets
3. **Check data quality**: Filter out invalid records early
4. **Build incrementally**: Add complexity step by step
5. **Validate results**: Cross-check with known values

### Performance Optimization

```bash
# ✅ Filter early in pipeline
hawk '.[] | select(.active == true) | group_by(.region) | count'

# ❌ Filter late in pipeline
hawk '.[] | group_by(.region) | select(.active == true) | count'

# ✅ Use appropriate data types
hawk '.[] | select(.amount > 100.0)' numeric_data.json

# ✅ Sample large datasets
hawk '.[0:1000] | group_by(.category) | avg(.price)' large_data.json
```

### Common Pitfalls

```bash
# ❌ Ignoring missing data
hawk '.[] | avg(.field)'  # May include nulls

# ✅ Handle missing data
hawk '.[] | select(.field) | avg(.field)'

# ❌ Not validating data types
hawk '.[] | sum(.text_field)'  # Error if not numeric

# ✅ Validate data types
hawk '.[] | select(.numeric_field > 0) | sum(.numeric_field)'
```

---

**Related Documentation:**

- [Getting Started](getting-started.md) - Basic introduction
- [Query Language Reference](query-language.md) - Complete syntax
- [String Operations](string-operations.md) - Text processing
- [Examples](examples/) - Real-world use cases
