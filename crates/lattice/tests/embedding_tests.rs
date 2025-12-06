//! Integration tests for the Lattice embedding API
//!
//! These tests verify that the LatticeRuntime API works correctly for
//! embedding scenarios, testing it as an external consumer would.

use lattice::runtime::{
    EnumSchema, FieldSchema, FunctionSignature, LatticeRuntime, LatticeValue, LlmError,
    LlmProvider, LlmRequest, LlmResponse, NoLlmProvider, NoSqlProvider, ParameterSchema,
    RuntimeBuilder, SharedRuntime, StructSchema, TypeSchema,
};

// ============================================================
// Helper functions
// ============================================================

/// Create a minimal runtime without LLM or SQL providers
fn create_minimal_runtime() -> LatticeRuntime {
    LatticeRuntime::from_built(
        RuntimeBuilder::new()
            .without_llm()
            .without_sql()
            .build()
            .expect("Failed to build runtime"),
    )
}

// ============================================================
// Runtime Creation Tests
// ============================================================

#[test]
fn test_create_runtime_minimal() {
    let runtime = create_minimal_runtime();
    assert!(runtime.function_names().is_empty());
    assert!(runtime.llm_function_names().is_empty());
}

#[test]
fn test_create_runtime_with_builder() {
    let built = RuntimeBuilder::new()
        .without_llm()
        .without_sql()
        .with_stdlib_core()
        .build()
        .expect("Build should succeed");

    let runtime = LatticeRuntime::from_built(built);
    // Runtime should be ready to use
    assert!(runtime.get_types().is_empty());
}

#[test]
fn test_runtime_builder_chaining() {
    use std::time::Duration;

    let built = RuntimeBuilder::new()
        .without_llm()
        .without_sql()
        .with_stdlib_core()
        .without_stdlib_core()
        .with_stdlib_io()
        .with_timeout(Duration::from_secs(30))
        .with_max_concurrent_llm_calls(5)
        .build()
        .expect("Build should succeed");

    assert_eq!(built.config().timeout, Some(Duration::from_secs(30)));
    assert_eq!(built.config().max_concurrent_llm_calls, Some(5));
    assert!(!built.config().include_stdlib_core);
    assert!(built.config().include_stdlib_io);
}

// ============================================================
// Expression Evaluation Tests
// ============================================================

#[test]
fn test_eval_integers() {
    let mut runtime = create_minimal_runtime();

    assert_eq!(runtime.eval("42").unwrap(), LatticeValue::Int(42));
    assert_eq!(runtime.eval("-100").unwrap(), LatticeValue::Int(-100));
    assert_eq!(runtime.eval("0").unwrap(), LatticeValue::Int(0));
}

#[test]
fn test_eval_floats() {
    let mut runtime = create_minimal_runtime();

    assert_eq!(runtime.eval("3.14").unwrap(), LatticeValue::Float(3.14));
    assert_eq!(runtime.eval("-2.5").unwrap(), LatticeValue::Float(-2.5));
}

#[test]
fn test_eval_booleans() {
    let mut runtime = create_minimal_runtime();

    assert_eq!(runtime.eval("true").unwrap(), LatticeValue::Bool(true));
    assert_eq!(runtime.eval("false").unwrap(), LatticeValue::Bool(false));
}

#[test]
fn test_eval_strings() {
    let mut runtime = create_minimal_runtime();

    assert_eq!(
        runtime.eval(r#""hello""#).unwrap(),
        LatticeValue::String("hello".to_string())
    );
    assert_eq!(
        runtime.eval(r#""hello world""#).unwrap(),
        LatticeValue::String("hello world".to_string())
    );
    assert_eq!(
        runtime.eval(r#""""#).unwrap(),
        LatticeValue::String(String::new())
    );
}

#[test]
fn test_eval_null() {
    let mut runtime = create_minimal_runtime();

    assert_eq!(runtime.eval("null").unwrap(), LatticeValue::Null);
}

#[test]
fn test_eval_lists() {
    let mut runtime = create_minimal_runtime();

    assert_eq!(
        runtime.eval("[1, 2, 3]").unwrap(),
        LatticeValue::List(vec![
            LatticeValue::Int(1),
            LatticeValue::Int(2),
            LatticeValue::Int(3),
        ])
    );

    // Empty list
    assert_eq!(runtime.eval("[]").unwrap(), LatticeValue::List(vec![]));

    // Nested lists
    assert_eq!(
        runtime.eval("[[1, 2], [3, 4]]").unwrap(),
        LatticeValue::List(vec![
            LatticeValue::List(vec![LatticeValue::Int(1), LatticeValue::Int(2)]),
            LatticeValue::List(vec![LatticeValue::Int(3), LatticeValue::Int(4)]),
        ])
    );
}

#[test]
fn test_eval_maps() {
    let mut runtime = create_minimal_runtime();

    let result = runtime.eval(r#"{"a": 1, "b": 2}"#).unwrap();

    // Maps are sorted by key for deterministic output
    assert_eq!(
        result,
        LatticeValue::Map(vec![
            ("a".to_string(), LatticeValue::Int(1)),
            ("b".to_string(), LatticeValue::Int(2)),
        ])
    );
}

#[test]
fn test_eval_arithmetic() {
    let mut runtime = create_minimal_runtime();

    assert_eq!(runtime.eval("1 + 2").unwrap(), LatticeValue::Int(3));
    assert_eq!(runtime.eval("10 - 3").unwrap(), LatticeValue::Int(7));
    assert_eq!(runtime.eval("4 * 5").unwrap(), LatticeValue::Int(20));
    assert_eq!(runtime.eval("15 / 3").unwrap(), LatticeValue::Int(5));
    assert_eq!(runtime.eval("17 % 5").unwrap(), LatticeValue::Int(2));
    assert_eq!(runtime.eval("1 + 2 * 3").unwrap(), LatticeValue::Int(7));
    assert_eq!(runtime.eval("(1 + 2) * 3").unwrap(), LatticeValue::Int(9));
}

#[test]
fn test_eval_string_concatenation() {
    let mut runtime = create_minimal_runtime();

    assert_eq!(
        runtime.eval(r#""hello" + " " + "world""#).unwrap(),
        LatticeValue::String("hello world".to_string())
    );
}

#[test]
fn test_eval_comparisons() {
    let mut runtime = create_minimal_runtime();

    assert_eq!(runtime.eval("1 < 2").unwrap(), LatticeValue::Bool(true));
    assert_eq!(runtime.eval("2 > 1").unwrap(), LatticeValue::Bool(true));
    assert_eq!(runtime.eval("1 == 1").unwrap(), LatticeValue::Bool(true));
    assert_eq!(runtime.eval("1 != 2").unwrap(), LatticeValue::Bool(true));
    assert_eq!(runtime.eval("1 <= 1").unwrap(), LatticeValue::Bool(true));
    assert_eq!(runtime.eval("1 >= 1").unwrap(), LatticeValue::Bool(true));
}

#[test]
fn test_eval_logical() {
    let mut runtime = create_minimal_runtime();

    // Lattice uses && and || for logical operators
    assert_eq!(
        runtime.eval("true && true").unwrap(),
        LatticeValue::Bool(true)
    );
    assert_eq!(
        runtime.eval("true && false").unwrap(),
        LatticeValue::Bool(false)
    );
    assert_eq!(
        runtime.eval("true || false").unwrap(),
        LatticeValue::Bool(true)
    );
}

#[test]
fn test_eval_let_binding() {
    let mut runtime = create_minimal_runtime();

    // Let binding returns null, but sets up the variable
    runtime.eval("let x = 42").unwrap();
    assert_eq!(runtime.eval("x").unwrap(), LatticeValue::Int(42));
}

#[test]
fn test_eval_if_expression() {
    let mut runtime = create_minimal_runtime();

    // If expressions need to be assigned to produce a value
    assert_eq!(
        runtime.eval("let x = if true { 1 } else { 2 }\nx").unwrap(),
        LatticeValue::Int(1)
    );
    assert_eq!(
        runtime.eval("let y = if false { 1 } else { 2 }\ny").unwrap(),
        LatticeValue::Int(2)
    );
}

#[test]
fn test_eval_for_loop() {
    let mut runtime = create_minimal_runtime();

    // For loop with accumulator
    let result = runtime
        .eval(
            r#"
        let sum = 0
        for x in [1, 2, 3, 4, 5] {
            sum = sum + x
        }
        sum
    "#,
        )
        .unwrap();
    assert_eq!(result, LatticeValue::Int(15));
}

// ============================================================
// Binding Tests
// ============================================================

#[test]
fn test_eval_with_bindings() {
    let mut runtime = create_minimal_runtime();

    let result = runtime
        .eval_with_bindings(
            "x + y",
            vec![
                ("x".to_string(), LatticeValue::Int(10)),
                ("y".to_string(), LatticeValue::Int(20)),
            ],
        )
        .unwrap();

    assert_eq!(result, LatticeValue::Int(30));
}

#[test]
fn test_eval_with_string_bindings() {
    let mut runtime = create_minimal_runtime();

    let result = runtime
        .eval_with_bindings(
            r#"greeting + ", " + name + "!""#,
            vec![
                (
                    "greeting".to_string(),
                    LatticeValue::String("Hello".to_string()),
                ),
                (
                    "name".to_string(),
                    LatticeValue::String("World".to_string()),
                ),
            ],
        )
        .unwrap();

    assert_eq!(
        result,
        LatticeValue::String("Hello, World!".to_string())
    );
}

#[test]
fn test_eval_with_list_bindings() {
    let mut runtime = create_minimal_runtime();

    let result = runtime
        .eval_with_bindings(
            r#"
            let sum = 0
            for n in numbers {
                sum = sum + n
            }
            sum
        "#,
            vec![(
                "numbers".to_string(),
                LatticeValue::List(vec![
                    LatticeValue::Int(1),
                    LatticeValue::Int(2),
                    LatticeValue::Int(3),
                ]),
            )],
        )
        .unwrap();

    assert_eq!(result, LatticeValue::Int(6));
}

// ============================================================
// Global Variable Tests
// ============================================================

#[test]
fn test_set_and_get_global() {
    let mut runtime = create_minimal_runtime();

    runtime.set_global("count", LatticeValue::Int(42));
    assert_eq!(runtime.get_global("count"), Some(LatticeValue::Int(42)));
}

#[test]
fn test_global_persists_across_evals() {
    let mut runtime = create_minimal_runtime();

    // First eval creates global
    runtime.eval("let x = 100").unwrap();

    // Second eval uses it
    let result = runtime.eval("x * 2").unwrap();
    assert_eq!(result, LatticeValue::Int(200));
}

#[test]
fn test_global_names() {
    let mut runtime = create_minimal_runtime();

    runtime.set_global("foo", LatticeValue::Int(1));
    runtime.set_global("bar", LatticeValue::Int(2));

    let names = runtime.global_names();
    assert!(names.contains(&"foo".to_string()));
    assert!(names.contains(&"bar".to_string()));
}

#[test]
fn test_use_global_in_eval() {
    let mut runtime = create_minimal_runtime();

    runtime.set_global("multiplier", LatticeValue::Int(10));

    let result = runtime.eval("multiplier * 5").unwrap();
    assert_eq!(result, LatticeValue::Int(50));
}

// ============================================================
// Function Definition and Call Tests
// ============================================================

#[test]
fn test_define_function() {
    let mut runtime = create_minimal_runtime();

    runtime
        .eval(
            r#"
        def add(a: Int, b: Int) -> Int {
            a + b
        }
    "#,
        )
        .unwrap();

    assert!(runtime.function_names().contains(&"add".to_string()));
    assert!(runtime.has_function("add"));
}

#[test]
fn test_call_function_via_eval() {
    let mut runtime = create_minimal_runtime();

    runtime
        .eval(
            r#"
        def add(a: Int, b: Int) -> Int {
            a + b
        }
    "#,
        )
        .unwrap();

    let result = runtime.eval("add(3, 4)").unwrap();
    assert_eq!(result, LatticeValue::Int(7));
}

#[test]
fn test_call_function_directly() {
    let mut runtime = create_minimal_runtime();

    runtime
        .eval(
            r#"
        def add(a: Int, b: Int) -> Int {
            a + b
        }
    "#,
        )
        .unwrap();

    let result = runtime
        .call("add", vec![LatticeValue::Int(3), LatticeValue::Int(4)])
        .unwrap();
    assert_eq!(result, LatticeValue::Int(7));
}

#[test]
fn test_call_no_args() {
    let mut runtime = create_minimal_runtime();

    runtime
        .eval(
            r#"
        def get_answer() -> Int {
            42
        }
    "#,
        )
        .unwrap();

    let result = runtime.call("get_answer", vec![]).unwrap();
    assert_eq!(result, LatticeValue::Int(42));
}

#[test]
fn test_call_with_string_arg() {
    let mut runtime = create_minimal_runtime();

    runtime
        .eval(
            r#"
        def greet(name: String) -> String {
            "Hello, " + name + "!"
        }
    "#,
        )
        .unwrap();

    let result = runtime
        .call("greet", vec![LatticeValue::String("World".to_string())])
        .unwrap();
    assert_eq!(
        result,
        LatticeValue::String("Hello, World!".to_string())
    );
}

#[test]
fn test_call_with_list_arg() {
    let mut runtime = create_minimal_runtime();

    runtime
        .eval(
            r#"
        def sum_list(nums: [Int]) -> Int {
            let total = 0
            for n in nums {
                total = total + n
            }
            total
        }
    "#,
        )
        .unwrap();

    let result = runtime
        .call(
            "sum_list",
            vec![LatticeValue::List(vec![
                LatticeValue::Int(1),
                LatticeValue::Int(2),
                LatticeValue::Int(3),
            ])],
        )
        .unwrap();
    assert_eq!(result, LatticeValue::Int(6));
}

#[test]
fn test_call_returns_list() {
    let mut runtime = create_minimal_runtime();

    runtime
        .eval(
            r#"
        def get_nums() -> [Int] {
            [1, 2, 3]
        }
    "#,
        )
        .unwrap();

    let result = runtime.call("get_nums", vec![]).unwrap();
    assert_eq!(
        result,
        LatticeValue::List(vec![
            LatticeValue::Int(1),
            LatticeValue::Int(2),
            LatticeValue::Int(3),
        ])
    );
}

#[test]
fn test_call_undefined_function() {
    let mut runtime = create_minimal_runtime();

    let result = runtime.call("nonexistent", vec![]);
    assert!(result.is_err());
}

#[test]
fn test_call_multiple_times() {
    let mut runtime = create_minimal_runtime();

    runtime
        .eval(
            r#"
        def multiply(a: Int, b: Int) -> Int {
            a * b
        }
    "#,
        )
        .unwrap();

    // Call the same function multiple times
    let r1 = runtime
        .call("multiply", vec![LatticeValue::Int(2), LatticeValue::Int(3)])
        .unwrap();
    let r2 = runtime
        .call("multiply", vec![LatticeValue::Int(4), LatticeValue::Int(5)])
        .unwrap();
    let r3 = runtime
        .call("multiply", vec![LatticeValue::Int(6), LatticeValue::Int(7)])
        .unwrap();

    assert_eq!(r1, LatticeValue::Int(6));
    assert_eq!(r2, LatticeValue::Int(20));
    assert_eq!(r3, LatticeValue::Int(42));
}

#[test]
fn test_call_function_that_calls_function() {
    let mut runtime = create_minimal_runtime();

    runtime
        .eval(
            r#"
        def double(x: Int) -> Int {
            x * 2
        }

        def quadruple(x: Int) -> Int {
            double(double(x))
        }
    "#,
        )
        .unwrap();

    let result = runtime
        .call("quadruple", vec![LatticeValue::Int(5)])
        .unwrap();
    assert_eq!(result, LatticeValue::Int(20));
}

#[test]
fn test_call_function_with_global() {
    let mut runtime = create_minimal_runtime();

    runtime.set_global("multiplier", LatticeValue::Int(10));

    runtime
        .eval(
            r#"
        def scale(x: Int) -> Int {
            x * multiplier
        }
    "#,
        )
        .unwrap();

    let result = runtime
        .call("scale", vec![LatticeValue::Int(5)])
        .unwrap();
    assert_eq!(result, LatticeValue::Int(50));
}

// ============================================================
// Function Signature Tests
// ============================================================

#[test]
fn test_get_function_signatures() {
    let mut runtime = create_minimal_runtime();

    runtime
        .eval(
            r#"
        def add(a: Int, b: Int) -> Int {
            a + b
        }

        def greet(name: String) -> String {
            "Hello, " + name
        }
    "#,
        )
        .unwrap();

    let signatures = runtime.get_function_signatures();
    assert_eq!(signatures.len(), 2);

    let names: Vec<&str> = signatures.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"add"));
    assert!(names.contains(&"greet"));
}

#[test]
fn test_get_function_signature_by_name() {
    let mut runtime = create_minimal_runtime();

    runtime
        .eval(
            r#"
        def add(a: Int, b: Int) -> Int {
            a + b
        }
    "#,
        )
        .unwrap();

    let sig = runtime.get_function_signature("add");
    assert!(sig.is_some());

    let sig = sig.unwrap();
    assert_eq!(sig.name, "add");
    assert_eq!(sig.arity(), 2);
    assert!(!sig.is_llm);
}

#[test]
fn test_has_function() {
    let mut runtime = create_minimal_runtime();

    runtime.eval("def my_func() -> Int { 42 }").unwrap();

    assert!(runtime.has_function("my_func"));
    assert!(!runtime.has_function("nonexistent"));
}

// ============================================================
// Type Definition Tests
// ============================================================

#[test]
fn test_define_struct_type() {
    let mut runtime = create_minimal_runtime();

    runtime
        .eval("type Person { name: String, age: Int }")
        .unwrap();

    let schema = runtime.get_type("Person");
    assert!(schema.is_some());

    match schema.unwrap() {
        TypeSchema::Struct(s) => {
            assert_eq!(s.name, "Person");
            assert_eq!(s.fields.len(), 2);
        }
        _ => panic!("Expected Struct schema"),
    }
}

#[test]
fn test_define_enum_type() {
    let mut runtime = create_minimal_runtime();

    runtime.eval("enum Color { Red, Green, Blue }").unwrap();

    let schema = runtime.get_type("Color");
    assert!(schema.is_some());

    match schema.unwrap() {
        TypeSchema::Enum(e) => {
            assert_eq!(e.name, "Color");
            assert_eq!(e.variants, vec!["Red", "Green", "Blue"]);
        }
        _ => panic!("Expected Enum schema"),
    }
}

#[test]
fn test_get_types() {
    let mut runtime = create_minimal_runtime();

    runtime
        .eval(
            r#"
        type Person { name: String }
        enum Status { Active, Inactive }
    "#,
        )
        .unwrap();

    let types = runtime.get_types();
    assert_eq!(types.len(), 2);
}

#[test]
fn test_register_external_type() {
    let mut runtime = create_minimal_runtime();

    let schema = TypeSchema::Struct(StructSchema {
        name: "ExternalType".to_string(),
        fields: vec![FieldSchema {
            name: "value".to_string(),
            type_schema: TypeSchema::Int,
            optional: false,
            description: None,
        }],
        description: None,
    });

    runtime.register_type(schema).unwrap();

    let retrieved = runtime.get_type("ExternalType");
    assert!(retrieved.is_some());
}

#[test]
fn test_register_external_enum() {
    let mut runtime = create_minimal_runtime();

    let schema = TypeSchema::Enum(EnumSchema {
        name: "Priority".to_string(),
        variants: vec!["High".to_string(), "Medium".to_string(), "Low".to_string()],
        description: None,
    });

    runtime.register_type(schema).unwrap();

    let retrieved = runtime.get_type("Priority");
    assert!(retrieved.is_some());
}

// ============================================================
// Runtime Isolation Tests
// ============================================================

#[test]
fn test_runtime_global_isolation() {
    let mut runtime1 = create_minimal_runtime();
    let mut runtime2 = create_minimal_runtime();

    // Set different values in each runtime
    runtime1.set_global("x", LatticeValue::Int(100));
    runtime2.set_global("x", LatticeValue::Int(200));

    // They should be completely independent
    assert_eq!(runtime1.get_global("x"), Some(LatticeValue::Int(100)));
    assert_eq!(runtime2.get_global("x"), Some(LatticeValue::Int(200)));
}

#[test]
fn test_runtime_function_isolation() {
    let mut runtime1 = create_minimal_runtime();
    let mut runtime2 = create_minimal_runtime();

    // Define function in runtime1
    runtime1.eval("def add_ten(x: Int) -> Int { x + 10 }").unwrap();

    // runtime2 should NOT see add_ten
    let result = runtime2.eval("add_ten(5)");
    assert!(result.is_err(), "runtime2 should not see add_ten defined in runtime1");

    // runtime1 should still work
    let val = runtime1.eval("add_ten(5)").unwrap();
    assert_eq!(val, LatticeValue::Int(15));
}

#[test]
fn test_runtime_type_isolation() {
    let mut runtime1 = create_minimal_runtime();
    let mut runtime2 = create_minimal_runtime();

    // Define type in runtime1
    runtime1.eval("type Foo { x: Int }").unwrap();

    // runtime2 should NOT have the type
    assert!(runtime1.get_type("Foo").is_some());
    assert!(runtime2.get_type("Foo").is_none());

    // Define different type in runtime2
    runtime2.eval("type Bar { y: String }").unwrap();

    // Each should have only its own types
    assert!(runtime1.get_type("Bar").is_none());
    assert!(runtime2.get_type("Bar").is_some());
}

#[test]
fn test_runtime_eval_isolation() {
    let mut runtime1 = create_minimal_runtime();
    let mut runtime2 = create_minimal_runtime();

    // Set up state in runtime1
    runtime1.eval("let shared = 42").unwrap();

    // runtime2 should NOT see the variable
    let result = runtime2.eval("shared");
    assert!(result.is_err());

    // runtime1 should still have it
    assert_eq!(runtime1.get_global("shared"), Some(LatticeValue::Int(42)));
}

// ============================================================
// Reset Tests
// ============================================================

#[test]
fn test_reset_clears_state() {
    let mut runtime = create_minimal_runtime();

    // Set up state
    runtime.set_global("x", LatticeValue::Int(42));
    runtime.eval("type Foo { bar: String }").unwrap();
    runtime.eval("def my_func() -> Int { 1 }").unwrap();

    // Reset
    runtime.reset();

    // All state should be cleared
    assert!(runtime.get_global("x").is_none());
    assert!(runtime.get_type("Foo").is_none());
    assert!(!runtime.has_function("my_func"));
    assert!(runtime.function_names().is_empty());
}

#[test]
fn test_reset_allows_reuse() {
    let mut runtime = create_minimal_runtime();

    // First use
    runtime.eval("let x = 100").unwrap();
    assert_eq!(runtime.eval("x").unwrap(), LatticeValue::Int(100));

    // Reset
    runtime.reset();

    // Should be able to reuse
    runtime.eval("let x = 200").unwrap();
    assert_eq!(runtime.eval("x").unwrap(), LatticeValue::Int(200));
}

// ============================================================
// SharedRuntime Tests
// ============================================================

#[test]
fn test_shared_runtime_basic() {
    let runtime = create_minimal_runtime();
    let shared = SharedRuntime::new(runtime);

    let result = shared.eval("1 + 1").unwrap();
    assert_eq!(result, LatticeValue::Int(2));
}

#[test]
fn test_shared_runtime_clonable() {
    let runtime = create_minimal_runtime();
    let shared = SharedRuntime::new(runtime);
    let shared2 = shared.clone();

    // Both should work
    let result1 = shared.eval("1 + 1").unwrap();
    let result2 = shared2.eval("2 + 2").unwrap();

    assert_eq!(result1, LatticeValue::Int(2));
    assert_eq!(result2, LatticeValue::Int(4));
}

#[test]
fn test_shared_runtime_state_shared() {
    let runtime = create_minimal_runtime();
    let shared = SharedRuntime::new(runtime);

    // Set global through one reference
    shared.set_global("count", LatticeValue::Int(0)).unwrap();

    // Read from the same reference
    let result = shared.eval("count + 1").unwrap();
    assert_eq!(result, LatticeValue::Int(1));
}

#[test]
fn test_shared_runtime_call() {
    let runtime = create_minimal_runtime();
    let shared = SharedRuntime::new(runtime);

    // Define function
    shared.eval("def square(x: Int) -> Int { x * x }").unwrap();

    // Call via shared runtime
    let result = shared.call("square", vec![LatticeValue::Int(5)]).unwrap();
    assert_eq!(result, LatticeValue::Int(25));
}

#[test]
fn test_shared_runtime_get_types() {
    let runtime = create_minimal_runtime();
    let shared = SharedRuntime::new(runtime);

    shared.eval("type Person { name: String }").unwrap();

    let types = shared.get_types().unwrap();
    assert_eq!(types.len(), 1);
}

// ============================================================
// Error Handling Tests
// ============================================================

#[test]
fn test_syntax_error() {
    let mut runtime = create_minimal_runtime();

    let result = runtime.eval("1 +");
    assert!(result.is_err());
}

#[test]
fn test_undefined_variable_error() {
    let mut runtime = create_minimal_runtime();

    let result = runtime.eval("undefined_var");
    assert!(result.is_err());
}

#[test]
fn test_type_error() {
    let mut runtime = create_minimal_runtime();

    // String + Int should be a type error
    let result = runtime.eval(r#""hello" + 5"#);
    assert!(result.is_err());
}

// ============================================================
// LatticeValue Marshaling Tests
// ============================================================

#[test]
fn test_lattice_value_from_conversions() {
    // From primitives
    assert_eq!(LatticeValue::from(true), LatticeValue::Bool(true));
    assert_eq!(LatticeValue::from(42i64), LatticeValue::Int(42));
    assert_eq!(LatticeValue::from(42i32), LatticeValue::Int(42));
    assert_eq!(LatticeValue::from(3.14f64), LatticeValue::Float(3.14));
    assert_eq!(
        LatticeValue::from("hello"),
        LatticeValue::String("hello".to_string())
    );
    assert_eq!(
        LatticeValue::from("hello".to_string()),
        LatticeValue::String("hello".to_string())
    );
    assert_eq!(LatticeValue::from(()), LatticeValue::Null);

    // From vec
    let list: LatticeValue = vec![1i64, 2i64, 3i64].into();
    assert_eq!(
        list,
        LatticeValue::List(vec![
            LatticeValue::Int(1),
            LatticeValue::Int(2),
            LatticeValue::Int(3),
        ])
    );
}

#[test]
fn test_lattice_value_accessors() {
    let null = LatticeValue::Null;
    assert!(null.is_null());
    assert!(null.as_bool().is_none());

    let b = LatticeValue::Bool(true);
    assert!(!b.is_null());
    assert_eq!(b.as_bool(), Some(true));

    let i = LatticeValue::Int(42);
    assert_eq!(i.as_int(), Some(42));

    let f = LatticeValue::Float(3.14);
    assert_eq!(f.as_float(), Some(3.14));

    let s = LatticeValue::String("test".to_string());
    assert_eq!(s.as_str(), Some("test"));

    let list = LatticeValue::List(vec![LatticeValue::Int(1)]);
    assert_eq!(list.as_list().map(|l| l.len()), Some(1));

    let map = LatticeValue::Map(vec![("key".to_string(), LatticeValue::Int(42))]);
    assert_eq!(map.as_map().map(|m| m.len()), Some(1));
    assert_eq!(map.get("key"), Some(&LatticeValue::Int(42)));
}

#[test]
fn test_lattice_value_display() {
    assert_eq!(format!("{}", LatticeValue::Null), "null");
    assert_eq!(format!("{}", LatticeValue::Bool(true)), "true");
    assert_eq!(format!("{}", LatticeValue::Int(42)), "42");
    assert_eq!(format!("{}", LatticeValue::Float(3.14)), "3.14");
    assert_eq!(
        format!("{}", LatticeValue::String("hello".to_string())),
        "\"hello\""
    );

    let list = LatticeValue::List(vec![LatticeValue::Int(1), LatticeValue::Int(2)]);
    assert_eq!(format!("{}", list), "[1, 2]");

    let map = LatticeValue::Map(vec![
        ("a".to_string(), LatticeValue::Int(1)),
        ("b".to_string(), LatticeValue::Int(2)),
    ]);
    assert_eq!(format!("{}", map), "{\"a\": 1, \"b\": 2}");
}

#[test]
fn test_lattice_value_serde() {
    use serde_json;

    let values = vec![
        LatticeValue::Null,
        LatticeValue::Bool(true),
        LatticeValue::Int(42),
        LatticeValue::Float(3.14),
        LatticeValue::String("hello".to_string()),
        LatticeValue::List(vec![LatticeValue::Int(1), LatticeValue::Int(2)]),
        LatticeValue::Map(vec![("key".to_string(), LatticeValue::Int(42))]),
    ];

    for value in values {
        let json = serde_json::to_string(&value).unwrap();
        let back: LatticeValue = serde_json::from_str(&json).unwrap();
        assert_eq!(value, back);
    }
}

// ============================================================
// Provider Injection Tests
// ============================================================

/// Mock LLM provider for testing
struct MockLlmProvider {
    response: String,
}

impl MockLlmProvider {
    fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
        }
    }
}

impl LlmProvider for MockLlmProvider {
    fn call(&self, _request: LlmRequest) -> Result<LlmResponse, LlmError> {
        Ok(LlmResponse {
            content: self.response.clone(),
            usage: None,
        })
    }
}

#[test]
fn test_no_llm_provider_returns_error() {
    let provider = NoLlmProvider;
    let request = LlmRequest::new(
        "https://api.example.com".to_string(),
        "model".to_string(),
        "key".to_string(),
        "prompt".to_string(),
    );

    let result = provider.call(request);
    assert!(result.is_err());
    match result {
        Err(LlmError::NotConfigured(_)) => (),
        _ => panic!("Expected NotConfigured error"),
    }
}

#[test]
fn test_no_sql_provider_returns_error() {
    use lattice::runtime::SqlProvider;

    let provider = NoSqlProvider;
    let result = provider.query("SELECT 1");
    assert!(result.is_err());
}

#[test]
fn test_custom_llm_provider() {
    let mock = MockLlmProvider::new("Mock response");

    let request = LlmRequest::new(
        "https://api.example.com".to_string(),
        "model".to_string(),
        "key".to_string(),
        "test prompt".to_string(),
    );

    let response = mock.call(request).unwrap();
    assert_eq!(response.content, "Mock response");
}

#[test]
fn test_runtime_with_custom_llm_provider() {
    let mock_provider = MockLlmProvider::new("Test response");

    let built = RuntimeBuilder::new()
        .with_llm_provider(mock_provider)
        .without_sql()
        .build()
        .expect("Build should succeed");

    // The provider is accessible through the built runtime
    let request = LlmRequest::new(
        "https://api.example.com".to_string(),
        "model".to_string(),
        "key".to_string(),
        "test".to_string(),
    );

    let response = built.llm_provider().call(request).unwrap();
    assert_eq!(response.content, "Test response");
}

// ============================================================
// TypeSchema Tests
// ============================================================

#[test]
fn test_type_schema_primitives() {
    assert_eq!(format!("{}", TypeSchema::Int), "Int");
    assert_eq!(format!("{}", TypeSchema::Float), "Float");
    assert_eq!(format!("{}", TypeSchema::String), "String");
    assert_eq!(format!("{}", TypeSchema::Bool), "Bool");
    assert_eq!(format!("{}", TypeSchema::Any), "Any");
}

#[test]
fn test_type_schema_list() {
    let list_type = TypeSchema::List(Box::new(TypeSchema::Int));
    assert_eq!(format!("{}", list_type), "[Int]");
}

#[test]
fn test_type_schema_map() {
    let map_type = TypeSchema::Map {
        key: Box::new(TypeSchema::String),
        value: Box::new(TypeSchema::Int),
    };
    assert_eq!(format!("{}", map_type), "Map<String, Int>");
}

#[test]
fn test_function_signature_display() {
    let sig = FunctionSignature::new(
        "add".to_string(),
        vec![
            ParameterSchema {
                name: "a".to_string(),
                type_schema: TypeSchema::Int,
            },
            ParameterSchema {
                name: "b".to_string(),
                type_schema: TypeSchema::Int,
            },
        ],
        TypeSchema::Int,
        false,
    );

    assert_eq!(sig.to_string(), "fn add(a: Int, b: Int) -> Int");
}

#[test]
fn test_function_signature_llm() {
    let sig = FunctionSignature::new(
        "analyze".to_string(),
        vec![ParameterSchema {
            name: "text".to_string(),
            type_schema: TypeSchema::String,
        }],
        TypeSchema::String,
        true,
    );

    assert!(sig.is_llm);
    assert!(sig.is_async);
    assert_eq!(
        sig.to_string(),
        "async fn analyze(text: String) -> String [LLM]"
    );
}

// ============================================================
// Complex Scenario Tests
// ============================================================

#[test]
fn test_multi_cell_notebook_scenario() {
    let mut runtime = create_minimal_runtime();

    // Simulate notebook cells
    // Cell 1: Define a type
    runtime.eval("type Point { x: Int, y: Int }").unwrap();

    // Cell 2: Define a function
    runtime
        .eval(
            r#"
        def distance_from_origin(p: Point) -> Float {
            // Simplified - just sum the coordinates
            float(p.x) + float(p.y)
        }
    "#,
        )
        .unwrap();

    // Cell 3: Create a value and use the function
    runtime.eval("let my_point = Point { x: 3, y: 4 }").unwrap();

    let result = runtime.eval("distance_from_origin(my_point)").unwrap();
    assert_eq!(result, LatticeValue::Float(7.0));
}

#[test]
fn test_iterative_computation() {
    let mut runtime = create_minimal_runtime();

    // Define factorial
    runtime
        .eval(
            r#"
        def factorial(n: Int) -> Int {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
    "#,
        )
        .unwrap();

    assert_eq!(
        runtime.call("factorial", vec![LatticeValue::Int(5)]).unwrap(),
        LatticeValue::Int(120)
    );
}

#[test]
fn test_data_transformation_pipeline() {
    let mut runtime = create_minimal_runtime();

    // Set up initial data
    runtime.set_global(
        "data",
        LatticeValue::List(vec![
            LatticeValue::Int(1),
            LatticeValue::Int(2),
            LatticeValue::Int(3),
            LatticeValue::Int(4),
            LatticeValue::Int(5),
        ]),
    );

    // Transform: filter even numbers, sum them
    // Note: Lattice doesn't support list concatenation with +, so we compute directly
    let result = runtime
        .eval(
            r#"
        let sum_of_evens = 0
        for n in data {
            if n % 2 == 0 {
                sum_of_evens = sum_of_evens + n
            }
        }
        sum_of_evens
    "#,
        )
        .unwrap();

    // evens = [2, 4], sum = 6
    assert_eq!(result, LatticeValue::Int(6));
}

#[test]
fn test_string_processing() {
    let mut runtime = create_minimal_runtime();

    runtime
        .eval(
            r#"
        def process_name(first: String, last: String) -> String {
            last + ", " + first
        }
    "#,
        )
        .unwrap();

    let result = runtime
        .call(
            "process_name",
            vec![
                LatticeValue::String("John".to_string()),
                LatticeValue::String("Doe".to_string()),
            ],
        )
        .unwrap();

    assert_eq!(
        result,
        LatticeValue::String("Doe, John".to_string())
    );
}
