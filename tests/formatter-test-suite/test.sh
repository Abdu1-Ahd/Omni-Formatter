#!/bin/bash
set -euo pipefail

# ── CASE 1: Variable declarations — spacing ────────────────────────────────
NAME="Alice"
AGE=  30
HOME_DIR=  "$HOME/documents"
LOG_FILE="/var/log/app.log"

# ── CASE 2: Functions — mixed spacing and brace style ─────────────────────
function greet ( ) {
    local name="$1"
    echo "Hello, $name!"
}

log_message() {
  local level="${1:-INFO}"
  local message="$2"
  echo "[$(date +%Y-%m-%dT%H:%M:%S)] [$level] $message" | tee -a "$LOG_FILE"
}

# ── CASE 3: Conditionals ──────────────────────────────────────────────────
check_environment() {
    if [ -z "${ENVIRONMENT:-}" ]; then
        echo "ENVIRONMENT not set"
        exit 1
    elif [ "$ENVIRONMENT" = "production" ]; then
        log_message "INFO" "Running in production mode"
    else
        log_message "DEBUG" "Running in $ENVIRONMENT mode"
    fi
}

# ── CASE 4: Loops ─────────────────────────────────────────────────────────
process_files() {
    local dir="$1"
    for file in "$dir"/*.log; do
        if [ -f "$file" ]; then
            echo "Processing: $file"
            while IFS= read -r line; do
                echo "$line"
            done < "$file"
        fi
    done
}

# ── CASE 5: Arrays and string ops ─────────────────────────────────────────
SERVICES=("nginx" "postgres" "redis" "app")
for service in "${SERVICES[@]}"; do
    if systemctl is-active --quiet "$service" 2>/dev/null; then
        echo "$service: running"
    else
        echo "$service: stopped"
    fi
done

# ── CASE 6: Heredoc ───────────────────────────────────────────────────────
cat << 'EOF'
This is a heredoc.
    Indentation is preserved exactly.
        Deep indent.
EOF

# ── CASE 7: Long line ─────────────────────────────────────────────────────
echo "This is a very long echo statement that exceeds the typical line width limit of eighty characters and may need to be wrapped"

# ── CASE 8: Trailing whitespace ───────────────────────────────────────────
echo "hello"   
echo "world"  
