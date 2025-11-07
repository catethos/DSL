use anyhow::Result;
use dsl_ir::{
    IRBinding, IRExecution, IRFunction, IRFunctionClause, IRFunctionGroup, IRMatchCase, IRNode,
    IRPattern, IRProperty, IRTemplateSegment, IR,
};
use std::collections::HashMap;

use crate::parser::{
    parse_expr, parse_program, Binding, Expr, FunctionDef, FunctionExecution, MatchCase, Pattern,
    PatternFunctionClause, PatternFunctionDef, Program, PropertyValue, TemplateSegment,
};

/// Compile DSL source code to IR
/// Handles both full programs (with types, enums, functions) and standalone expressions
pub fn compile_to_ir(source: &str) -> Result<IR> {
    // Try parsing as a program first (handles types, enums, and functions)
    if let Ok(program) = parse_program(source) {
        // Check if the program has any meaningful content (declarations or entry expression)
        let has_content = !program.types.is_empty()
            || !program.enums.is_empty()
            || !program.functions.is_empty()
            || !program.pattern_functions.is_empty()
            || program.entry_expr.is_some();

        if has_content {
            return compile_program_to_ir(&program);
        }
        // If parse_program succeeded but found nothing, fall through to try parse_expr
    }

    // Fall back to parsing as a standalone expression
    let expr = parse_expr(source).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
    let entry_expr = compile_expr(&expr)?;

    Ok(IR {
        version: "0.1.0".to_string(),
        types: Vec::new(),
        enums: Vec::new(),
        functions: Vec::new(),
        function_groups: Vec::new(),
        agents: Vec::new(),
        entry_expr,
    })
}

/// Compile a complete program to IR
pub fn compile_program(source: &str) -> Result<IR> {
    let program = parse_program(source).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    compile_program_to_ir(&program)
}

/// Compile a parsed program to IR
pub fn compile_program_to_ir(program: &Program) -> Result<IR> {
    // Compile traditional functions
    let functions: Result<Vec<IRFunction>> = program
        .functions
        .iter()
        .map(compile_function)
        .collect();
    let functions = functions?;

    // Compile pattern-based functions to function groups
    let function_groups: Result<Vec<IRFunctionGroup>> = program
        .pattern_functions
        .iter()
        .map(compile_pattern_function)
        .collect();
    let function_groups = function_groups?;

    // Compile entry expression if present
    let entry_expr = if let Some(expr) = &program.entry_expr {
        compile_expr(expr)?
    } else {
        IRNode::Int(0) // Default: return 0 if no entry expression
    };

    Ok(IR {
        version: "0.1.0".to_string(),
        types: program.types.clone(),
        enums: program.enums.clone(),
        functions,
        function_groups,
        agents: Vec::new(),
        entry_expr,
    })
}

/// Compile an expression to IR node
pub fn compile_expr(expr: &Expr) -> Result<IRNode> {
    match expr {
        Expr::String(s) => Ok(IRNode::String(s.clone())),

        Expr::TemplateString(segments) => {
            let ir_segments = segments
                .iter()
                .map(compile_template_segment)
                .collect::<Result<Vec<_>>>()?;
            Ok(IRNode::TemplateString(ir_segments))
        }

        Expr::Int(i) => Ok(IRNode::Int(*i)),
        Expr::Float(f) => Ok(IRNode::Float(*f)),
        Expr::Bool(b) => Ok(IRNode::Bool(*b)),

        Expr::List(items) => {
            let ir_items = items
                .iter()
                .map(compile_expr)
                .collect::<Result<Vec<_>>>()?;
            Ok(IRNode::List(ir_items))
        }

        Expr::Map(entries) => {
            let ir_entries = entries
                .iter()
                .map(|(k, v)| Ok((k.clone(), compile_expr(v)?)))
                .collect::<Result<Vec<_>>>()?;
            Ok(IRNode::Map(ir_entries))
        }

        Expr::Variable(name) => Ok(IRNode::Variable(name.clone())),

        Expr::FunctionCall { name, args } => {
            let ir_args = args
                .iter()
                .map(compile_expr)
                .collect::<Result<Vec<_>>>()?;
            Ok(IRNode::FunctionCall {
                name: name.clone(),
                args: ir_args,
            })
        }

        Expr::TypeInstantiation { type_name, fields } => {
            let ir_fields = fields
                .iter()
                .map(|(k, v)| Ok((k.clone(), compile_expr(v)?)))
                .collect::<Result<Vec<_>>>()?;
            Ok(IRNode::TypeInstantiation {
                type_name: type_name.clone(),
                fields: ir_fields,
            })
        }

        Expr::FieldAccess { base, field } => Ok(IRNode::FieldAccess {
            base: Box::new(compile_expr(base)?),
            field: field.clone(),
        }),

        Expr::IndexAccess { base, index } => Ok(IRNode::IndexAccess {
            base: Box::new(compile_expr(base)?),
            index: Box::new(compile_expr(index)?),
        }),

        Expr::BinaryOp { left, op, right } => Ok(IRNode::BinaryOp {
            left: Box::new(compile_expr(left)?),
            op: op.clone(),
            right: Box::new(compile_expr(right)?),
        }),

        Expr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => Ok(IRNode::Conditional {
            condition: Box::new(compile_expr(condition)?),
            then_expr: Box::new(compile_expr(then_expr)?),
            else_expr: Box::new(compile_expr(else_expr)?),
        }),

        Expr::Sequential {
            left,
            right,
            binding,
        } => Ok(IRNode::Sequential {
            left: Box::new(compile_expr(left)?),
            right: Box::new(compile_expr(right)?),
            binding: binding.as_ref().map(compile_binding).transpose()?,
        }),

        Expr::Parallel { exprs, binding } => {
            let ir_exprs = exprs
                .iter()
                .map(compile_expr)
                .collect::<Result<Vec<_>>>()?;
            Ok(IRNode::Parallel {
                exprs: ir_exprs,
                binding: binding.as_ref().map(compile_binding).transpose()?,
            })
        }

        Expr::Match { scrutinee, cases } => {
            let ir_scrutinee = Box::new(compile_expr(scrutinee)?);
            let ir_cases = cases
                .iter()
                .map(compile_match_case)
                .collect::<Result<Vec<_>>>()?;
            Ok(IRNode::Match {
                scrutinee: ir_scrutinee,
                cases: ir_cases,
            })
        }
    }
}

/// Compile a binding pattern
fn compile_binding(binding: &Binding) -> Result<IRBinding> {
    match binding {
        Binding::Single(name) => Ok(IRBinding::Single(name.clone())),
        Binding::List(names) => Ok(IRBinding::List(names.clone())),
    }
}

/// Compile a match case
fn compile_match_case(case: &MatchCase) -> Result<IRMatchCase> {
    Ok(IRMatchCase {
        pattern: compile_pattern(&case.pattern)?,
        guard: case.guard.as_ref().map(compile_expr).transpose()?.map(Box::new),
        body: Box::new(compile_expr(&case.body)?),
    })
}

/// Compile a pattern
fn compile_pattern(pattern: &Pattern) -> Result<IRPattern> {
    match pattern {
        Pattern::Any => Ok(IRPattern::Any),

        Pattern::Literal(expr) => {
            let ir_node = compile_expr(expr)?;
            Ok(IRPattern::Literal(Box::new(ir_node)))
        }

        Pattern::Variable(name) => Ok(IRPattern::Variable(name.clone())),

        Pattern::Binding(name, nested) => Ok(IRPattern::Binding(
            name.clone(),
            Box::new(compile_pattern(nested)?),
        )),

        Pattern::Type { type_name, inner } => Ok(IRPattern::Type {
            type_name: type_name.clone(),
            inner: inner.as_ref().map(|p| compile_pattern(p)).transpose()?.map(Box::new),
        }),

        Pattern::List { patterns, rest } => {
            let ir_patterns = patterns
                .iter()
                .map(compile_pattern)
                .collect::<Result<Vec<_>>>()?;
            Ok(IRPattern::List {
                patterns: ir_patterns,
                rest: rest.clone(),
            })
        }

        Pattern::Map { fields, strict } => {
            let ir_fields = fields
                .iter()
                .map(|(name, pat)| Ok((name.clone(), compile_pattern(pat)?)))
                .collect::<Result<Vec<_>>>()?;
            Ok(IRPattern::Map {
                fields: ir_fields,
                strict: *strict,
            })
        }

        Pattern::Tuple(patterns) => {
            let ir_patterns = patterns
                .iter()
                .map(compile_pattern)
                .collect::<Result<Vec<_>>>()?;
            Ok(IRPattern::Tuple(ir_patterns))
        }
    }
}

/// Compile a template segment
fn compile_template_segment(segment: &TemplateSegment) -> Result<IRTemplateSegment> {
    match segment {
        TemplateSegment::Text(t) => Ok(IRTemplateSegment::Text(t.clone())),
        TemplateSegment::Interpolation(expr) => Ok(IRTemplateSegment::Interpolation(expr.clone())),
    }
}

/// Compile a pattern-based function to IRFunctionGroup
pub fn compile_pattern_function(func: &PatternFunctionDef) -> Result<IRFunctionGroup> {
    let clauses: Result<Vec<IRFunctionClause>> = func
        .clauses
        .iter()
        .map(compile_pattern_function_clause)
        .collect();

    Ok(IRFunctionGroup {
        name: func.name.clone(),
        clauses: clauses?,
        return_type: func.return_type.clone(),
    })
}

/// Compile a pattern function clause to IRFunctionClause
fn compile_pattern_function_clause(clause: &PatternFunctionClause) -> Result<IRFunctionClause> {
    let param_patterns: Result<Vec<IRPattern>> = clause
        .param_patterns
        .iter()
        .map(compile_pattern)
        .collect();

    Ok(IRFunctionClause {
        param_patterns: param_patterns?,
        guard: clause.guard.as_ref().map(compile_expr).transpose()?.map(Box::new),
        body: Box::new(compile_expr(&clause.body)?),
    })
}

/// Compile a function definition to IR
pub fn compile_function(func: &FunctionDef) -> Result<IRFunction> {
    let execution = match &func.execution {
        FunctionExecution::LLM {
            prompt,
            model,
            base_url,
            api_key_env,
            temperature,
        } => IRExecution::LLM {
            prompt: prompt.clone(),
            model: model.clone(),
            base_url: base_url.clone(),
            api_key_env: api_key_env.clone(),
            temperature: *temperature,
        },

        FunctionExecution::HTTP {
            method,
            url,
            params,
            headers,
            body,
        } => IRExecution::HTTP {
            method: method.clone(),
            url: url.clone(),
            params: params.clone(),
            headers: headers.clone(),
            body: body.clone(),
        },

        FunctionExecution::SQL { query } => IRExecution::SQL {
            query: query.clone(),
        },

        FunctionExecution::HTTPWithLLM {
            http_method,
            http_url,
            http_params,
            http_headers,
            llm_prompt,
            llm_model,
            llm_base_url,
            llm_api_key_env,
            llm_temperature,
        } => IRExecution::HTTPWithLLM {
            http_method: http_method.clone(),
            http_url: http_url.clone(),
            http_params: http_params.clone(),
            http_headers: http_headers.clone(),
            llm_prompt: llm_prompt.clone(),
            llm_model: llm_model.clone(),
            llm_base_url: llm_base_url.clone(),
            llm_api_key_env: llm_api_key_env.clone(),
            llm_temperature: *llm_temperature,
        },
    };

    // Convert properties
    let properties: HashMap<String, IRProperty> = func
        .properties
        .iter()
        .map(|(k, v)| (k.clone(), compile_property(v)))
        .collect();

    Ok(IRFunction {
        name: func.name.clone(),
        params: func.params.clone(),
        return_type: func.return_type.clone(),
        properties,
        execution,
    })
}

/// Compile a property value
fn compile_property(prop: &PropertyValue) -> IRProperty {
    match prop {
        PropertyValue::String(s) => IRProperty::String(s.clone()),
        PropertyValue::Template(segments) => {
            let ir_segments = segments
                .iter()
                .map(|seg| match seg {
                    TemplateSegment::Text(t) => IRTemplateSegment::Text(t.clone()),
                    TemplateSegment::Interpolation(expr) => {
                        IRTemplateSegment::Interpolation(expr.clone())
                    }
                })
                .collect();
            IRProperty::Template(ir_segments)
        }
        PropertyValue::Int(i) => IRProperty::Int(*i),
        PropertyValue::Float(f) => IRProperty::Float(*f),
        PropertyValue::Bool(b) => IRProperty::Bool(*b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_string_literal() {
        let source = r#""hello""#;
        let ir = compile_to_ir(source).unwrap();
        match ir.entry_expr {
            IRNode::String(s) => assert_eq!(s, "hello"),
            IRNode::TemplateString(segments) => {
                // Strings can be parsed as template strings with no interpolation
                assert_eq!(segments.len(), 1);
                match &segments[0] {
                    IRTemplateSegment::Text(t) => assert_eq!(t, "hello"),
                    _ => panic!("Expected text segment"),
                }
            }
            _ => panic!("Expected string or template string node, got: {:?}", ir.entry_expr),
        }
    }

    #[test]
    fn test_compile_int_literal() {
        let source = "42";
        let ir = compile_to_ir(source).unwrap();
        match ir.entry_expr {
            IRNode::Int(i) => assert_eq!(i, 42),
            _ => panic!("Expected int node"),
        }
    }

    #[test]
    fn test_compile_list() {
        let source = "[1, 2, 3]";
        let ir = compile_to_ir(source).unwrap();
        match ir.entry_expr {
            IRNode::List(items) => assert_eq!(items.len(), 3),
            _ => panic!("Expected list node"),
        }
    }

    #[test]
    fn test_compile_function_call() {
        let source = "Length(\"test\")";
        let ir = compile_to_ir(source).unwrap();
        match ir.entry_expr {
            IRNode::FunctionCall { name, args } => {
                assert_eq!(name, "Length");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected function call node"),
        }
    }

    #[test]
    fn test_compile_binary_op() {
        let source = "1 + 2";
        let ir = compile_to_ir(source).unwrap();
        match ir.entry_expr {
            IRNode::BinaryOp { left, op, right } => {
                assert_eq!(op, "+");
                assert!(matches!(*left, IRNode::Int(1)));
                assert!(matches!(*right, IRNode::Int(2)));
            }
            _ => panic!("Expected binary op node"),
        }
    }

    #[test]
    fn test_compile_sequential() {
        let source = "5 |> _ * 2";
        let ir = compile_to_ir(source).unwrap();
        match ir.entry_expr {
            IRNode::Sequential { left, right, .. } => {
                assert!(matches!(*left, IRNode::Int(5)));
                assert!(matches!(*right, IRNode::BinaryOp { .. }));
            }
            _ => panic!("Expected sequential node"),
        }
    }

    #[test]
    fn test_compile_conditional() {
        let source = "true ? 1 : 2";
        let ir = compile_to_ir(source).unwrap();
        match ir.entry_expr {
            IRNode::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                assert!(matches!(*condition, IRNode::Bool(true)));
                assert!(matches!(*then_expr, IRNode::Int(1)));
                assert!(matches!(*else_expr, IRNode::Int(2)));
            }
            _ => panic!("Expected conditional node"),
        }
    }
}
