/**
 * Basic Examples for DSL IR Interpreter Node.js Bindings
 *
 * This file demonstrates common usage patterns.
 */

const { Interpreter, TracingInterpreter } = require('../index');

// ============================================================================
// Example 1: Simple Arithmetic
// ============================================================================

async function example1_arithmetic() {
  console.log('\n=== Example 1: Simple Arithmetic ===');

  const interpreter = new Interpreter();

  // Evaluate: 10 + 32
  const node = {
    type: "BinaryOp",
    op: "Add",
    left: { type: "Literal", value: { type: "int", value: 10 } },
    right: { type: "Literal", value: { type: "int", value: 32 } }
  };

  const result = await interpreter.eval(node);
  console.log('10 + 32 =', result.value);
}

// ============================================================================
// Example 2: Using Variables and Let Bindings
// ============================================================================

async function example2_variables() {
  console.log('\n=== Example 2: Variables and Let Bindings ===');

  const interpreter = new Interpreter();

  // Evaluate: let x = 5 in let y = 7 in x * y
  const node = {
    type: "Let",
    name: "x",
    value: { type: "Literal", value: { type: "int", value: 5 } },
    body: {
      type: "Let",
      name: "y",
      value: { type: "Literal", value: { type: "int", value: 7 } },
      body: {
        type: "BinaryOp",
        op: "Multiply",
        left: { type: "Variable", name: "x" },
        right: { type: "Variable", name: "y" }
      }
    }
  };

  const result = await interpreter.eval(node);
  console.log('let x = 5 in let y = 7 in x * y =', result.value);
}

// ============================================================================
// Example 3: Conditional Logic
// ============================================================================

async function example3_conditionals() {
  console.log('\n=== Example 3: Conditional Logic ===');

  const interpreter = new Interpreter();

  // Evaluate: if 10 > 5 then "yes" else "no"
  const node = {
    type: "If",
    condition: {
      type: "BinaryOp",
      op: "GreaterThan",
      left: { type: "Literal", value: { type: "int", value: 10 } },
      right: { type: "Literal", value: { type: "int", value: 5 } }
    },
    then_branch: { type: "Literal", value: { type: "string", value: "yes" } },
    else_branch: { type: "Literal", value: { type: "string", value: "no" } }
  };

  const result = await interpreter.eval(node);
  console.log('if 10 > 5 then "yes" else "no" =', result.value);
}

// ============================================================================
// Example 4: Arrays and Indexing
// ============================================================================

async function example4_arrays() {
  console.log('\n=== Example 4: Arrays and Indexing ===');

  const interpreter = new Interpreter();

  // Evaluate: [1, 2, 3, 4, 5][2]
  const node = {
    type: "Index",
    object: {
      type: "Array",
      elements: [
        { type: "Literal", value: { type: "int", value: 1 } },
        { type: "Literal", value: { type: "int", value: 2 } },
        { type: "Literal", value: { type: "int", value: 3 } },
        { type: "Literal", value: { type: "int", value: 4 } },
        { type: "Literal", value: { type: "int", value: 5 } }
      ]
    },
    index: { type: "Literal", value: { type: "int", value: 2 } }
  };

  const result = await interpreter.eval(node);
  console.log('[1, 2, 3, 4, 5][2] =', result.value);
}

// ============================================================================
// Example 5: Objects and Member Access
// ============================================================================

async function example5_objects() {
  console.log('\n=== Example 5: Objects and Member Access ===');

  const interpreter = new Interpreter();

  // Evaluate: { name: "Alice", age: 30 }.name
  const node = {
    type: "MemberAccess",
    object: {
      type: "Object",
      fields: {
        name: { type: "Literal", value: { type: "string", value: "Alice" } },
        age: { type: "Literal", value: { type: "int", value: 30 } }
      }
    },
    member: "name"
  };

  const result = await interpreter.eval(node);
  console.log('{ name: "Alice", age: 30 }.name =', result.value);
}

// ============================================================================
// Example 6: Using IR with Functions
// ============================================================================

async function example6_functions() {
  console.log('\n=== Example 6: Functions in IR ===');

  const ir = {
    version: "1.0",
    functions: {
      factorial: {
        params: ["n"],
        body: {
          type: "If",
          condition: {
            type: "BinaryOp",
            op: "LessThanOrEqual",
            left: { type: "Variable", name: "n" },
            right: { type: "Literal", value: { type: "int", value: 1 } }
          },
          then_branch: { type: "Literal", value: { type: "int", value: 1 } },
          else_branch: {
            type: "BinaryOp",
            op: "Multiply",
            left: { type: "Variable", name: "n" },
            right: {
              type: "FunctionCall",
              function: { type: "Variable", name: "factorial" },
              arguments: [
                {
                  type: "BinaryOp",
                  op: "Subtract",
                  left: { type: "Variable", name: "n" },
                  right: { type: "Literal", value: { type: "int", value: 1 } }
                }
              ]
            }
          }
        }
      }
    },
    entry_expr: {
      type: "FunctionCall",
      function: { type: "Variable", name: "factorial" },
      arguments: [{ type: "Literal", value: { type: "int", value: 5 } }]
    }
  };

  const result = await Interpreter.run(ir);
  console.log('factorial(5) =', result.value);
}

// ============================================================================
// Example 7: Lambda Functions
// ============================================================================

async function example7_lambdas() {
  console.log('\n=== Example 7: Lambda Functions ===');

  const interpreter = new Interpreter();

  // Evaluate: ((x, y) => x + y)(3, 4)
  const node = {
    type: "FunctionCall",
    function: {
      type: "Lambda",
      params: ["x", "y"],
      body: {
        type: "BinaryOp",
        op: "Add",
        left: { type: "Variable", name: "x" },
        right: { type: "Variable", name: "y" }
      }
    },
    arguments: [
      { type: "Literal", value: { type: "int", value: 3 } },
      { type: "Literal", value: { type: "int", value: 4 } }
    ]
  };

  const result = await interpreter.eval(node);
  console.log('((x, y) => x + y)(3, 4) =', result.value);
}

// ============================================================================
// Example 8: Tracing Execution
// ============================================================================

async function example8_tracing() {
  console.log('\n=== Example 8: Tracing Execution ===');

  const config = {
    max_depth: 10,
    capture_values: true,
    capture_spans: false
  };

  const interpreter = new TracingInterpreter(config);

  // Evaluate: (2 + 3) * 4
  const node = {
    type: "BinaryOp",
    op: "Multiply",
    left: {
      type: "BinaryOp",
      op: "Add",
      left: { type: "Literal", value: { type: "int", value: 2 } },
      right: { type: "Literal", value: { type: "int", value: 3 } }
    },
    right: { type: "Literal", value: { type: "int", value: 4 } }
  };

  const { value, traces } = await interpreter.evalWithTrace(node);
  console.log('(2 + 3) * 4 =', value.value);
  console.log(`Captured ${traces.length} trace events`);

  // Show first few trace events
  console.log('\nFirst 5 trace events:');
  traces.slice(0, 5).forEach((trace, i) => {
    console.log(`  ${i + 1}. ${trace.event_type} (depth: ${trace.depth}, type: ${trace.node_type})`);
    if (trace.value) {
      console.log(`     Value: ${JSON.stringify(trace.value)}`);
    }
  });
}

// ============================================================================
// Example 9: Error Handling
// ============================================================================

async function example9_errors() {
  console.log('\n=== Example 9: Error Handling ===');

  const interpreter = new Interpreter();

  // Try to reference an undefined variable
  const node = {
    type: "Variable",
    name: "undefined_variable",
    span: {
      file: "example.dsl",
      line: 10,
      column: 5
    }
  };

  try {
    await interpreter.eval(node);
  } catch (error) {
    console.log('Caught error:', error.message);
    console.log('Error kind:', error.kind);
    if (error.details.span) {
      const { file, line, column } = error.details.span;
      console.log(`Location: ${file}:${line}:${column}`);
    }
  }
}

// ============================================================================
// Example 10: Block Expressions
// ============================================================================

async function example10_blocks() {
  console.log('\n=== Example 10: Block Expressions ===');

  const interpreter = new Interpreter();

  // Evaluate a block with multiple statements
  const node = {
    type: "Block",
    statements: [
      {
        type: "Let",
        name: "x",
        value: { type: "Literal", value: { type: "int", value: 10 } },
        body: {
          type: "Let",
          name: "y",
          value: { type: "Literal", value: { type: "int", value: 20 } },
          body: {
            type: "BinaryOp",
            op: "Add",
            left: { type: "Variable", name: "x" },
            right: { type: "Variable", name: "y" }
          }
        }
      }
    ]
  };

  const result = await interpreter.eval(node);
  console.log('Block result:', result.value);
}

// ============================================================================
// Run All Examples
// ============================================================================

async function main() {
  console.log('DSL IR Interpreter - Node.js Bindings Examples');
  console.log('='.repeat(60));

  try {
    await example1_arithmetic();
    await example2_variables();
    await example3_conditionals();
    await example4_arrays();
    await example5_objects();
    await example6_functions();
    await example7_lambdas();
    await example8_tracing();
    await example9_errors();
    await example10_blocks();

    console.log('\n' + '='.repeat(60));
    console.log('All examples completed successfully!');
  } catch (error) {
    console.error('\nUnexpected error:', error);
    process.exit(1);
  }
}

// Run if this file is executed directly
if (require.main === module) {
  main().catch(console.error);
}

module.exports = {
  example1_arithmetic,
  example2_variables,
  example3_conditionals,
  example4_arrays,
  example5_objects,
  example6_functions,
  example7_lambdas,
  example8_tracing,
  example9_errors,
  example10_blocks
};
