#!/bin/bash

# download_datasets.sh - Download real-world datasets for hawk practice
# Usage: ./download_datasets.sh [options]

set -euo pipefail

# Default configuration
DEFAULT_OUTPUT_DIR="examples/external"
DEFAULT_DATASET="all"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration variables
OUTPUT_DIR="$DEFAULT_OUTPUT_DIR"
DATASET="$DEFAULT_DATASET"
FORCE=false
RETRY_COUNT=3

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
Download real-world datasets for hawk practice and advanced learning.

Usage: $0 [OPTIONS]

Options:
    --dataset TYPE      Dataset to download: github, apis, logs, configs, all (default: $DEFAULT_DATASET)
    --output DIR        Output directory (default: $DEFAULT_OUTPUT_DIR)
    --force             Overwrite existing files
    --retry N           Number of retry attempts (default: $RETRY_COUNT)
    --help              Show this help message

Available Datasets:
    github              GitHub API responses (repositories, users, issues)
    apis                Public API samples (REST, GraphQL responses)
    logs                Real application logs from open source projects
    configs             Configuration file samples (nginx, docker, k8s)
    all                 Download all available datasets

Examples:
    $0                                    # Download all datasets
    $0 --dataset github                   # Only GitHub data
    $0 --dataset logs --force             # Force download logs
    $0 --output /tmp/data --retry 5       # Custom output with retries

Downloaded datasets are perfect for:
    - Learning complex queries with real data
    - Performance testing with actual data patterns
    - Understanding real-world data structures

EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
        --dataset)
            DATASET="$2"
            shift 2
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --force)
            FORCE=true
            shift
            ;;
        --retry)
            RETRY_COUNT="$2"
            shift 2
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

# Check dependencies
check_dependencies() {
    local missing_deps=()

    if ! command -v curl &>/dev/null; then
        missing_deps+=("curl")
    fi

    if ! command -v jq &>/dev/null; then
        log_warning "jq not found - JSON formatting will be skipped"
    fi

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_error "Please install missing tools and try again"
        exit 1
    fi

    log_info "Dependencies check passed"
}

# Setup output directory
setup_output_dir() {
    if [[ ! -d "$OUTPUT_DIR" ]]; then
        log_info "Creating output directory: $OUTPUT_DIR"
        mkdir -p "$OUTPUT_DIR"
    fi

    # Create .gitignore
    if [[ ! -f "$OUTPUT_DIR/.gitignore" ]]; then
        echo "*" >"$OUTPUT_DIR/.gitignore"
        echo "!.gitignore" >>"$OUTPUT_DIR/.gitignore"
        echo "!README.md" >>"$OUTPUT_DIR/.gitignore"
        log_info "Created .gitignore in $OUTPUT_DIR"
    fi

    # Create README
    cat >"$OUTPUT_DIR/README.md" <<'EOF'
# External Datasets

This directory contains real-world datasets downloaded for hawk practice.

## Datasets

- `github_*.json` - GitHub API responses
- `public_apis.json` - Public APIs directory
- `real_logs.log` - Application logs from open source projects
- `config_samples/` - Configuration file samples

## Usage Examples

```bash
# Analyze GitHub repositories by language
hawk '.items[] | group_by(.language) | count' github_repos.json

# Find API endpoints by category
hawk '.entries[] | group_by(.Category) | count' public_apis.json

# Extract error patterns from logs
hawk -t '. | select(. | contains("ERROR")) | map(. | split(" ")[0:3] | join(" ")) | unique' real_logs.log
```

These datasets are perfect for learning advanced hawk queries with real data patterns.
EOF
}

# Download with retry logic
download_with_retry() {
    local url="$1"
    local output_file="$2"
    local description="$3"

    if [[ -f "$output_file" && "$FORCE" != "true" ]]; then
        log_warning "File exists, skipping: $output_file (use --force to overwrite)"
        return 0
    fi

    log_info "Downloading $description..."

    local attempt=1
    while [[ $attempt -le $RETRY_COUNT ]]; do
        if curl -sL "$url" -o "$output_file"; then
            if [[ -s "$output_file" ]]; then
                log_success "Downloaded: $output_file ($(du -h "$output_file" | cut -f1))"
                return 0
            else
                log_warning "Downloaded file is empty, retrying..."
                rm -f "$output_file"
            fi
        else
            log_warning "Download failed (attempt $attempt/$RETRY_COUNT)"
        fi

        ((attempt++))
        if [[ $attempt -le $RETRY_COUNT ]]; then
            sleep 2
        fi
    done

    log_error "Failed to download $description after $RETRY_COUNT attempts"
    return 1
}

# Download GitHub datasets
download_github() {
    log_info "Downloading GitHub datasets..."

    # Popular Rust repositories
    download_with_retry \
        "https://api.github.com/search/repositories?q=language:rust&sort=stars&per_page=100" \
        "$OUTPUT_DIR/github_rust_repos.json" \
        "GitHub Rust repositories"

    # Recent repositories
    download_with_retry \
        "https://api.github.com/search/repositories?q=created:>2024-01-01&sort=stars&per_page=100" \
        "$OUTPUT_DIR/github_recent_repos.json" \
        "GitHub recent repositories"

    # Issues from popular repositories
    download_with_retry \
        "https://api.github.com/search/issues?q=repo:microsoft/vscode+is:issue+state:open&per_page=100" \
        "$OUTPUT_DIR/github_vscode_issues.json" \
        "GitHub VSCode issues"

    # Users
    download_with_retry \
        "https://api.github.com/search/users?q=type:user+followers:>1000&per_page=100" \
        "$OUTPUT_DIR/github_users.json" \
        "GitHub popular users"
}

# Download public APIs dataset
download_apis() {
    log_info "Downloading public APIs datasets..."

    # Public APIs directory
    download_with_retry \
        "https://api.publicapis.org/entries" \
        "$OUTPUT_DIR/public_apis.json" \
        "Public APIs directory"

    # Sample API responses
    download_with_retry \
        "https://jsonplaceholder.typicode.com/posts" \
        "$OUTPUT_DIR/sample_posts.json" \
        "Sample blog posts API"

    download_with_retry \
        "https://jsonplaceholder.typicode.com/users" \
        "$OUTPUT_DIR/sample_users.json" \
        "Sample users API"

    download_with_retry \
        "https://jsonplaceholder.typicode.com/comments" \
        "$OUTPUT_DIR/sample_comments.json" \
        "Sample comments API"

    # HTTPBin responses
    download_with_retry \
        "https://httpbin.org/json" \
        "$OUTPUT_DIR/httpbin_sample.json" \
        "HTTPBin sample response"
}

# Download log samples
download_logs() {
    log_info "Downloading log samples..."

    # Create sample application logs
    cat >"$OUTPUT_DIR/real_application.log" <<'EOF'
2024-07-18 09:00:01 INFO  [main] Application startup initiated
2024-07-18 09:00:02 DEBUG [config] Loading configuration from /etc/app/config.yaml
2024-07-18 09:00:03 INFO  [database] Connecting to PostgreSQL at localhost:5432
2024-07-18 09:00:04 INFO  [cache] Redis connection established: localhost:6379
2024-07-18 09:00:05 INFO  [web] HTTP server listening on :8080
2024-07-18 09:00:06 INFO  [metrics] Prometheus metrics available at /metrics
2024-07-18 09:05:12 INFO  [auth] User authentication successful: user_id=12345
2024-07-18 09:05:45 DEBUG [api] Processing GET /api/v1/users request
2024-07-18 09:05:46 DEBUG [database] Executing query: SELECT * FROM users WHERE active = true
2024-07-18 09:05:47 INFO  [api] Request completed: GET /api/v1/users (120ms)
2024-07-18 09:08:23 WARN  [database] Slow query detected: SELECT * FROM orders WHERE created_at > '2024-01-01' (2.1s)
2024-07-18 09:10:15 ERROR [payment] Payment processing failed: card declined, order_id=ORD_123456
2024-07-18 09:10:16 ERROR [notification] Failed to send payment failure notification: SMTP error
2024-07-18 09:12:30 INFO  [scheduler] Starting daily backup job
2024-07-18 09:12:31 DEBUG [backup] Creating database backup: /backups/db_20240718_091231.sql
2024-07-18 09:15:45 WARN  [memory] High memory usage detected: 85% of 8GB used
2024-07-18 09:18:22 CRITICAL [security] Multiple failed login attempts: IP=192.168.1.100, attempts=5
2024-07-18 09:18:23 INFO  [security] IP address blocked: 192.168.1.100 (duration: 1h)
2024-07-18 09:20:10 ERROR [external] Third-party API timeout: payments.stripe.com (30s)
2024-07-18 09:20:11 WARN  [circuit] Circuit breaker opened: stripe_payments
2024-07-18 09:25:33 INFO  [health] Health check passed: all services operational
2024-07-18 09:30:00 DEBUG [scheduler] Cleaning up expired sessions
2024-07-18 09:30:01 INFO  [cleanup] Removed 150 expired sessions
2024-07-18 09:35:12 FATAL [storage] Critical error: disk space below 1% on /var/lib/data
2024-07-18 09:35:13 ERROR [alert] Failed to send critical alert: notification service down
2024-07-18 09:40:22 INFO  [maintenance] Emergency cleanup initiated
2024-07-18 09:40:45 INFO  [maintenance] Freed 2.5GB disk space
2024-07-18 09:41:00 INFO  [storage] Disk space recovered: 15% available
2024-07-18 09:45:30 DEBUG [api] Processing POST /api/v1/orders request
2024-07-18 09:45:31 INFO  [validation] Order validation passed: order_id=ORD_789012
2024-07-18 09:45:32 INFO  [payment] Payment processed successfully: $299.99
2024-07-18 09:45:33 INFO  [inventory] Stock updated: product_id=PROD001, new_quantity=45
2024-07-18 09:45:34 INFO  [email] Order confirmation sent: customer@example.com
2024-07-18 09:50:15 WARN  [rate_limit] Rate limit exceeded: API key abc123xyz (limit: 1000/hour)
2024-07-18 09:55:00 INFO  [backup] Daily backup completed successfully (duration: 42m29s)
2024-07-18 10:00:00 INFO  [scheduler] Hourly metrics collection started
2024-07-18 10:00:30 DEBUG [metrics] CPU usage: 45%, Memory: 62%, Disk: 85%
2024-07-18 10:05:22 ERROR [database] Connection pool exhausted: 20/20 connections in use
2024-07-18 10:05:23 WARN  [database] Increasing connection pool size to 30
2024-07-18 10:10:45 INFO  [deployment] Rolling update initiated: version v2.1.3
2024-07-18 10:11:00 INFO  [deployment] Health checks passed for new version
2024-07-18 10:11:15 INFO  [deployment] Traffic switched to new version
2024-07-18 10:15:30 DEBUG [cache] Cache eviction started: removing 1000 oldest entries
2024-07-18 10:20:12 WARN  [monitoring] Service degradation detected: response time > 1s
2024-07-18 10:25:00 INFO  [monitoring] Service performance restored: avg response time 250ms
EOF

    log_success "Created real_application.log ($(wc -l <"$OUTPUT_DIR/real_application.log") lines)"

    # Download sample Nginx logs (create realistic sample)
    cat >"$OUTPUT_DIR/nginx_production.log" <<'EOF'
10.0.1.100 - - [18/Jul/2024:09:00:15 +0000] "GET /api/v1/health HTTP/1.1" 200 15 "-" "HealthCheck/1.0"
203.0.113.45 - - [18/Jul/2024:09:02:30 +0000] "POST /api/v1/auth/login HTTP/1.1" 200 342 "https://app.example.com/login" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
198.51.100.23 - - [18/Jul/2024:09:05:12 +0000] "GET /api/v1/users HTTP/1.1" 200 2500 "https://app.example.com/dashboard" "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36"
192.168.1.150 - alice [18/Jul/2024:09:08:23 +0000] "GET /api/v1/orders?limit=50 HTTP/1.1" 200 8900 "https://app.example.com/orders" "Mozilla/5.0 (iPhone; CPU iPhone OS 14_7_1 like Mac OS X) AppleWebKit/605.1.15"
203.0.113.67 - bob [18/Jul/2024:09:10:15 +0000] "POST /api/v1/orders HTTP/1.1" 201 156 "https://app.example.com/checkout" "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:90.0) Gecko/20100101 Firefox/90.0"
198.51.100.89 - - [18/Jul/2024:09:12:30 +0000] "GET /api/v1/products?category=electronics HTTP/1.1" 200 1800 "https://app.example.com/products" "Mozilla/5.0 (Linux; Android 11; SM-G991B) AppleWebKit/537.36"
192.168.1.100 - - [18/Jul/2024:09:15:45 +0000] "POST /api/v1/auth/login HTTP/1.1" 401 89 "https://app.example.com/login" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
192.168.1.100 - - [18/Jul/2024:09:15:46 +0000] "POST /api/v1/auth/login HTTP/1.1" 401 89 "https://app.example.com/login" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
192.168.1.100 - - [18/Jul/2024:09:15:47 +0000] "POST /api/v1/auth/login HTTP/1.1" 401 89 "https://app.example.com/login" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
203.0.113.156 - carol [18/Jul/2024:09:18:22 +0000] "GET /api/v1/analytics/dashboard HTTP/1.1" 200 5600 "https://admin.example.com/dashboard" "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36"
198.51.100.234 - - [18/Jul/2024:09:20:10 +0000] "DELETE /api/v1/cart/items/123 HTTP/1.1" 204 0 "https://app.example.com/cart" "Mozilla/5.0 (iPad; CPU OS 14_7_1 like Mac OS X) AppleWebKit/605.1.15"
EOF

    log_success "Created nginx_production.log ($(wc -l <"$OUTPUT_DIR/nginx_production.log") lines)"
}

# Download configuration samples
download_configs() {
    log_info "Downloading configuration samples..."

    local config_dir="$OUTPUT_DIR/config_samples"
    mkdir -p "$config_dir"

    # Docker Compose sample
    cat >"$config_dir/docker-compose.yaml" <<'EOF'
version: '3.8'

services:
  web:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/ssl
    depends_on:
      - api
    networks:
      - frontend
      - backend

  api:
    build: 
      context: .
      dockerfile: Dockerfile.api
    environment:
      - DATABASE_URL=postgresql://user:pass@db:5432/appdb
      - REDIS_URL=redis://cache:6379
      - JWT_SECRET=${JWT_SECRET}
    depends_on:
      - db
      - cache
    networks:
      - backend
    deploy:
      replicas: 3

  db:
    image: postgres:14
    environment:
      POSTGRES_DB: appdb
      POSTGRES_USER: user
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - backend

  cache:
    image: redis:alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    networks:
      - backend

volumes:
  postgres_data:
  redis_data:

networks:
  frontend:
  backend:
EOF

    # Kubernetes deployment
    cat >"$config_dir/k8s-deployment.yaml" <<'EOF'
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
  labels:
    app: web-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: web-app
  template:
    metadata:
      labels:
        app: web-app
    spec:
      containers:
      - name: web
        image: nginx:1.21
        ports:
        - containerPort: 80
        resources:
          limits:
            cpu: 500m
            memory: 512Mi
          requests:
            cpu: 250m
            memory: 256Mi
        env:
        - name: API_URL
          value: "http://api-service:3000"
      - name: api
        image: myapp:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: url
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: web-app-service
spec:
  selector:
    app: web-app
  ports:
  - protocol: TCP
    port: 80
    targetPort: 80
  type: LoadBalancer
EOF

    log_success "Created configuration samples in $config_dir"
}

# Main download function
download_dataset() {
    local dataset_type=$1

    case $dataset_type in
    "github")
        download_github
        ;;
    "apis")
        download_apis
        ;;
    "logs")
        download_logs
        ;;
    "configs")
        download_configs
        ;;
    "all")
        download_github
        download_apis
        download_logs
        download_configs
        ;;
    *)
        log_error "Unknown dataset type: $dataset_type"
        exit 1
        ;;
    esac
}

# Display summary
show_summary() {
    log_info "Download completed!"
    log_info "Output directory: $OUTPUT_DIR"

    if [[ -d "$OUTPUT_DIR" ]]; then
        log_info "Downloaded files:"
        find "$OUTPUT_DIR" -type f -name "*.json" -o -name "*.log" -o -name "*.yaml" | while read -r file; do
            echo "  $(basename "$file") ($(du -h "$file" | cut -f1))"
        done

        local total_size=$(du -sh "$OUTPUT_DIR" | cut -f1)
        log_info "Total size: $total_size"
    fi

    echo
    log_info "Example queries to try:"
    echo "  hawk '.items[] | select(.language == \"Rust\") | count' $OUTPUT_DIR/github_rust_repos.json"
    echo "  hawk '.entries[] | group_by(.Category) | count' $OUTPUT_DIR/public_apis.json"
    echo "  hawk -t '. | select(. | contains(\"ERROR\")) | count' $OUTPUT_DIR/real_application.log"
}

# Main function
main() {
    log_info "Starting dataset download..."
    log_info "Configuration: dataset=$DATASET, output=$OUTPUT_DIR"

    check_dependencies
    setup_output_dir
    download_dataset "$DATASET"
    show_summary

    log_success "Download complete! Ready for advanced hawk queries! 🦅"
}

# Parse arguments and run
parse_args "$@"

# Validate arguments
case "$DATASET" in
"github" | "apis" | "logs" | "configs" | "all") ;;
*)
    log_error "Invalid dataset type: $DATASET"
    log_error "Valid types: github, apis, logs, configs, all"
    exit 1
    ;;
esac

# Run main function
main
