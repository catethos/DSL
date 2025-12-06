#!/bin/bash
# Test parallel LLM execution with timing
#
# Usage: OPENAI_API_KEY=your_key ./example/test_parallel_llm.sh

set -e

if [ -z "$OPENAI_API_KEY" ]; then
    echo "Error: OPENAI_API_KEY not set"
    echo "Usage: OPENAI_API_KEY=your_key ./example/test_parallel_llm.sh"
    exit 1
fi

echo "Building..."
cargo build --release --quiet

echo ""
echo "=== Testing map_column with 5 parallel LLM calls ==="
echo ""

time ./target/release/lat run example/map_column_llm_test.lat

echo ""
echo "=== Expected: ~1-3 seconds (parallel) vs ~5-10 seconds (sequential) ==="
