#!/bin/bash

# Test runner for higher-order functions

echo "=== Running Basic HOF Tests ==="
cargo run --quiet --bin dsl -- -c < examples/test_hof_basic.txt 2>&1 | grep -E "^(\[|✓|Error)"

echo ""
echo "=== Running Chaining Tests ==="
cargo run --quiet --bin dsl -- -c < examples/test_hof_chaining.txt 2>&1 | grep -E "^(\[|✓|Error)"

echo ""
echo "=== Running Edge Case Tests ==="
cargo run --quiet --bin dsl -- -c < examples/test_hof_edge_cases.txt 2>&1 | grep -E "^(\[|✓|Error)"

echo ""
echo "=== Running Map/Object Tests ==="
cargo run --quiet --bin dsl -- -c < examples/test_hof_maps.txt 2>&1 | grep -E "^(\[|✓|{|Error)"
