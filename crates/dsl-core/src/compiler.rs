use anyhow::Result;
use dsl_ir::{
    IRBinding, IRExecution, IRFunction, IRFunctionClause, IRFunctionGroup, IRMatchCase, IRNode,
    IRPattern, IRProperty, IRTemplateSegment, LambdaIR, IR,
};
use std::collections::HashMap;

use crate::parser::{
    parse_expr, parse_program, Binding, Expr, FunctionDef, FunctionExecution, MatchCase, Pattern,
    PatternFunctionClause, PatternFunctionDef, Program, PropertyValue, TemplateSegment,
};
use crate::resolver::{resolve_program, Clause, FunctionBody, FunctionGroup, SymbolTable};

/// Compile DSL source code to IR
/// Handles both full programs (with types, enums, functions) and standalone expressions
pub fn compile_to_ir(source: &str) -> Result<IR> {
    // Try parsing as a program first (handles types, enums, and functions)
    if let Ok(program) = parse_program(source) {
        // Reject programs with imports
        if !program.imports.is_empty() {
            return Err(anyhow::anyhow!(
                "This program contains import statements, which require file-based compilation.\n\
                Please save your code to a file and use compile_file_to_ir() instead, \n\
                or use the :run command in the REPL with a file path."
            ));
        }

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

/// Compile a complete program to IR using the resolver (NEW workflow)
/// This is the recommended way to compile programs going forward
pub fn compile_program_with_resolver(source: &str) -> Result<IR> {
    let program = parse_program(source).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    compile_program_to_ir_with_resolver(&program)
}

/// Compile a parsed program to IR
pub fn compile_program_to_ir(program: &Program) -> Result<IR> {
    // Compile traditional functions
    let functions: Result<Vec<IRFunction>> =
        program.functions.iter().map(compile_function).collect();
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

/// Compile a parsed program to IR using the resolver (NEW workflow)
/// This uses the resolver to group functions and eliminates duplicate classification logic
pub fn compile_program_to_ir_with_resolver(program: &Program) -> Result<IR> {
    // Step 1: Use resolver to group functions
    let mut symbol_table = SymbolTable::new();
    let groups = resolve_program(program.clone(), &mut symbol_table)
        .map_err(|e| anyhow::anyhow!("Resolver error: {}", e))?;

    // Step 2: Compile each group
    let mut ir_functions = Vec::new();
    let mut ir_function_groups = Vec::new();

    for group in &groups {
        let (ir_func, ir_group) = compile_function_group(group)?;

        if let Some(f) = ir_func {
            ir_functions.push(f);
        }
        if let Some(g) = ir_group {
            ir_function_groups.push(g);
        }
    }

    // Step 3: Compile entry expression if present
    let entry_expr = if let Some(expr) = &program.entry_expr {
        compile_expr(expr)?
    } else {
        IRNode::Int(0) // Default: return 0 if no entry expression
    };

    Ok(IR {
        version: "0.1.0".to_string(),
        types: program.types.clone(),
        enums: program.enums.clone(),
        functions: ir_functions,
        function_groups: ir_function_groups,
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
            let ir_items = items.iter().map(compile_expr).collect::<Result<Vec<_>>>()?;
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
            let ir_args = args.iter().map(compile_expr).collect::<Result<Vec<_>>>()?;
            Ok(IRNode::FunctionCall {
                name: name.clone(),
                args: ir_args,
                effect_kind: None, // Will be set during lowering for intrinsics
                source_span: None, // TODO: Extract from parser
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
            let ir_exprs = exprs.iter().map(compile_expr).collect::<Result<Vec<_>>>()?;
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

        Expr::Block { statements, result } => {
            let ir_statements = statements
                .iter()
                .map(compile_expr)
                .collect::<Result<Vec<_>>>()?;
            Ok(IRNode::Block {
                statements: ir_statements,
                result: Box::new(compile_expr(result)?),
            })
        }

        Expr::Lambda { params, body } => Ok(IRNode::Lambda(LambdaIR {
            params: params.clone(),
            body: Box::new(compile_expr(body)?),
        })),
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
        guard: case
            .guard
            .as_ref()
            .map(compile_expr)
            .transpose()?
            .map(Box::new),
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
            inner: inner
                .as_ref()
                .map(|p| compile_pattern(p))
                .transpose()?
                .map(Box::new),
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
        TemplateSegment::Interpolation(expr_str) => {
            // Parse and compile the interpolation expression
            let expr = parse_expr(expr_str)
                .map_err(|e| anyhow::anyhow!("Failed to parse template interpolation: {}", e))?;
            let ir_node = compile_expr(&expr)?;
            Ok(IRTemplateSegment::Interpolation(Box::new(ir_node)))
        }
    }
}

/// Compile a prompt template string into an IRNode
/// Parses the string as a template and compiles all interpolations
fn compile_prompt_template(prompt: &str) -> Result<IRNode> {
    // Simple template parsing: split on ${...}
    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut chars = prompt.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            // Found interpolation start
            chars.next(); // consume '{'

            // Save any accumulated text
            if !current_text.is_empty() {
                segments.push(TemplateSegment::Text(current_text.clone()));
                current_text.clear();
            }

            // Extract the expression until '}'
            let mut expr = String::new();
            let mut depth = 1;
            while let Some(ch) = chars.next() {
                if ch == '{' {
                    depth += 1;
                    expr.push(ch);
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    expr.push(ch);
                } else {
                    expr.push(ch);
                }
            }

            segments.push(TemplateSegment::Interpolation(expr));
        } else {
            current_text.push(ch);
        }
    }

    // Add any remaining text
    if !current_text.is_empty() {
        segments.push(TemplateSegment::Text(current_text));
    }

    // Compile each segment
    let ir_segments: Result<Vec<IRTemplateSegment>> =
        segments.iter().map(compile_template_segment).collect();

    Ok(IRNode::TemplateString(ir_segments?))
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
    let param_patterns: Result<Vec<IRPattern>> =
        clause.param_patterns.iter().map(compile_pattern).collect();

    Ok(IRFunctionClause {
        param_patterns: param_patterns?,
        guard: clause
            .guard
            .as_ref()
            .map(compile_expr)
            .transpose()?
            .map(Box::new),
        body: Box::new(compile_expr(&clause.body)?),
    })
}

/// Compile a function definition to IR
pub fn compile_function(func: &FunctionDef) -> Result<IRFunction> {
    let execution = match &func.execution {
        FunctionExecution::Expression { body } => IRExecution::Expression {
            body: Box::new(compile_expr(body)?),
        },

        FunctionExecution::LLM {
            prompt,
            model,
            base_url,
            api_key_env,
            temperature,
        } => IRExecution::LLM {
            prompt: Box::new(compile_prompt_template(prompt)?),
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
    };

    // Convert properties
    let properties: Result<HashMap<String, IRProperty>> = func
        .properties
        .iter()
        .map(|(k, v)| Ok((k.clone(), compile_property(v)?)))
        .collect();
    let properties = properties?;

    Ok(IRFunction {
        name: func.name.clone(),
        params: func.params.clone(),
        return_type: func.return_type.clone(),
        properties,
        execution,
    })
}

/// Compile a property value
fn compile_property(prop: &PropertyValue) -> Result<IRProperty> {
    match prop {
        PropertyValue::String(s) => Ok(IRProperty::String(s.clone())),
        PropertyValue::Template(segments) => {
            let ir_segments: Result<Vec<IRTemplateSegment>> = segments
                .iter()
                .map(|seg| match seg {
                    TemplateSegment::Text(t) => Ok(IRTemplateSegment::Text(t.clone())),
                    TemplateSegment::Interpolation(expr_str) => {
                        let expr = parse_expr(expr_str).map_err(|e| {
                            anyhow::anyhow!("Failed to parse template interpolation: {}", e)
                        })?;
                        let ir_node = compile_expr(&expr)?;
                        Ok(IRTemplateSegment::Interpolation(Box::new(ir_node)))
                    }
                })
                .collect();
            Ok(IRProperty::Template(ir_segments?))
        }
        PropertyValue::Int(i) => Ok(IRProperty::Int(*i)),
        PropertyValue::Float(f) => Ok(IRProperty::Float(*f)),
        PropertyValue::Bool(b) => Ok(IRProperty::Bool(*b)),
    }
}

// ==================== NEW RESOLVER-BASED COMPILATION ====================

/// Compile a FunctionGroup (from resolver) to IR
/// Handles both trivial functions (converted to IRFunction) and pattern functions (IRFunctionGroup)
pub fn compile_function_group(
    group: &FunctionGroup,
) -> Result<(Option<IRFunction>, Option<IRFunctionGroup>)> {
    // Check if this is a trivial function (single clause with simple patterns)
    if group.is_trivial() {
        // Compile as a simple IRFunction for optimization
        let clause = &group.clauses[0];
        let ir_function = compile_trivial_clause(&group.name, clause)?;
        Ok((Some(ir_function), None))
    } else {
        // Compile as IRFunctionGroup with pattern matching
        let ir_group = compile_function_group_to_ir(group)?;
        Ok((None, Some(ir_group)))
    }
}

/// Compile a trivial clause to IRFunction (optimization for simple functions)
fn compile_trivial_clause(name: &str, clause: &Clause) -> Result<IRFunction> {
    // Extract parameter names from simple Variable patterns
    let params: Vec<String> = clause
        .patterns
        .iter()
        .filter_map(|p| match p {
            Pattern::Variable(name) => Some(name.clone()),
            _ => None, // Should never happen for trivial clauses
        })
        .collect();

    // Compile the body
    let execution = compile_function_body(&clause.body)?;

    Ok(IRFunction {
        name: name.to_string(),
        params,
        return_type: clause.return_type.clone(),
        properties: HashMap::new(), // Trivial clauses don't have properties
        execution,
    })
}

/// Compile a FunctionBody to IRExecution
fn compile_function_body(body: &FunctionBody) -> Result<IRExecution> {
    match body {
        FunctionBody::Expr(expr) => Ok(IRExecution::Expression {
            body: Box::new(compile_expr(expr)?),
        }),
        FunctionBody::LLM {
            prompt,
            model,
            base_url,
            api_key_env,
            temperature,
        } => Ok(IRExecution::LLM {
            prompt: Box::new(compile_prompt_template(prompt)?),
            model: model.clone(),
            base_url: base_url.clone(),
            api_key_env: api_key_env.clone(),
            temperature: *temperature,
        }),
        FunctionBody::HTTP {
            method,
            url,
            params,
            headers,
            body,
        } => Ok(IRExecution::HTTP {
            method: method.clone(),
            url: url.clone(),
            params: params.clone(),
            headers: headers.clone(),
            body: body.clone(),
        }),
        FunctionBody::SQL { query } => Ok(IRExecution::SQL {
            query: query.clone(),
        }),
    }
}

/// Compile a FunctionGroup to IRFunctionGroup
fn compile_function_group_to_ir(group: &FunctionGroup) -> Result<IRFunctionGroup> {
    let clauses: Result<Vec<IRFunctionClause>> = group.clauses.iter().map(compile_clause).collect();

    Ok(IRFunctionGroup {
        name: group.name.clone(),
        clauses: clauses?,
        return_type: group.clauses.first().and_then(|c| c.return_type.clone()),
    })
}

/// Compile a Clause to IRFunctionClause
fn compile_clause(clause: &Clause) -> Result<IRFunctionClause> {
    let param_patterns: Result<Vec<IRPattern>> =
        clause.patterns.iter().map(compile_pattern).collect();

    // Extract the body expression from FunctionBody
    let body_expr = match &clause.body {
        FunctionBody::Expr(expr) => expr,
        _ => {
            return Err(anyhow::anyhow!(
                "Pattern function clauses must have expression bodies"
            ))
        }
    };

    Ok(IRFunctionClause {
        param_patterns: param_patterns?,
        guard: clause
            .guard
            .as_ref()
            .map(compile_expr)
            .transpose()?
            .map(Box::new),
        body: Box::new(compile_expr(body_expr)?),
    })
}

/// Compile a DSL file with imports to IR
///
/// This function handles multi-file compilation by:
/// 1. Loading the main file and all its dependencies via ModuleLoader
/// 2. Merging all types, enums, and functions from all modules
/// 3. Compiling to IR
///
/// # Arguments
/// * `file_path` - Path to the main .dsl file to compile
///
/// # Returns
/// Compiled IR with all modules merged
///
/// # Errors
/// Returns error if:
/// - File cannot be read
/// - Circular dependencies are detected
/// - Parse errors occur
/// - Compilation fails
pub fn compile_file_to_ir(file_path: &std::path::Path) -> Result<IR> {
    use crate::module_loader::ModuleLoader;

    // Create module loader
    let mut loader = ModuleLoader::new();

    // Get the file path as absolute
    let absolute_path = file_path.canonicalize().map_err(|e| {
        anyhow::anyhow!("Failed to resolve file path {}: {}", file_path.display(), e)
    })?;

    // Load the main module (this will recursively load all imports)
    let main_module = loader.load_module(
        &format!(
            "./{}",
            absolute_path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?
                .to_string_lossy()
        ),
        &absolute_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?,
    )?;

    // Get all loaded modules
    let all_modules = loader.get_all_modules();

    // Merge all types and enums from all modules
    let mut all_types = Vec::new();
    let mut all_enums = Vec::new();
    let mut all_functions = Vec::new();
    let mut all_pattern_functions = Vec::new();

    for module in &all_modules {
        all_types.extend(module.program.types.clone());
        all_enums.extend(module.program.enums.clone());
        all_functions.extend(module.program.functions.clone());
        all_pattern_functions.extend(module.program.pattern_functions.clone());
    }

    // Use the main module's entry expression
    let entry_expr = main_module.program.entry_expr.clone();

    // Create a merged program
    let merged_program = Program {
        imports: vec![], // Imports already resolved
        types: all_types,
        enums: all_enums,
        functions: all_functions,
        pattern_functions: all_pattern_functions,
        entry_expr,
    };

    // Compile the merged program
    compile_program_to_ir(&merged_program)
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
            _ => panic!(
                "Expected string or template string node, got: {:?}",
                ir.entry_expr
            ),
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
            IRNode::FunctionCall { name, args, .. } => {
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
