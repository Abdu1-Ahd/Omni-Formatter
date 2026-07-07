#!/usr/bin/env zsh

# ── CASE 1: Variable declarations ─────────────────────────────────────────
NAME="Alice"
AGE=  30
typeset -A config
config=(host localhost port 5432 db mydb)

# ── CASE 2: Functions ─────────────────────────────────────────────────────
function greet() {
    local name="${1:-World}"
    echo "Hello, ${name}!"
}

log_message() {
  local level="${1:-INFO}"
  local message="$2"
  print -P "%F{blue}[${level}]%f ${message}"
}

# ── CASE 3: Arrays ────────────────────────────────────────────────────────
local -a SERVICES=(nginx postgres redis app)
for service in "${SERVICES[@]}"; do
    if systemctl is-active --quiet "${service}" 2>/dev/null; then
        log_message INFO "${service}: running"
    else
        log_message WARN "${service}: stopped"
    fi
done

# ── CASE 4: Zsh-specific: glob qualifiers, parameter expansion ────────────
local files=(**/*.{js,ts}(N))
local upper="${NAME:u}"
local lower="${NAME:l}"
local length="${#NAME}"

# ── CASE 5: Associative array and integer ─────────────────────────────────
integer count=0
typeset -A scores
scores[alice]=95
scores[bob]=  87
scores[carol]=92

for user in "${(@k)scores}"; do
    (( count++ ))
    print "${user}: ${scores[$user]}"
done

# ── CASE 6: Completion system ─────────────────────────────────────────────
autoload -Uz compinit
compinit

# ── CASE 7: Trailing whitespace ───────────────────────────────────────────
echo "hello"   
echo "world"  
