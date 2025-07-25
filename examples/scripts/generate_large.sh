#!/bin/bash

# generate_large.sh - Generate large sample datasets for hawk testing
# Usage: ./generate_large.sh [options]

set -euo pipefail

# Default configuration
DEFAULT_SIZE=1000
DEFAULT_OUTPUT_DIR="examples/large"
DEFAULT_TYPE="all"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration variables
SIZE=$DEFAULT_SIZE
OUTPUT_DIR="$DEFAULT_OUTPUT_DIR"
TYPE="$DEFAULT_TYPE"
PARALLEL=false
STREAMING=false
VERBOSE=false

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

show_help() {
    cat <<EOF
Generate large sample datasets for hawk testing and learning.

Usage: $0 [OPTIONS]

Options:
    --size N            Number of records to generate (default: $DEFAULT_SIZE)
    --type TYPE         Type of data to generate: customers, orders, employees, 
                        logs, metrics, all (default: $DEFAULT_TYPE)
    --output DIR        Output directory (default: $DEFAULT_OUTPUT_DIR)
    --parallel          Enable parallel generation
    --streaming         Use streaming mode for large datasets
    --verbose           Enable verbose output
    --help              Show this help message

Examples:
    $0                                    # Generate all datasets with default size
    $0 --size 5000 --type customers      # Generate 5000 customer records
    $0 --size 10000 --parallel           # Generate all datasets in parallel
    $0 --type logs --size 50000          # Generate 50k log entries

Generated files will be placed in the output directory:
    - customers_large.json
    - orders_large.csv  
    - employees_large.json
    - logs_large.log
    - metrics_large.csv
    - user_behavior_large.json

EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
        --size)
            SIZE="$2"
            shift 2
            ;;
        --type)
            TYPE="$2"
            shift 2
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --parallel)
            PARALLEL=true
            shift
            ;;
        --streaming)
            STREAMING=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
        esac
    done
}

# Create output directory
setup_output_dir() {
    if [[ ! -d "$OUTPUT_DIR" ]]; then
        log_info "Creating output directory: $OUTPUT_DIR"
        mkdir -p "$OUTPUT_DIR"
    fi

    # Create .gitignore to exclude from git
    if [[ ! -f "$OUTPUT_DIR/.gitignore" ]]; then
        cat >"$OUTPUT_DIR/.gitignore" <<'EOF'
# Generated large datasets - exclude from git to avoid bloating repositories
*
!.gitignore
!README.md

# This directory contains large sample datasets generated by scripts/generate_large.sh
# These files are intentionally excluded from git to keep repositories lightweight
# Users can regenerate this data anytime using: ./scripts/generate_large.sh
EOF
        log_info "Created .gitignore in $OUTPUT_DIR"
    fi
}

# Generate customers dataset
generate_customers() {
    local size=$1
    local output_file="$OUTPUT_DIR/customers_large.json"

    log_info "Generating customers dataset ($size records)..."

    # Use Python for more realistic data generation
    python3 -c "
import json
import random
from datetime import datetime, timedelta

# Sample data pools
first_names = ['Alice', 'Bob', 'Carol', 'David', 'Elena', 'Frank', 'Grace', 'Henry', 'Irene', 'Jack', 'Kate', 'Liam', 'Maria', 'Nathan', 'Olivia', 'Peter', 'Quinn', 'Rachel', 'Steve', 'Tina']
last_names = ['Johnson', 'Smith', 'Wang', 'Brown', 'Rodriguez', 'Chen', 'Kim', 'Taylor', 'Davis', 'Miller', 'Wilson', 'Garcia', 'Anderson', 'Lee', 'Martinez', 'Thompson', 'White', 'Harris', 'Clark', 'Lewis']
countries = ['USA', 'Canada', 'UK', 'Germany', 'France', 'Japan', 'Australia', 'Brazil', 'India', 'Mexico']
segments = ['enterprise', 'business', 'small', 'startup']
statuses = ['active', 'inactive', 'suspended', 'deleted']
companies = ['TechCorp', 'GlobalSoft', 'InnovateLab', 'DataSystems', 'CloudWorks', 'StartupXYZ', 'Enterprise Inc', 'Solutions Ltd', 'Digital Corp', 'Future Tech']

customers = []
for i in range($size):
    customer_id = f'CUST{i+1:06d}'
    first_name = random.choice(first_names)
    last_name = random.choice(last_names)
    country = random.choice(countries)
    segment = random.choice(segments)
    status = random.choices(statuses, weights=[70, 15, 10, 5])[0]
    
    # Generate realistic lifetime value based on segment
    if segment == 'enterprise':
        ltv_base = random.uniform(10000, 100000)
    elif segment == 'business':
        ltv_base = random.uniform(5000, 25000)
    elif segment == 'small':
        ltv_base = random.uniform(1000, 8000)
    else:  # startup
        ltv_base = random.uniform(500, 5000)
    
    # Adjust by status
    if status == 'deleted':
        ltv_base *= random.uniform(0.1, 0.3)
    elif status == 'suspended':
        ltv_base *= random.uniform(0.3, 0.7)
    elif status == 'inactive':
        ltv_base *= random.uniform(0.5, 0.9)
    
    reg_date = datetime.now() - timedelta(days=random.randint(1, 1095))
    
    customer = {
        'id': customer_id,
        'name': f'{first_name} {last_name}',
        'email': f'{first_name.lower()}.{last_name.lower()}@{random.choice(companies).lower()}.com',
        'company': f'{random.choice(companies)} {random.choice([\"Inc\", \"Ltd\", \"Corp\", \"LLC\"])}',
        'country': country,
        'status': status,
        'lifetime_value': round(ltv_base, 2),
        'segment': segment,
        'registration_date': reg_date.strftime('%Y-%m-%d')
    }
    customers.append(customer)

with open('$output_file', 'w') as f:
    json.dump(customers, f, indent=2)
"

    log_success "Generated $output_file ($(wc -l <"$output_file") lines)"
}

# Generate orders dataset
generate_orders() {
    local size=$1
    local output_file="$OUTPUT_DIR/orders_large.csv"

    log_info "Generating orders dataset ($size records)..."

    python3 -c "
import csv
import random
from datetime import datetime, timedelta

# Sample data
statuses = ['completed', 'processing', 'shipped', 'cancelled', 'pending']
payment_methods = ['credit_card', 'paypal', 'bank_transfer', 'apple_pay']
products = [f'PROD{i:03d}' for i in range(1, 101)]

with open('$output_file', 'w', newline='') as f:
    writer = csv.writer(f)
    writer.writerow(['order_id', 'customer_id', 'product_id', 'quantity', 'price', 'order_date', 'status', 'payment_method'])
    
    for i in range($size):
        order_id = f'ORD{i+1:06d}'
        customer_id = f'CUST{random.randint(1, min(1000, $size // 5)):06d}'
        product_id = random.choice(products)
        quantity = random.randint(1, 5)
        price = round(random.uniform(10, 1000) * quantity, 2)
        order_date = (datetime.now() - timedelta(days=random.randint(0, 365))).strftime('%Y-%m-%d')
        status = random.choices(statuses, weights=[60, 15, 15, 7, 3])[0]
        payment_method = random.choice(payment_methods)
        
        writer.writerow([order_id, customer_id, product_id, quantity, price, order_date, status, payment_method])
"

    log_success "Generated $output_file ($(wc -l <"$output_file") lines)"
}

# Generate logs dataset
generate_logs() {
    local size=$1
    local output_file="$OUTPUT_DIR/logs_large.log"

    log_info "Generating logs dataset ($size lines)..."

    python3 -c "
import random
from datetime import datetime, timedelta

log_levels = ['DEBUG', 'INFO', 'WARN', 'ERROR', 'FATAL', 'CRITICAL']
level_weights = [50, 30, 10, 7, 2, 1]
components = ['main', 'worker-1', 'worker-2', 'worker-3', 'database', 'cache', 'auth', 'payment', 'scheduler']
messages = [
    'Application started successfully',
    'Processing request GET /api/v1/users',
    'Database connection established',
    'User login successful: user_id={}',
    'Order created: order_id={}',
    'Payment processed: amount=\${}',
    'Database connection timeout',
    'Authentication failed for user',
    'Rate limit exceeded',
    'Cache miss for key: {}',
    'Slow query detected: {}ms',
    'Memory usage high: {}%',
    'Disk space low: {}% remaining',
    'Service unavailable',
    'Request completed: {} ({} ms)'
]

start_time = datetime.now() - timedelta(hours=24)

with open('$output_file', 'w') as f:
    current_time = start_time
    for i in range($size):
        # Natural time progression
        current_time += timedelta(seconds=random.uniform(0.1, 30))
        
        level = random.choices(log_levels, weights=level_weights)[0]
        component = random.choice(components)
        message = random.choice(messages)
        
        # Add realistic parameters to messages
        if '{}' in message:
            if 'user_id' in message:
                message = message.format(random.randint(10000, 99999))
            elif 'order_id' in message:
                message = message.format(f'ORD{random.randint(1, 999999):06d}')
            elif '\${}' in message:
                message = message.format(round(random.uniform(10, 1000), 2))
            elif 'key:' in message:
                message = message.format(f'cache_key_{random.randint(1000, 9999)}')
            elif 'ms' in message and 'Slow' in message:
                message = message.format(random.randint(2000, 10000))
            elif '%' in message:
                message = message.format(random.randint(80, 95))
            elif 'Request completed' in message:
                method = random.choice(['GET', 'POST', 'PUT', 'DELETE'])
                endpoint = random.choice(['/api/v1/users', '/api/v1/orders', '/api/v1/products'])
                duration = random.randint(10, 500)
                message = message.format(f'{method} {endpoint}', duration)
        
        timestamp = current_time.strftime('%Y-%m-%d %H:%M:%S')
        log_line = f'{timestamp} {level:<8} [{component}] {message}'
        f.write(log_line + '\\n')
"

    log_success "Generated $output_file ($(wc -l <"$output_file") lines)"
}

# Generate metrics dataset
generate_metrics() {
    local size=$1
    local output_file="$OUTPUT_DIR/metrics_large.csv"

    log_info "Generating metrics dataset ($size records)..."

    python3 -c "
import csv
import random
import math
from datetime import datetime, timedelta

with open('$output_file', 'w', newline='') as f:
    writer = csv.writer(f)
    writer.writerow(['timestamp', 'server_id', 'cpu_percent', 'memory_percent', 'disk_percent', 'network_in_mb', 'network_out_mb', 'active_connections'])
    
    servers = [f'srv-web-{i:02d}' for i in range(1, 6)]
    start_time = datetime.now() - timedelta(hours=$size // 60)
    
    for i in range($size):
        timestamp = (start_time + timedelta(minutes=i)).strftime('%Y-%m-%dT%H:%M:%SZ')
        server_id = random.choice(servers)
        
        # Simulate daily patterns with some noise
        hour = (start_time + timedelta(minutes=i)).hour
        base_load = 30 + 40 * math.sin(math.pi * hour / 12)  # Daily cycle
        
        cpu_percent = max(5, min(95, base_load + random.uniform(-15, 15)))
        memory_percent = max(20, min(90, base_load * 0.8 + random.uniform(-10, 10)))
        disk_percent = max(10, min(80, 45 + random.uniform(-5, 5)))
        network_in = max(0, cpu_percent * random.uniform(0.5, 2.0))
        network_out = max(0, cpu_percent * random.uniform(0.3, 1.5))
        connections = int(max(0, cpu_percent * random.uniform(2, 8)))
        
        writer.writerow([
            timestamp, server_id, 
            round(cpu_percent, 1), round(memory_percent, 1), round(disk_percent, 1),
            round(network_in, 2), round(network_out, 2), connections
        ])
"

    log_success "Generated $output_file ($(wc -l <"$output_file") lines)"
}

# Generate user behavior dataset
generate_user_behavior() {
    local size=$1
    local output_file="$OUTPUT_DIR/user_behavior_large.json"

    log_info "Generating user behavior dataset ($size records)..."

    python3 -c "
import json
import random
from datetime import datetime, timedelta

actions = ['page_view', 'click', 'scroll', 'search', 'add_to_cart', 'purchase', 'login', 'logout']
pages = ['/home', '/products', '/cart', '/checkout', '/account', '/support', '/search', '/categories']
devices = ['desktop', 'mobile', 'tablet']
browsers = ['Chrome', 'Firefox', 'Safari', 'Edge']
countries = ['USA', 'Canada', 'UK', 'Germany', 'Japan', 'Australia', 'France', 'Brazil']

events = []
for i in range($size):
    session_id = f'sess_{random.randint(100000000, 999999999)}'
    user_id = f'user_{random.randint(1000, 9999)}'
    timestamp = (datetime.now() - timedelta(minutes=random.randint(0, 10080))).strftime('%Y-%m-%dT%H:%M:%SZ')
    
    event = {
        'session_id': session_id,
        'user_id': user_id,
        'timestamp': timestamp,
        'page': random.choice(pages),
        'action': random.choice(actions),
        'duration_seconds': random.randint(5, 300),
        'device': random.choice(devices),
        'browser': random.choice(browsers),
        'location': {
            'country': random.choice(countries),
            'city': f'City{random.randint(1, 100)}'
        }
    }
    events.append(event)

with open('$output_file', 'w') as f:
    json.dump(events, f, indent=2)
"

    log_success "Generated $output_file ($(wc -l <"$output_file") lines)"
}

# Generate employees dataset
generate_employees() {
    local size=$1
    local output_file="$OUTPUT_DIR/employees_large.json"

    log_info "Generating employees dataset ($size records)..."

    python3 -c "
import json
import random
from datetime import datetime, timedelta

first_names = ['Alice', 'Bob', 'Carol', 'David', 'Elena', 'Frank', 'Grace', 'Henry', 'Irene', 'Jack', 'Kate', 'Liam', 'Maria', 'Nathan', 'Olivia']
last_names = ['Johnson', 'Smith', 'Wang', 'Brown', 'Rodriguez', 'Chen', 'Kim', 'Taylor', 'Davis', 'Miller']
departments = ['Engineering', 'Sales', 'Marketing', 'HR', 'Finance', 'Operations', 'Design', 'DevOps']
roles = ['Senior Developer', 'Developer', 'Manager', 'Director', 'Analyst', 'Specialist', 'Lead', 'Engineer']
locations = ['San Francisco', 'New York', 'Chicago', 'Austin', 'Seattle', 'Remote', 'London', 'Toronto']
statuses = ['active', 'inactive', 'on_leave', 'terminated']

employees = []
for i in range($size):
    emp_id = f'EMP{i+1:06d}'
    first_name = random.choice(first_names)
    last_name = random.choice(last_names)
    department = random.choice(departments)
    role = f'{random.choice(roles)} {department.rstrip(\"s\")}'
    
    # Salary based on role level
    if 'Director' in role:
        salary = random.randint(140000, 200000)
    elif 'Manager' in role or 'Lead' in role:
        salary = random.randint(100000, 150000)
    elif 'Senior' in role:
        salary = random.randint(80000, 120000)
    else:
        salary = random.randint(50000, 90000)
    
    hire_date = (datetime.now() - timedelta(days=random.randint(30, 1095))).strftime('%Y-%m-%d')
    
    employee = {
        'id': emp_id,
        'name': f'{first_name} {last_name}',
        'email': f'{first_name.lower()}.{last_name.lower()}@company.com',
        'department': department,
        'role': role,
        'salary': salary,
        'hire_date': hire_date,
        'status': random.choices(statuses, weights=[85, 5, 7, 3])[0],
        'location': random.choice(locations)
    }
    employees.append(employee)

with open('$output_file', 'w') as f:
    json.dump(employees, f, indent=2)
"

    log_success "Generated $output_file ($(wc -l <"$output_file") lines)"
}

# Main generation function
generate_dataset() {
    local dataset_type=$1

    case $dataset_type in
    "customers")
        generate_customers $SIZE
        ;;
    "orders")
        generate_orders $SIZE
        ;;
    "employees")
        generate_employees $SIZE
        ;;
    "logs")
        generate_logs $SIZE
        ;;
    "metrics")
        generate_metrics $SIZE
        ;;
    "user_behavior")
        generate_user_behavior $SIZE
        ;;
    "all")
        if [[ "$PARALLEL" == "true" ]]; then
            log_info "Generating all datasets in parallel..."
            generate_customers $SIZE &
            generate_orders $((SIZE * 5)) &
            generate_employees $((SIZE / 2)) &
            generate_logs $((SIZE * 10)) &
            generate_metrics $((SIZE * 24)) &
            generate_user_behavior $((SIZE * 20)) &
            wait
        else
            generate_customers $SIZE
            generate_orders $((SIZE * 5))
            generate_employees $((SIZE / 2))
            generate_logs $((SIZE * 10))
            generate_metrics $((SIZE * 24))
            generate_user_behavior $((SIZE * 20))
        fi
        ;;
    *)
        log_error "Unknown dataset type: $dataset_type"
        exit 1
        ;;
    esac
}

# Check dependencies
check_dependencies() {
    if ! command -v python3 &>/dev/null; then
        log_error "python3 is required but not installed"
        exit 1
    fi

    log_info "Dependencies check passed"
}

# Display summary
show_summary() {
    log_info "Generation completed!"
    log_info "Output directory: $OUTPUT_DIR"

    if [[ -d "$OUTPUT_DIR" ]]; then
        log_info "Generated files:"
        ls -lh "$OUTPUT_DIR"/*.{json,csv,log} 2>/dev/null | while read -r line; do
            echo "  $line"
        done

        local total_size=$(du -sh "$OUTPUT_DIR" | cut -f1)
        log_info "Total size: $total_size"
    fi

    echo
    log_info "Example queries to try:"
    echo "  hawk '.[] | count' $OUTPUT_DIR/customers_large.json"
    echo "  hawk -t '. | select(. | contains(\"ERROR\")) | count' $OUTPUT_DIR/logs_large.log"
    echo "  hawk '.[] | group_by(.country) | count' $OUTPUT_DIR/customers_large.json"
    echo "  hawk '.[] | group_by(.department) | avg(.salary)' $OUTPUT_DIR/employees_large.json"
}

# Main function
main() {
    log_info "Starting large dataset generation..."
    log_info "Configuration: size=$SIZE, type=$TYPE, output=$OUTPUT_DIR"

    check_dependencies
    setup_output_dir
    generate_dataset "$TYPE"
    show_summary

    log_success "All done! Happy querying with hawk! 🦅"
}

# Parse arguments and run
parse_args "$@"

# Validate arguments
if ! [[ "$SIZE" =~ ^[0-9]+$ ]] || [[ "$SIZE" -lt 1 ]]; then
    log_error "Size must be a positive integer"
    exit 1
fi

if [[ "$SIZE" -gt 100000 ]]; then
    log_warning "Large size ($SIZE) may take significant time and disk space"
    read -p "Continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Cancelled by user"
        exit 0
    fi
fi

# Run main function
main
