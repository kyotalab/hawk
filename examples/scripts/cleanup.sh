#!/bin/bash

# cleanup.sh - Clean up generated datasets and temporary files
# Usage: ./cleanup.sh [options]

set -euo pipefail

# Default configuration
DEFAULT_TARGET="all"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration variables
TARGET="$DEFAULT_TARGET"
INTERACTIVE=false
DRY_RUN=false
FORCE=false

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
Clean up generated datasets and temporary files.

Usage: $0 [OPTIONS]

Options:
    --target TARGET     What to clean: large, external, generated, temp, all (default: $DEFAULT_TARGET)
    --interactive       Ask for confirmation before deleting each item
    --dry-run          Show what would be deleted without actually deleting
    --force            Delete without any confirmation (dangerous!)
    --help             Show this help message

Targets:
    large              Generated large datasets (examples/large/)
    external           Downloaded external datasets (examples/external/)
    generated          All generated content (large + external)
    temp               Temporary files (*.tmp, *.temp, etc.)
    all                Everything except small sample data

Examples:
    $0                                    # Clean everything with confirmation
    $0 --target large --dry-run           # Preview large dataset cleanup
    $0 --target temp --force              # Force delete temp files
    $0 --interactive                      # Ask before each deletion

Safety Features:
    - Never deletes small/ sample data
    - Always shows what will be deleted
    - Requires confirmation unless --force is used
    - Supports dry-run mode

EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
        --target)
            TARGET="$2"
            shift 2
            ;;
        --interactive)
            INTERACTIVE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --force)
            FORCE=true
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

# Get files to clean based on target
get_cleanup_targets() {
    local target=$1
    local targets=()

    case $target in
    "large")
        if [[ -d "$BASE_DIR/large" ]]; then
            targets+=("$BASE_DIR/large")
        fi
        ;;
    "external")
        if [[ -d "$BASE_DIR/external" ]]; then
            targets+=("$BASE_DIR/external")
        fi
        ;;
    "generated")
        if [[ -d "$BASE_DIR/large" ]]; then
            targets+=("$BASE_DIR/large")
        fi
        if [[ -d "$BASE_DIR/external" ]]; then
            targets+=("$BASE_DIR/external")
        fi
        ;;
    "temp")
        # Find temporary files
        while IFS= read -r -d '' file; do
            targets+=("$file")
        done < <(find "$BASE_DIR" -type f \( -name "*.tmp" -o -name "*.temp" -o -name "*.swp" -o -name "*.bak" \) -print0 2>/dev/null || true)
        ;;
    "all")
        if [[ -d "$BASE_DIR/large" ]]; then
            targets+=("$BASE_DIR/large")
        fi
        if [[ -d "$BASE_DIR/external" ]]; then
            targets+=("$BASE_DIR/external")
        fi
        # Add temp files
        while IFS= read -r -d '' file; do
            targets+=("$file")
        done < <(find "$BASE_DIR" -type f \( -name "*.tmp" -o -name "*.temp" -o -name "*.swp" -o -name "*.bak" \) -print0 2>/dev/null || true)
        ;;
    *)
        log_error "Unknown target: $target"
        exit 1
        ;;
    esac

    printf '%s\n' "${targets[@]}"
}

# Calculate total size of targets
calculate_total_size() {
    local targets=("$@")
    local total_size=0

    for target in "${targets[@]}"; do
        if [[ -e "$target" ]]; then
            if [[ -d "$target" ]]; then
                local size=$(du -sb "$target" 2>/dev/null | cut -f1 || echo 0)
            else
                local size=$(stat -c%s "$target" 2>/dev/null || echo 0)
            fi
            total_size=$((total_size + size))
        fi
    done

    # Convert bytes to human readable
    if [[ $total_size -eq 0 ]]; then
        echo "0B"
    elif [[ $total_size -lt 1024 ]]; then
        echo "${total_size}B"
    elif [[ $total_size -lt 1048576 ]]; then
        echo "$((total_size / 1024))KB"
    elif [[ $total_size -lt 1073741824 ]]; then
        echo "$((total_size / 1048576))MB"
    else
        echo "$((total_size / 1073741824))GB"
    fi
}

# Show what will be deleted
show_cleanup_preview() {
    local targets=("$@")

    if [[ ${#targets[@]} -eq 0 ]]; then
        log_info "No files to clean up"
        return 0
    fi

    log_info "Files and directories to be deleted:"

    for target in "${targets[@]}"; do
        if [[ -e "$target" ]]; then
            local relative_path="${target#$BASE_DIR/}"
            if [[ -d "$target" ]]; then
                local size=$(du -sh "$target" 2>/dev/null | cut -f1 || echo "unknown")
                local count=$(find "$target" -type f | wc -l)
                echo "  📁 $relative_path/ ($size, $count files)"
            else
                local size=$(du -h "$target" 2>/dev/null | cut -f1 || echo "unknown")
                echo "  📄 $relative_path ($size)"
            fi
        fi
    done

    local total_size=$(calculate_total_size "${targets[@]}")
    echo
    log_info "Total size to be freed: $total_size"
}

# Confirm deletion
confirm_deletion() {
    if [[ "$FORCE" == "true" ]]; then
        return 0
    fi

    echo
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "This is a dry run - no files will actually be deleted"
        return 0
    fi

    read -p "$(echo -e "${YELLOW}Do you want to proceed with deletion? (y/N): ${NC}")" -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        return 0
    else
        log_info "Cleanup cancelled by user"
        return 1
    fi
}

# Interactive confirmation for each item
confirm_item() {
    local item="$1"
    local relative_path="${item#$BASE_DIR/}"

    read -p "$(echo -e "${YELLOW}Delete $relative_path? (y/N/q): ${NC}")" -n 1 -r
    echo
    case $REPLY in
    [Yy])
        return 0
        ;;
    [Qq])
        log_info "Cleanup cancelled by user"
        exit 0
        ;;
    *)
        return 1
        ;;
    esac
}

# Perform cleanup
perform_cleanup() {
    local targets=("$@")
    local deleted_count=0
    local total_freed=0

    for target in "${targets[@]}"; do
        if [[ ! -e "$target" ]]; then
            continue
        fi

        if [[ "$INTERACTIVE" == "true" && "$DRY_RUN" != "true" ]]; then
            if ! confirm_item "$target"; then
                continue
            fi
        fi

        local relative_path="${target#$BASE_DIR/}"

        if [[ "$DRY_RUN" == "true" ]]; then
            log_info "[DRY RUN] Would delete: $relative_path"
            ((deleted_count++))
        else
            # Calculate size before deletion
            local size=0
            if [[ -d "$target" ]]; then
                size=$(du -sb "$target" 2>/dev/null | cut -f1 || echo 0)
            else
                size=$(stat -c%s "$target" 2>/dev/null || echo 0)
            fi

            # Perform deletion
            if rm -rf "$target" 2>/dev/null; then
                log_success "Deleted: $relative_path"
                ((deleted_count++))
                total_freed=$((total_freed + size))
            else
                log_error "Failed to delete: $relative_path"
            fi
        fi
    done

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Dry run completed: $deleted_count items would be deleted"
    else
        local freed_readable=$(echo $total_freed | awk '{
            if ($1 >= 1073741824) printf "%.1fGB", $1/1073741824
            else if ($1 >= 1048576) printf "%.1fMB", $1/1048576  
            else if ($1 >= 1024) printf "%.1fKB", $1/1024
            else printf "%dB", $1
        }')
        log_success "Cleanup completed: $deleted_count items deleted, $freed_readable freed"
    fi
}

# Validate target
validate_target() {
    case "$TARGET" in
    "large" | "external" | "generated" | "temp" | "all") ;;
    *)
        log_error "Invalid target: $TARGET"
        log_error "Valid targets: large, external, generated, temp, all"
        exit 1
        ;;
    esac
}

# Safety check to prevent accidental deletion of important files
safety_check() {
    local targets=("$@")

    for target in "${targets[@]}"; do
        # Ensure we never delete the small samples directory
        if [[ "$target" == *"/small"* ]] || [[ "$target" == *"/scripts"* ]]; then
            log_error "Safety check failed: attempting to delete protected directory: $target"
            log_error "This script will never delete small sample data or scripts"
            exit 1
        fi

        # Ensure we're only deleting within the examples directory
        if [[ "$target" != "$BASE_DIR"* ]]; then
            log_error "Safety check failed: attempting to delete outside examples directory: $target"
            exit 1
        fi
    done
}

# Main function
main() {
    log_info "Starting cleanup process..."
    log_info "Target: $TARGET"

    # Get targets to clean
    mapfile -t targets < <(get_cleanup_targets "$TARGET")

    if [[ ${#targets[@]} -eq 0 ]]; then
        log_info "Nothing to clean up for target: $TARGET"
        return 0
    fi

    # Safety checks
    safety_check "${targets[@]}"

    # Show preview
    show_cleanup_preview "${targets[@]}"

    # Confirm and perform cleanup
    if confirm_deletion; then
        perform_cleanup "${targets[@]}"
    fi
}

# Display banner
show_banner() {
    echo "🧹 Hawk Examples Cleanup Tool"
    echo "=============================="
    echo
}

# Parse arguments and run
parse_args "$@"
validate_target

# Show banner unless in quiet mode
show_banner

# Conflict checking
if [[ "$INTERACTIVE" == "true" && "$FORCE" == "true" ]]; then
    log_error "Cannot use --interactive and --force together"
    exit 1
fi

if [[ "$DRY_RUN" == "true" && "$FORCE" == "true" ]]; then
    log_warning "--force has no effect in dry-run mode"
fi

# Run main function
main

log_info "Cleanup process completed"
