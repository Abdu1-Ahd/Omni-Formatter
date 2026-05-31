#!/bin/bash
set -e
export PATH="/c/Users/zawia/.cargo/bin:$PATH"

echo "=== Step 1: Build check ==="
cargo.exe build --release --manifest-path tests/native_runner/Cargo.toml 2>&1 || { echo "BUILD FAILED"; exit 1; }

echo "=== Step 2 & 3: Format pass & Idempotency check ==="
declare -A TIME_RESULTS
declare -A IDEMP_RESULTS

for file in tests/fixtures/*; do
  if [[ -f "$file" && ! "$file" == *.out* && ! "$file" == *.ref* && "$file" != *"pyproject.toml"* ]]; then
    filename=$(basename "$file")
    start=$(date +%s%N)
    cargo.exe run --manifest-path tests/native_runner/Cargo.toml --release -- format "$file" --output "${file}.out" > /dev/null 2>&1
    end=$(date +%s%N)
    elapsed=$(( (end - start) / 1000000 ))
    TIME_RESULTS["$filename"]=$elapsed
    
    cargo.exe run --manifest-path tests/native_runner/Cargo.toml --release -- format "${file}.out" --output "${file}.out2" > /dev/null 2>&1
    if diff -q "${file}.out" "${file}.out2" > /dev/null; then
      IDEMP_RESULTS["$filename"]="PASS"
    else
      IDEMP_RESULTS["$filename"]="FAIL"
    fi
    echo "[$filename] Time: ${elapsed}ms | Idempotency: ${IDEMP_RESULTS["$filename"]}"
  fi
done

echo "=== Step 4: Quality check ==="
declare -A COMPAT_RESULTS

for ext in js ts css scss html; do
  fname="messy.${ext}"
  if [[ -f "tests/fixtures/$fname" ]]; then
    echo "Testing Prettier ($ext)..."
    npx --yes prettier@3 "tests/fixtures/$fname" > "tests/fixtures/${fname}.ref" 2>/dev/null || true
    diff_count=$(diff -u "tests/fixtures/${fname}.out" "tests/fixtures/${fname}.ref" | grep -c '^[+-][^+-]' || true)
    if [ "$diff_count" -eq 0 ]; then COMPAT_RESULTS["$fname"]="EXACT"
    elif [ "$diff_count" -le 15 ]; then COMPAT_RESULTS["$fname"]="NEAR"
    else COMPAT_RESULTS["$fname"]="FAIL"; echo "FAIL $ext diff:"; diff -u "tests/fixtures/${fname}.out" "tests/fixtures/${fname}.ref" | head -n 12 || true; fi
    echo "$ext: ${COMPAT_RESULTS["$fname"]}"
  fi
done

echo "Testing Black (Python)..."
python -m pip install black >/dev/null 2>&1 || true
cp tests/fixtures/messy.py tests/fixtures/messy.py.ref
python -m black -q --config tests/pyproject.toml tests/fixtures/messy.py.ref >/dev/null 2>&1 || true
diff_count=$(diff -u tests/fixtures/messy.py.out tests/fixtures/messy.py.ref | grep -c '^[+-][^+-]' || true)
if [ "$diff_count" -eq 0 ]; then COMPAT_RESULTS["messy.py"]="EXACT"
elif [ "$diff_count" -le 15 ]; then COMPAT_RESULTS["messy.py"]="NEAR"
else COMPAT_RESULTS["messy.py"]="FAIL"; echo "FAIL py diff:"; diff -u tests/fixtures/messy.py.out tests/fixtures/messy.py.ref | head -n 12 || true; fi
echo "PY: ${COMPAT_RESULTS["messy.py"]}"


echo "Testing Rustfmt (Rust)..."
rustup component add rustfmt >/dev/null 2>&1 || true
cp tests/fixtures/messy.rs tests/fixtures/messy.rs.ref
rustfmt tests/fixtures/messy.rs.ref >/dev/null 2>&1 || true
diff_count=$(diff -u tests/fixtures/messy.rs.out tests/fixtures/messy.rs.ref | grep -c '^[+-][^+-]' || true)
if [ "$diff_count" -eq 0 ]; then COMPAT_RESULTS["messy.rs"]="EXACT"
elif [ "$diff_count" -le 15 ]; then COMPAT_RESULTS["messy.rs"]="NEAR"
else COMPAT_RESULTS["messy.rs"]="FAIL"; echo "FAIL rs diff:"; diff -u tests/fixtures/messy.rs.out tests/fixtures/messy.rs.ref | head -n 12 || true; fi
echo "RS: ${COMPAT_RESULTS["messy.rs"]}"

echo "Testing Gofmt (Go)..."
cp tests/fixtures/messy.go tests/fixtures/messy.go.ref
if command -v go &> /dev/null; then
  gofmt -w tests/fixtures/messy.go.ref >/dev/null 2>&1 || true
  diff_count=$(diff -u tests/fixtures/messy.go.out tests/fixtures/messy.go.ref | grep -c '^[+-][^+-]' || true)
  if [ "$diff_count" -eq 0 ]; then COMPAT_RESULTS["messy.go"]="EXACT"
  elif [ "$diff_count" -le 3 ]; then COMPAT_RESULTS["messy.go"]="NEAR"
  else COMPAT_RESULTS["messy.go"]="FAIL"; echo "FAIL go diff:"; diff -u tests/fixtures/messy.go.out tests/fixtures/messy.go.ref | head -n 12 || true; fi
else
  COMPAT_RESULTS["messy.go"]="SKIPPED"
fi
echo "GO: ${COMPAT_RESULTS["messy.go"]}"

echo "=== Step 5: Zone routing check (HTML) ==="
script_content=$(awk '/<script>/{flag=1; next} /<\/script>/{flag=0} flag' tests/fixtures/messy.html.out)
echo "$script_content" > tests/fixtures/script_block.js
npx --yes prettier@3 --parser babel tests/fixtures/script_block.js > tests/fixtures/script_block.js.ref 2>/dev/null || true
if diff -q tests/fixtures/script_block.js tests/fixtures/script_block.js.ref > /dev/null; then
  echo "ZONE JS: PASS"
else
  echo "ZONE JS: FAIL"
fi

style_content=$(awk '/<style>/{flag=1; next} /<\/style>/{flag=0} flag' tests/fixtures/messy.html.out)
echo "$style_content" > tests/fixtures/style_block.css
npx --yes prettier@3 --parser css tests/fixtures/style_block.css > tests/fixtures/style_block.css.ref 2>/dev/null || true
if diff -q tests/fixtures/style_block.css tests/fixtures/style_block.css.ref > /dev/null; then
  echo "ZONE CSS: PASS"
else
  echo "ZONE CSS: FAIL"
fi

echo "=== Step 6: Speed threshold check ==="
for file in "${!TIME_RESULTS[@]}"; do
  ms=${TIME_RESULTS[$file]}
  if [ "$ms" -le 200 ]; then
    echo "SPEED [$file]: PASS (${ms}ms)"
  elif [ "$ms" -le 400 ]; then
    echo "SPEED [$file]: WARN (${ms}ms)"
  else
    echo "SPEED [$file]: FAIL (${ms}ms)"
  fi
done

echo "=== RUN COMPLETE ==="
