#!/bin/bash
# Parallel vs Sequential LLM Benchmark
#
# This script runs both versions and compares timing.
#
# Prerequisites:
#   1. Build: cargo build --release -p lattice-cli
#   2. Set API key: export OPENAI_API_KEY=your_key
#
# Usage:
#   chmod +x example/parallel_vs_sequential.sh
#   OPENAI_API_KEY=your_key ./example/parallel_vs_sequential.sh

set -e

LAT="./target/release/lat"

if [ -z "$OPENAI_API_KEY" ]; then
    echo "Error: OPENAI_API_KEY not set"
    echo "Usage: OPENAI_API_KEY=your_key $0"
    exit 1
fi

echo "=== Parallel vs Sequential LLM Benchmark ==="
echo ""

# Create temporary files for each test
PARALLEL_SCRIPT=$(mktemp)
SEQUENTIAL_SCRIPT=$(mktemp)

# Parallel version
cat > "$PARALLEL_SCRIPT" << 'EOF'
def Classify(text: String) -> String {
    base_url: "https://api.openai.com/v1"
    model: "gpt-4o-mini"
    api_key_env: "OPENAI_API_KEY"
    temperature: 0.0
    max_tokens: 10
    prompt: f"""Classify sentiment as positive/negative/neutral (one word): {text}"""
}

let texts = [
    "I love this!",
    "This is terrible.",
    "It's okay I guess.",
    "Amazing experience!",
    "Very disappointed.",
    "Best ever!",
    "Total waste.",
    "It works fine.",
    "Exceeded expectations!",
    "Would not recommend."
]

parallel_map(texts, |t| Classify(t))
EOF

# Sequential version
cat > "$SEQUENTIAL_SCRIPT" << 'EOF'
def Classify(text: String) -> String {
    base_url: "https://api.openai.com/v1"
    model: "gpt-4o-mini"
    api_key_env: "OPENAI_API_KEY"
    temperature: 0.0
    max_tokens: 10
    prompt: f"""Classify sentiment as positive/negative/neutral (one word): {text}"""
}

let texts = [
    "I love this!",
    "This is terrible.",
    "It's okay I guess.",
    "Amazing experience!",
    "Very disappointed.",
    "Best ever!",
    "Total waste.",
    "It works fine.",
    "Exceeded expectations!",
    "Would not recommend."
]

let results = []
for text in texts {
    results = push(results, Classify(text))
}
results
EOF

echo "--- Running PARALLEL (10 concurrent LLM calls) ---"
PARALLEL_START=$(python3 -c 'import time; print(time.time())')
PARALLEL_RESULT=$("$LAT" run "$PARALLEL_SCRIPT" 2>/dev/null)
PARALLEL_END=$(python3 -c 'import time; print(time.time())')
PARALLEL_TIME=$(python3 -c "print(f'{$PARALLEL_END - $PARALLEL_START:.2f}')")

echo "Result: $PARALLEL_RESULT"
echo "Time: ${PARALLEL_TIME}s"
echo ""

echo "--- Running SEQUENTIAL (10 serial LLM calls) ---"
SEQUENTIAL_START=$(python3 -c 'import time; print(time.time())')
SEQUENTIAL_RESULT=$("$LAT" run "$SEQUENTIAL_SCRIPT" 2>/dev/null)
SEQUENTIAL_END=$(python3 -c 'import time; print(time.time())')
SEQUENTIAL_TIME=$(python3 -c "print(f'{$SEQUENTIAL_END - $SEQUENTIAL_START:.2f}')")

echo "Result: $SEQUENTIAL_RESULT"
echo "Time: ${SEQUENTIAL_TIME}s"
echo ""

echo "=== Summary ==="
SPEEDUP=$(python3 -c "print(f'{$SEQUENTIAL_TIME / $PARALLEL_TIME:.1f}')")
echo "Parallel:   ${PARALLEL_TIME}s"
echo "Sequential: ${SEQUENTIAL_TIME}s"
echo "Speedup:    ${SPEEDUP}x"

# Cleanup
rm -f "$PARALLEL_SCRIPT" "$SEQUENTIAL_SCRIPT"
