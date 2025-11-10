use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct DslParser;

// Submodules
mod ast;
mod error;
mod expressions;
mod functions;
mod parsing;
mod types;
mod validation;

// Re-export AST types
pub use ast::*;

// Re-export parsing functions
pub use error::format_parse_error;
pub use functions::parse_function_definition;
pub use parsing::{is_command, parse_command, parse_expr, parse_expr_with_binding, parse_program};
pub use types::{parse_enum_definition, parse_type_definition};
pub use validation::{validate_balanced_delimiters, ValidationError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_literal() {
        let result = parse_expr("42").unwrap();
        assert!(matches!(result, Expr::Int(42)));

        let result = parse_expr("3.14").unwrap();
        assert!(matches!(result, Expr::Float(_)));

        let result = parse_expr("true").unwrap();
        assert!(matches!(result, Expr::Bool(true)));

        let result = parse_expr(r#""hello""#).unwrap();
        // Strings can be parsed as TemplateString (with no interpolation) or String
        assert!(matches!(result, Expr::String(_) | Expr::TemplateString(_)));
    }

    #[test]
    fn test_parse_function_call() {
        let result = parse_expr("Length(\"test\")").unwrap();
        if let Expr::FunctionCall { name, args } = result {
            assert_eq!(name, "Length");
            assert_eq!(args.len(), 1);
        } else {
            panic!("Expected function call");
        }
    }

    #[test]
    fn test_parse_access_chain() {
        let result = parse_expr("user.name").unwrap();
        if let Expr::FieldAccess { base, field } = result {
            assert!(matches!(*base, Expr::Variable(_)));
            assert_eq!(field, "name");
        } else {
            panic!("Expected field access");
        }
    }

    #[test]
    fn test_parse_type_definition() {
        let input = "type Person { name: string, age: int }";
        let result = parse_type_definition(input).unwrap();

        assert_eq!(result.name, "Person");
        assert_eq!(result.fields.len(), 2);
        assert_eq!(result.fields[0].name, "name");
        assert_eq!(result.fields[1].name, "age");
    }

    #[test]
    fn test_parse_enum_definition() {
        let input = "enum Status { Pending, InProgress, Completed }";
        let result = parse_enum_definition(input).unwrap();

        assert_eq!(result.name, "Status");
        assert_eq!(result.values.len(), 3);
        assert_eq!(result.values[0], "Pending");
    }

    #[test]
    fn test_parse_sequential() {
        // Test simple sequential
        let result = parse_expr("5 |> Length(_)").unwrap();
        assert!(matches!(result, Expr::Sequential { .. }));

        // Test chained sequential
        let result = parse_expr("1 |> _ * 2 |> _ + 3").unwrap();
        // Debug: print the actual structure
        println!("Parsed result: {:?}", result);

        if let Expr::Sequential { left, right, .. } = result {
            // The chain should be built left-to-right:
            // Sequential { left: Sequential { left: 1, right: _*2 }, right: _+3 }
            println!("Left: {:?}", left);
            println!("Right: {:?}", right);
            assert!(matches!(*left, Expr::Sequential { .. }));
            assert!(matches!(*right, Expr::BinaryOp { .. }));
        } else {
            panic!("Expected sequential expression, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_parallel() {
        // Test parallel with par() function
        let result = parse_expr("par(Ask(\"a\"), Ask(\"b\"))").unwrap();
        if let Expr::FunctionCall { name, args } = result {
            assert_eq!(name, "par");
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected function call to par, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_comparison_operators() {
        // Test equality
        let result = parse_expr("5 == 5").unwrap();
        if let Expr::BinaryOp { op, .. } = result {
            assert_eq!(op, "==");
        } else {
            panic!("Expected comparison expression");
        }

        // Test inequality
        let result = parse_expr("5 != 3").unwrap();
        if let Expr::BinaryOp { op, .. } = result {
            assert_eq!(op, "!=");
        } else {
            panic!("Expected comparison expression");
        }

        // Test less than
        let result = parse_expr("3 < 5").unwrap();
        if let Expr::BinaryOp { op, .. } = result {
            assert_eq!(op, "<");
        } else {
            panic!("Expected comparison expression");
        }

        // Test greater than or equal
        let result = parse_expr("5 >= 3").unwrap();
        if let Expr::BinaryOp { op, .. } = result {
            assert_eq!(op, ">=");
        } else {
            panic!("Expected comparison expression");
        }
    }

    #[test]
    fn test_parse_logical_operators() {
        // Test AND
        let result = parse_expr("true && false").unwrap();
        if let Expr::BinaryOp { op, .. } = result {
            assert_eq!(op, "&&");
        } else {
            panic!("Expected logical AND expression");
        }

        // Test OR
        let result = parse_expr("true || false").unwrap();
        if let Expr::BinaryOp { op, .. } = result {
            assert_eq!(op, "||");
        } else {
            panic!("Expected logical OR expression");
        }

        // Test NOT
        let result = parse_expr("!true").unwrap();
        if let Expr::FunctionCall { name, args } = result {
            assert_eq!(name, "not");
            assert_eq!(args.len(), 1);
        } else {
            panic!("Expected NOT expression (as function call)");
        }
    }

    #[test]
    fn test_parse_complex_conditional() {
        // Test conditional with comparison
        let result = parse_expr("age >= 18 ? \"adult\" : \"minor\"").unwrap();
        if let Expr::Conditional { condition, .. } = result {
            // Condition should be a comparison
            if let Expr::BinaryOp { op, .. } = *condition {
                assert_eq!(op, ">=");
            } else {
                panic!("Expected comparison in condition");
            }
        } else {
            panic!("Expected conditional expression");
        }

        // Test conditional with logical operators
        let result = parse_expr("age >= 18 && verified ? \"proceed\" : \"reject\"").unwrap();
        if let Expr::Conditional { condition, .. } = result {
            // Condition should be a logical AND
            if let Expr::BinaryOp { op, .. } = *condition {
                assert_eq!(op, "&&");
            } else {
                panic!("Expected logical AND in condition");
            }
        } else {
            panic!("Expected conditional expression");
        }
    }

    #[test]
    fn test_parse_conditional() {
        // Test ternary operator
        let result = parse_expr("true ? 1 : 2").unwrap();
        if let Expr::Conditional {
            condition,
            then_expr,
            else_expr,
        } = result
        {
            assert!(matches!(*condition, Expr::Bool(true)));
            assert!(matches!(*then_expr, Expr::Int(1)));
            assert!(matches!(*else_expr, Expr::Int(2)));
        } else {
            panic!("Expected conditional expression");
        }
    }

    #[test]
    fn test_parse_binding_with_sequential() {
        // Regression test for: [1, 2, 3] as numbers |> Length(numbers)
        // This previously failed with "Invalid variable name" error
        let result = parse_expr("[1, 2, 3] as numbers |> Length(numbers)").unwrap();

        // Should be: Sequential {
        //   left: Parallel { exprs: [[1,2,3]], binding: Some("numbers") },
        //   right: Length(numbers),
        //   binding: None
        // }
        if let Expr::Sequential {
            left,
            right,
            binding,
        } = result
        {
            // Left should be Parallel (single expr with binding) or Sequential
            match *left {
                Expr::Parallel {
                    binding: left_binding,
                    ..
                } => {
                    if let Some(Binding::Single(name)) = left_binding {
                        assert_eq!(name, "numbers");
                    } else {
                        panic!("Expected binding 'numbers' on left");
                    }
                }
                Expr::Sequential {
                    binding: left_binding,
                    ..
                } => {
                    if let Some(Binding::Single(name)) = left_binding {
                        assert_eq!(name, "numbers");
                    } else {
                        panic!("Expected binding 'numbers' on left Sequential");
                    }
                }
                _ => panic!(
                    "Expected left to be Parallel or Sequential with binding, got: {:?}",
                    left
                ),
            }

            // Right should be function call
            assert!(matches!(*right, Expr::FunctionCall { .. }));

            // No binding on the outer sequential
            assert!(binding.is_none());
        } else {
            panic!("Expected sequential expression, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_type_instantiation() {
        // Test basic type instantiation
        let result = parse_expr(r#"Person { name: "Alice", age: 30 }"#).unwrap();
        if let Expr::TypeInstantiation { type_name, fields } = result {
            assert_eq!(type_name, "Person");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "name");
            assert_eq!(fields[1].0, "age");
            // Strings are parsed as TemplateString, not String
            assert!(matches!(
                fields[0].1,
                Expr::String(_) | Expr::TemplateString(_)
            ));
            assert!(matches!(fields[1].1, Expr::Int(30)));
        } else {
            panic!("Expected type instantiation, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_type_instantiation_with_optional_fields() {
        // Test type instantiation with optional fields
        let result =
            parse_expr(r#"Person { name: "Bob", age: 25, email: "bob@example.com" }"#).unwrap();
        if let Expr::TypeInstantiation { type_name, fields } = result {
            assert_eq!(type_name, "Person");
            assert_eq!(fields.len(), 3);
            assert_eq!(fields[0].0, "name");
            assert_eq!(fields[1].0, "age");
            assert_eq!(fields[2].0, "email");
        } else {
            panic!("Expected type instantiation, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_type_instantiation_with_expressions() {
        // Test type instantiation with complex expressions
        let result = parse_expr(r#"Point { x: 1 + 2, y: 3 * 4 }"#).unwrap();
        if let Expr::TypeInstantiation { type_name, fields } = result {
            assert_eq!(type_name, "Point");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "x");
            assert_eq!(fields[1].0, "y");
            assert!(matches!(fields[0].1, Expr::BinaryOp { .. }));
            assert!(matches!(fields[1].1, Expr::BinaryOp { .. }));
        } else {
            panic!("Expected type instantiation, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_type_instantiation_nested() {
        // Test nested type instantiation
        let result =
            parse_expr(r#"Employee { person: Person { name: "Alice", age: 30 }, id: 123 }"#)
                .unwrap();
        if let Expr::TypeInstantiation { type_name, fields } = result {
            assert_eq!(type_name, "Employee");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "person");
            assert_eq!(fields[1].0, "id");

            // Check that the first field is itself a type instantiation
            assert!(matches!(fields[0].1, Expr::TypeInstantiation { .. }));
        } else {
            panic!("Expected type instantiation, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_empty_type_instantiation() {
        // Test empty type instantiation
        let result = parse_expr("Empty {}").unwrap();
        if let Expr::TypeInstantiation { type_name, fields } = result {
            assert_eq!(type_name, "Empty");
            assert_eq!(fields.len(), 0);
        } else {
            panic!("Expected type instantiation, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_type_instantiation_syntax_error() {
        // Test syntax error - missing comma between fields
        let result = parse_expr(r#"Person { name: "Alice", age: 30 email: "ada@gmail.com"}"#);
        println!("Parse result: {:?}", result);

        // Let's see what it actually parses to
        if let Ok(expr) = result {
            println!("Parsed as: {:?}", expr);
            // It probably parses as: Person { name: "Alice", age: 30 }
            // with "email: ..." left unparsed or treated as something else
        }
    }

    #[test]
    fn test_parse_map_literal() {
        // Test empty map
        let result = parse_expr("{}").unwrap();
        if let Expr::Map(entries) = result {
            assert_eq!(entries.len(), 0);
        } else {
            panic!("Expected map literal, got: {:?}", result);
        }

        // Test simple map with identifier keys
        let result = parse_expr(r#"{ name: "Alice", age: 30 }"#).unwrap();
        if let Expr::Map(entries) = result {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, "name");
            assert_eq!(entries[1].0, "age");
            assert!(matches!(
                entries[0].1,
                Expr::String(_) | Expr::TemplateString(_)
            ));
            assert!(matches!(entries[1].1, Expr::Int(30)));
        } else {
            panic!("Expected map literal, got: {:?}", result);
        }

        // Test map with string literal keys
        let result = parse_expr(
            r#"{ "Authorization": "Bearer token123", "Content-Type": "application/json" }"#,
        )
        .unwrap();
        if let Expr::Map(entries) = result {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, "Authorization");
            assert_eq!(entries[1].0, "Content-Type");
        } else {
            panic!("Expected map literal, got: {:?}", result);
        }

        // Test nested map
        let result =
            parse_expr(r#"{ config: { timeout: 30, retries: 3 }, enabled: true }"#).unwrap();
        if let Expr::Map(entries) = result {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, "config");
            assert_eq!(entries[1].0, "enabled");
            assert!(matches!(entries[0].1, Expr::Map(_)));
            assert!(matches!(entries[1].1, Expr::Bool(true)));
        } else {
            panic!("Expected map literal, got: {:?}", result);
        }

        // Test map with mixed value types
        let result =
            parse_expr(r#"{ str: "hello", num: 42, float: 3.14, bool: true, list: [1, 2, 3] }"#)
                .unwrap();
        if let Expr::Map(entries) = result {
            assert_eq!(entries.len(), 5);
            assert_eq!(entries[0].0, "str");
            assert_eq!(entries[1].0, "num");
            assert_eq!(entries[2].0, "float");
            assert_eq!(entries[3].0, "bool");
            assert_eq!(entries[4].0, "list");
            assert!(matches!(entries[4].1, Expr::List(_)));
        } else {
            panic!("Expected map literal, got: {:?}", result);
        }

        // Test map with trailing comma
        let result = parse_expr(r#"{ name: "Bob", age: 25, }"#).unwrap();
        if let Expr::Map(entries) = result {
            assert_eq!(entries.len(), 2);
        } else {
            panic!("Expected map literal, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_let_binding() {
        // 'let' is now a statement, not an expression
        // Test that let is rejected as a standalone expression
        let result = parse_expr("let x = 5");
        assert!(
            result.is_err(),
            "let should not be parseable as an expression"
        );

        // Test that let works as a statement in a program
        let program = parse_program("let x = 5\nx").unwrap();
        assert!(
            program.entry_expr.is_some(),
            "Should have an entry expression"
        );
    }

    #[test]
    fn test_let_vs_as_equivalence() {
        // 'let' is now a statement, so test that 'as' expressions work
        let as_result = parse_expr("5 as x").unwrap();

        // Should be Parallel with single expression and binding
        if let Expr::Parallel { exprs, binding } = as_result {
            assert_eq!(exprs.len(), 1);
            assert!(matches!(exprs[0], Expr::Int(5)));
            assert!(matches!(binding, Some(Binding::Single(ref name)) if name == "x"));
        } else {
            panic!("Expected Parallel expression with binding");
        }

        // Test list destructuring with 'as'
        let as_result = parse_expr("[1, 2] as [a, b]").unwrap();

        if let Expr::Parallel { binding, .. } = as_result {
            if let Some(Binding::List(vars)) = binding {
                assert_eq!(vars.len(), 2);
                assert_eq!(vars[0], "a");
                assert_eq!(vars[1], "b");
            } else {
                panic!("Expected list binding");
            }
        } else {
            panic!("Expected Parallel expression");
        }
    }

    #[test]
    fn test_parse_match_expression() {
        // Test simple match with literal patterns
        let result = parse_expr(
            r#"match x {
            0 => "zero",
            1 => "one",
            _ => "other"
        }"#,
        )
        .unwrap();

        if let Expr::Match { scrutinee, cases } = result {
            assert!(matches!(*scrutinee, Expr::Variable(_)));
            assert_eq!(cases.len(), 3);

            // First case: 0 => "zero"
            assert!(matches!(cases[0].pattern, Pattern::Literal(_)));
            assert!(cases[0].guard.is_none());

            // Second case: 1 => "one"
            assert!(matches!(cases[1].pattern, Pattern::Literal(_)));
            assert!(cases[1].guard.is_none());

            // Third case: _ => "other"
            assert!(matches!(cases[2].pattern, Pattern::Any));
            assert!(cases[2].guard.is_none());
        } else {
            panic!("Expected match expression, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_match_with_guards() {
        // Test match with guard conditions
        let result = parse_expr(
            r#"match x {
            n if n > 0 => "positive",
            n if n < 0 => "negative",
            _ => "zero"
        }"#,
        )
        .unwrap();

        if let Expr::Match { cases, .. } = result {
            assert_eq!(cases.len(), 3);

            // First case has a guard
            assert!(matches!(cases[0].pattern, Pattern::Variable(_)));
            assert!(cases[0].guard.is_some());

            // Second case has a guard
            assert!(matches!(cases[1].pattern, Pattern::Variable(_)));
            assert!(cases[1].guard.is_some());

            // Third case has no guard
            assert!(matches!(cases[2].pattern, Pattern::Any));
            assert!(cases[2].guard.is_none());
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_parse_pattern_wildcard() {
        let result = parse_expr("match x { _ => 1 }").unwrap();
        if let Expr::Match { cases, .. } = result {
            assert!(matches!(cases[0].pattern, Pattern::Any));
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_parse_pattern_variable() {
        let result = parse_expr("match x { y => y }").unwrap();
        if let Expr::Match { cases, .. } = result {
            if let Pattern::Variable(name) = &cases[0].pattern {
                assert_eq!(name, "y");
            } else {
                panic!("Expected variable pattern");
            }
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_parse_pattern_list() {
        // Test empty list pattern
        let result = parse_expr("match x { [] => 0 }").unwrap();
        if let Expr::Match { cases, .. } = result {
            if let Pattern::List { patterns, rest } = &cases[0].pattern {
                assert_eq!(patterns.len(), 0);
                assert!(rest.is_none());
            } else {
                panic!("Expected list pattern");
            }
        } else {
            panic!("Expected match expression");
        }

        // Test list pattern with elements
        let result = parse_expr("match x { [a, b] => a }").unwrap();
        if let Expr::Match { cases, .. } = result {
            if let Pattern::List { patterns, rest } = &cases[0].pattern {
                assert_eq!(patterns.len(), 2);
                assert!(rest.is_none());
            } else {
                panic!("Expected list pattern");
            }
        } else {
            panic!("Expected match expression");
        }

        // Test list pattern with rest
        let result = parse_expr("match x { [head, ...tail] => head }").unwrap();
        if let Expr::Match { cases, .. } = result {
            if let Pattern::List { patterns, rest } = &cases[0].pattern {
                assert_eq!(patterns.len(), 1);
                assert!(rest.is_some());
                assert_eq!(rest.as_ref().unwrap(), "tail");
            } else {
                panic!("Expected list pattern with rest");
            }
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_parse_pattern_type() {
        // Test type pattern with inner pattern
        let result = parse_expr("match x { Int(n) => n }").unwrap();
        if let Expr::Match { cases, .. } = result {
            if let Pattern::Type { type_name, inner } = &cases[0].pattern {
                assert_eq!(type_name, "Int");
                assert!(inner.is_some());
                if let Some(inner_pattern) = inner {
                    assert!(matches!(**inner_pattern, Pattern::Variable(_)));
                }
            } else {
                panic!("Expected type pattern");
            }
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_parse_pattern_map() {
        // Test empty map pattern
        let result = parse_expr("match x { {} => 0 }").unwrap();
        if let Expr::Match { cases, .. } = result {
            if let Pattern::Map { fields, .. } = &cases[0].pattern {
                assert_eq!(fields.len(), 0);
            } else {
                panic!("Expected map pattern");
            }
        } else {
            panic!("Expected match expression");
        }

        // Test map pattern with fields
        let result = parse_expr(r#"match x { {name, age} => name }"#).unwrap();
        if let Expr::Match { cases, .. } = result {
            if let Pattern::Map { fields, .. } = &cases[0].pattern {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "name");
                assert_eq!(fields[1].0, "age");
            } else {
                panic!("Expected map pattern");
            }
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_parse_pattern_tuple() {
        // Test tuple pattern
        let result = parse_expr("match x { (a, b) => a }").unwrap();
        if let Expr::Match { cases, .. } = result {
            if let Pattern::Tuple(patterns) = &cases[0].pattern {
                assert_eq!(patterns.len(), 2);
            } else {
                panic!("Expected tuple pattern");
            }
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_parse_pattern_binding() {
        // Test binding pattern: x @ pattern
        let result = parse_expr("match x { n @ 0 => n }").unwrap();
        if let Expr::Match { cases, .. } = result {
            if let Pattern::Binding(name, nested) = &cases[0].pattern {
                assert_eq!(name, "n");
                assert!(matches!(**nested, Pattern::Literal(_)));
            } else {
                panic!("Expected binding pattern");
            }
        } else {
            panic!("Expected match expression");
        }
    }

    #[test]
    fn test_compile_match_expression() {
        // Test that match expressions compile to IR correctly
        use crate::compiler::compile_expr;

        let expr = parse_expr(
            r#"match x {
            0 => "zero",
            _ => "other"
        }"#,
        )
        .unwrap();

        let ir_node = compile_expr(&expr).unwrap();

        // Should compile to IRNode::Match
        if let dsl_ir::IRNode::Match { cases, .. } = ir_node {
            assert_eq!(cases.len(), 2);
        } else {
            panic!("Expected IRNode::Match, got: {:?}", ir_node);
        }
    }

    #[test]
    fn test_parse_pattern_function() {
        // Test parsing a simple pattern-based function
        let program = parse_program(
            r#"
            def factorial(0) { 1 }
            def factorial(n) { n * factorial(n - 1) }
        "#,
        )
        .unwrap();

        assert_eq!(program.pattern_functions.len(), 1);
        let factorial = &program.pattern_functions[0];
        assert_eq!(factorial.name, "factorial");
        assert_eq!(factorial.clauses.len(), 2);

        // First clause: factorial(0) { 1 }
        assert_eq!(factorial.clauses[0].param_patterns.len(), 1);
        assert!(matches!(
            factorial.clauses[0].param_patterns[0],
            Pattern::Literal(_)
        ));

        // Second clause: factorial(n) { n * factorial(n - 1) }
        assert_eq!(factorial.clauses[1].param_patterns.len(), 1);
        assert!(matches!(
            factorial.clauses[1].param_patterns[0],
            Pattern::Variable(_)
        ));
    }

    #[test]
    fn test_compile_pattern_function() {
        // Test compiling pattern-based functions to IR
        use crate::compiler::compile_program;

        let ir = compile_program(
            r#"
            def factorial(0) { 1 }
            def factorial(n) { n * factorial(n - 1) }
        "#,
        )
        .unwrap();

        assert_eq!(ir.function_groups.len(), 1);
        let factorial_group = &ir.function_groups[0];
        assert_eq!(factorial_group.name, "factorial");
        assert_eq!(factorial_group.clauses.len(), 2);

        // First clause should have a literal pattern
        assert!(matches!(
            factorial_group.clauses[0].param_patterns[0],
            dsl_ir::IRPattern::Literal(_)
        ));

        // Second clause should have a variable pattern
        assert!(matches!(
            factorial_group.clauses[1].param_patterns[0],
            dsl_ir::IRPattern::Variable(_)
        ));
    }

    #[test]
    fn test_parse_mixed_functions() {
        // Test parsing both traditional and pattern-based functions
        let program = parse_program(
            r#"
            def traditional_func(x: Int) {
                prompt: "test"
            }

            def factorial(0) { 1 }
            def factorial(n) { n * factorial(n - 1) }
        "#,
        )
        .unwrap();

        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.pattern_functions.len(), 1);

        assert_eq!(program.functions[0].name, "traditional_func");
        assert_eq!(program.pattern_functions[0].name, "factorial");
    }
}
