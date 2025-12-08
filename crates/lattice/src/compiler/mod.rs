//! Compiler module
//!
//! Compiles AST to bytecode instructions.

use crate::error::Result;
use crate::syntax::ast::{
    Accessor, AssignTarget, BinaryOp, Block, ElseClause, EnumDef, Expr, ExprKind, FStringPart,
    FunctionBody, FunctionDef, IfExprElse, Item, LambdaBody, LlmConfigDecl, Literal, MapKey,
    MatchArm, MatchArmBody, Pattern, PatternKind, PrimitiveType, Program, Stmt, StmtKind,
    TypeAnnotation, TypeDef, TypeExpr, UnaryOp,
};
use crate::syntax::desugar::{contains_dollar_field, wrap_dollar_expr_in_lambda};
use crate::types::{Class, Enum, Field, FieldType, TypeChecker, Value};
use crate::vm::bytecode::{Chunk, CompiledFunction, LlmFunction, OpCode};

/// Native functions built into the VM
const NATIVE_FUNCTIONS: &[&str] = &[
    "len", "type", "str", "int", "float", "bool", "push", "pop", "keys", "values", "print", "sqrt",
    // String functions
    "contains", "regex_match", "word_count",
    // Path functions
    "path", "path_join", "path_parent", "path_file_name", "path_extension", "path_exists",
    "path_is_file", "path_is_dir", "path_to_str",
];

/// Check if a function name is a native function
fn is_native(name: &str) -> bool {
    NATIVE_FUNCTIONS.contains(&name)
}

/// A local variable in the current scope
#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: usize,
}

/// Result of compilation
#[derive(Debug, Clone, Default)]
pub struct CompileResult {
    /// Main bytecode chunk for top-level code
    pub chunk: Chunk,
    /// Compiled classes (type definitions)
    pub classes: Vec<Class>,
    /// Compiled enums
    pub enums: Vec<Enum>,
    /// Compiled user functions
    pub functions: Vec<CompiledFunction>,
    /// Compiled LLM functions
    pub llm_functions: Vec<LlmFunction>,
}

/// Compiler state
pub struct Compiler {
    /// Bytecode being built
    chunk: Chunk,
    /// Stack of local variables
    locals: Vec<Local>,
    /// Current scope depth (0 = global)
    scope_depth: usize,
    /// Collected classes from type definitions
    classes: Vec<Class>,
    /// Collected enums from enum definitions
    enums: Vec<Enum>,
    /// Collected compiled functions
    functions: Vec<CompiledFunction>,
    /// Collected LLM functions
    llm_functions: Vec<LlmFunction>,
    /// LLM config declarations (reusable config blocks)
    llm_configs: std::collections::HashMap<String, LlmConfigDecl>,
    /// Counter for generating unique names (for hidden variables)
    unique_counter: usize,
    /// Known function names (for forward/recursive calls)
    known_functions: Vec<String>,
    /// Known LLM function names (from previous cells in notebook)
    known_llm_functions: Vec<String>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    /// Create a new compiler
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            classes: Vec::new(),
            enums: Vec::new(),
            functions: Vec::new(),
            llm_functions: Vec::new(),
            llm_configs: std::collections::HashMap::new(),
            unique_counter: 0,
            known_functions: Vec::new(),
            known_llm_functions: Vec::new(),
        }
    }

    /// Compile a program to bytecode
    pub fn compile(program: &Program) -> Result<CompileResult> {
        let mut compiler = Compiler::new();
        compiler.compile_program(program)?;
        Ok(CompileResult {
            chunk: compiler.chunk,
            classes: compiler.classes,
            enums: compiler.enums,
            functions: compiler.functions,
            llm_functions: compiler.llm_functions,
        })
    }

    /// Compile a program with pre-existing known function names
    ///
    /// This is useful for REPL/notebook environments where functions
    /// from previous cells should be callable in subsequent cells.
    pub fn compile_with_known_functions(
        program: &Program,
        known_functions: Vec<String>,
    ) -> Result<CompileResult> {
        Self::compile_with_known_functions_full(program, known_functions, Vec::new())
    }

    /// Compile a program with pre-existing known function names (both user and LLM)
    ///
    /// This is useful for REPL/notebook environments where functions
    /// from previous cells should be callable in subsequent cells.
    pub fn compile_with_known_functions_full(
        program: &Program,
        known_functions: Vec<String>,
        known_llm_functions: Vec<String>,
    ) -> Result<CompileResult> {
        let mut compiler = Compiler::new();
        compiler.known_functions = known_functions;
        compiler.known_llm_functions = known_llm_functions;
        compiler.compile_program(program)?;
        Ok(CompileResult {
            chunk: compiler.chunk,
            classes: compiler.classes,
            enums: compiler.enums,
            functions: compiler.functions,
            llm_functions: compiler.llm_functions,
        })
    }

    /// Compile a program with type checking
    pub fn compile_with_typecheck(program: &Program) -> Result<CompileResult> {
        // Run type checker first
        let mut checker = TypeChecker::new();
        checker.check_program(program)?;

        // Then compile
        Self::compile(program)
    }

    /// Compile an entire program
    fn compile_program(&mut self, program: &Program) -> Result<()> {
        // First pass: collect all LLM config declarations
        for item in &program.items {
            if let Item::LlmConfigDecl(config_decl) = item {
                self.llm_configs
                    .insert(config_decl.name.node.clone(), config_decl.clone());
            }
        }

        // Second pass: compile all items
        let len = program.items.len();
        for (i, item) in program.items.iter().enumerate() {
            let is_last = i == len - 1;
            self.compile_item_with_context(item, is_last)?;
        }
        Ok(())
    }

    /// Compile a top-level item with context about whether it's the last item
    fn compile_item_with_context(&mut self, item: &Item, is_last: bool) -> Result<()> {
        match item {
            Item::TypeDef(type_def) => self.compile_type_def(type_def),
            Item::EnumDef(enum_def) => self.compile_enum_def(enum_def),
            Item::LlmConfigDecl(_) => {
                // LLM config declarations are already collected in compile_program
                // They don't generate any bytecode themselves
                Ok(())
            }
            Item::FunctionDef(func_def) => self.compile_function_def(func_def),
            Item::Statement(stmt) => {
                // For the last item, if it's an expression statement, don't pop
                if is_last {
                    if let StmtKind::Expr { expr } = &stmt.kind {
                        self.compile_expr(expr)?;
                        return Ok(());
                    }
                }
                self.compile_stmt(stmt)
            }
        }
    }


    // ========================================================================
    // Type Definition Compilation (DSL-rewrite-n46)
    // ========================================================================

    /// Compile a type definition to IR Class
    fn compile_type_def(&mut self, type_def: &TypeDef) -> Result<()> {
        let class = Class {
            name: type_def.name.node.clone(),
            description: None,
            fields: type_def
                .fields
                .iter()
                .map(|f| Field {
                    name: f.name.node.clone(),
                    field_type: convert_type_annotation(&f.ty),
                    optional: f.ty.optional,
                    description: f.description.clone(),
                })
                .collect(),
        };
        self.classes.push(class);
        Ok(())
    }

    /// Compile an enum definition to IR Enum
    fn compile_enum_def(&mut self, enum_def: &EnumDef) -> Result<()> {
        let enum_ir = Enum {
            name: enum_def.name.node.clone(),
            description: None,
            values: enum_def.variants.iter().map(|v| v.node.clone()).collect(),
        };
        self.enums.push(enum_ir);
        Ok(())
    }

    // ========================================================================
    // Function Compilation (DSL-rewrite-rej)
    // ========================================================================

    /// Compile a function definition
    fn compile_function_def(&mut self, func_def: &FunctionDef) -> Result<()> {
        match &func_def.body {
            FunctionBody::Block(block) => {
                self.compile_regular_function(func_def, block)?;
            }
            FunctionBody::LlmConfig(config) => {
                self.compile_llm_function(func_def, config)?;
            }
        }
        Ok(())
    }

    /// Compile a regular function with a block body
    fn compile_regular_function(
        &mut self,
        func_def: &FunctionDef,
        block: &Block,
    ) -> Result<()> {
        // Register the function name before compiling body (allows recursion)
        let func_name = func_def.name.node.clone();
        self.known_functions.push(func_name.clone());

        // Create a new compiler for the function body
        let mut func_compiler = Compiler::new();
        func_compiler.scope_depth = 1; // Function scope
        // Copy known functions to allow recursive/mutual calls
        func_compiler.known_functions = self.known_functions.clone();
        func_compiler.known_llm_functions = self.known_llm_functions.clone();
        // Copy llm_functions so LLM functions defined in same file can be detected for map_row
        func_compiler.llm_functions = self.llm_functions.clone();
        // Copy unique_counter to avoid lambda name collisions between functions
        func_compiler.unique_counter = self.unique_counter;

        // Add parameters as locals (in order)
        for param in &func_def.params {
            func_compiler.add_local(param.name.node.clone());
        }

        // Compile function body
        func_compiler.compile_block(block)?;

        // Ensure there's a return at the end
        // The block compilation leaves a value on stack, so emit Return
        func_compiler.emit(OpCode::Return, func_def.span.line);

        let compiled = CompiledFunction {
            name: func_name,
            arity: func_def.params.len(),
            local_count: func_compiler.locals.len(),
            chunk: func_compiler.chunk,
        };

        self.functions.push(compiled);

        // Collect any nested lambdas compiled inside the function body
        self.functions.extend(func_compiler.functions);

        // Update unique_counter to avoid collisions with future compilations
        self.unique_counter = func_compiler.unique_counter;

        Ok(())
    }

    /// Compile an LLM function
    fn compile_llm_function(
        &mut self,
        func_def: &FunctionDef,
        config: &crate::syntax::ast::LlmConfig,
    ) -> Result<()> {
        // Resolve base config from use: reference if present
        let (base_url, model, api_key_env, mut temperature, mut max_tokens, mut provider) =
            if let Some(ref config_name) = config.use_config {
                if let Some(base_config) = self.llm_configs.get(config_name) {
                    (
                        base_config.base_url.clone(),
                        base_config.model.clone(),
                        base_config.api_key_env.clone(),
                        base_config.temperature,
                        base_config.max_tokens,
                        base_config.provider.as_ref().map(convert_provider_config),
                    )
                } else {
                    return Err(crate::error::LatticeError::Compile(format!(
                        "Unknown llm_config '{}' referenced in function '{}'",
                        config_name, func_def.name.node
                    )));
                }
            } else {
                (None, None, None, None, None, None)
            };

        // Override with inline config values (inline values take precedence)
        let final_base_url = config.base_url.clone().or(base_url).unwrap_or_default();
        let final_model = config.model.clone().or(model).unwrap_or_default();
        let final_api_key_env = config.api_key_env.clone().or(api_key_env).unwrap_or_default();

        // For temperature and max_tokens, inline values override base config
        if config.temperature.is_some() {
            temperature = config.temperature;
        }
        if config.max_tokens.is_some() {
            max_tokens = config.max_tokens;
        }
        // For provider, inline config overrides base config
        if config.provider.is_some() {
            provider = config.provider.as_ref().map(convert_provider_config);
        }

        // Determine return type
        let return_type = func_def
            .return_type
            .as_ref()
            .map(convert_type_annotation)
            .unwrap_or(FieldType::String);

        // Check if prompt contains complex expressions that need runtime evaluation
        let needs_runtime_eval = prompt_has_complex_expressions(&config.prompt);

        // Convert the prompt expression to a template string (for simple cases or fallback)
        let prompt_template = prompt_expr_to_template(&config.prompt);

        let mut llm_func = LlmFunction::new(
            func_def.name.node.clone(),
            final_base_url,
            final_model,
            final_api_key_env,
            prompt_template,
            return_type,
            func_def
                .params
                .iter()
                .map(|p| (p.name.node.clone(), convert_type_annotation(&p.ty)))
                .collect(),
        );

        // If the prompt has complex expressions, compile it to bytecode
        if needs_runtime_eval {
            let prompt_chunk = self.compile_prompt_to_bytecode(func_def, &config.prompt)?;
            llm_func = llm_func.with_prompt_chunk(prompt_chunk);
        }

        // Apply optional settings
        let llm_func = if let Some(temp) = temperature {
            llm_func.with_temperature(temp)
        } else {
            llm_func
        };

        let llm_func = if let Some(max_tok) = max_tokens {
            llm_func.with_max_tokens(max_tok)
        } else {
            llm_func
        };

        let llm_func = if let Some(prov) = provider {
            llm_func.with_provider(prov)
        } else {
            llm_func
        };

        self.llm_functions.push(llm_func);
        Ok(())
    }

    /// Compile a prompt expression to bytecode for runtime evaluation.
    /// The resulting chunk expects parameters to be on the stack and produces a String.
    fn compile_prompt_to_bytecode(
        &mut self,
        func_def: &FunctionDef,
        prompt: &Expr,
    ) -> Result<Chunk> {
        // Create a new compiler for the prompt expression
        let mut prompt_compiler = Compiler::new();
        prompt_compiler.scope_depth = 1; // Function scope

        // Add function parameters as locals (in order)
        for param in &func_def.params {
            prompt_compiler.add_local(param.name.node.clone());
        }

        // Compile the prompt expression - this should leave a String on the stack
        prompt_compiler.compile_expr(prompt)?;

        // The result should be a String; we don't emit Return here because
        // the VM will just read the top of stack after executing the chunk

        Ok(prompt_compiler.chunk)
    }

    // ========================================================================
    // Statement Compilation (DSL-rewrite-mr2)
    // ========================================================================

    /// Compile a statement
    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        let line = stmt.span.line;
        match &stmt.kind {
            StmtKind::Let { name, ty: _, value } => {
                self.compile_expr(value)?;
                if self.scope_depth == 0 {
                    // Global variable
                    let name_idx = self.intern_string(&name.node);
                    self.emit(OpCode::SetGlobal(name_idx), line);
                    self.emit(OpCode::Pop, line);
                } else {
                    // Local variable - value is already on stack
                    self.add_local(name.node.clone());
                }
            }
            StmtKind::Assign { target, value } => {
                self.compile_assignment(target, value, line)?;
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_if(condition, then_branch, else_branch, line)?;
            }
            StmtKind::While { condition, body } => {
                self.compile_while(condition, body, line)?;
            }
            StmtKind::For { var, iterable, body } => {
                self.compile_for(&var.node, iterable, body, line)?;
            }
            StmtKind::Return { value } => {
                if let Some(v) = value {
                    self.compile_expr(v)?;
                } else {
                    self.emit_const(Value::Null, line);
                }
                self.emit(OpCode::Return, line);
            }
            StmtKind::Expr { expr } => {
                self.compile_expr(expr)?;
                self.emit(OpCode::Pop, line);
            }
        }
        Ok(())
    }

    /// Compile an assignment statement
    fn compile_assignment(&mut self, target: &AssignTarget, value: &Expr, line: usize) -> Result<()> {
        if target.accessors.is_empty() {
            // Simple variable assignment
            self.compile_expr(value)?;
            if let Some(slot) = self.resolve_local(&target.base.node) {
                self.emit(OpCode::SetLocal(slot), line);
            } else {
                let name_idx = self.intern_string(&target.base.node);
                self.emit(OpCode::SetGlobal(name_idx), line);
            }
            self.emit(OpCode::Pop, line);
        } else {
            // Complex assignment with accessors: a.b[c].d = value
            // Strategy: Get base, apply all but last accessor, then set
            self.compile_complex_assignment(target, value, line)?;
        }
        Ok(())
    }

    /// Compile a complex assignment with field/index accessors
    fn compile_complex_assignment(
        &mut self,
        target: &AssignTarget,
        value: &Expr,
        line: usize,
    ) -> Result<()> {
        let accessors = &target.accessors;
        let last_idx = accessors.len() - 1;

        // Load the base variable
        if let Some(slot) = self.resolve_local(&target.base.node) {
            self.emit(OpCode::GetLocal(slot), line);
        } else {
            let name_idx = self.intern_string(&target.base.node);
            self.emit(OpCode::GetGlobal(name_idx), line);
        }

        // Apply all accessors except the last one
        for accessor in &accessors[..last_idx] {
            match accessor {
                Accessor::Field(name) => {
                    let field_idx = self.intern_string(name);
                    self.emit(OpCode::GetField(field_idx), line);
                }
                Accessor::Index(index_expr) => {
                    self.compile_expr(index_expr)?;
                    self.emit(OpCode::Index, line);
                }
            }
        }

        // Now handle the last accessor with set semantics
        match &accessors[last_idx] {
            Accessor::Field(name) => {
                self.compile_expr(value)?;
                let field_idx = self.intern_string(name);
                self.emit(OpCode::SetField(field_idx), line);
            }
            Accessor::Index(index_expr) => {
                self.compile_expr(index_expr)?;
                self.compile_expr(value)?;
                self.emit(OpCode::IndexSet, line);
            }
        }

        // Store result back to base variable
        if let Some(slot) = self.resolve_local(&target.base.node) {
            self.emit(OpCode::SetLocal(slot), line);
        } else {
            let name_idx = self.intern_string(&target.base.node);
            self.emit(OpCode::SetGlobal(name_idx), line);
        }
        self.emit(OpCode::Pop, line);

        Ok(())
    }

    /// Compile an if statement
    fn compile_if(
        &mut self,
        condition: &Expr,
        then_branch: &Block,
        else_branch: &Option<ElseClause>,
        line: usize,
    ) -> Result<()> {
        // Compile condition
        self.compile_expr(condition)?;

        // Jump to else (or end) if false
        let jump_to_else = self.emit_jump(OpCode::JumpIfFalse(0), line);

        // Compile then branch
        self.compile_block(then_branch)?;
        self.emit(OpCode::Pop, line); // Discard block result for statement

        if let Some(else_clause) = else_branch {
            // Jump over else
            let jump_over_else = self.emit_jump(OpCode::Jump(0), line);

            // Patch jump to else
            self.patch_jump(jump_to_else);

            // Compile else branch
            match else_clause {
                ElseClause::Else(block) => {
                    self.compile_block(block)?;
                    self.emit(OpCode::Pop, line);
                }
                ElseClause::ElseIf(if_stmt) => {
                    self.compile_stmt(if_stmt)?;
                }
            }

            // Patch jump over else
            self.patch_jump(jump_over_else);
        } else {
            // Patch jump to end
            self.patch_jump(jump_to_else);
        }

        Ok(())
    }

    /// Compile a while loop
    fn compile_while(&mut self, condition: &Expr, body: &Block, line: usize) -> Result<()> {
        let loop_start = self.chunk.len();

        // Compile condition
        self.compile_expr(condition)?;

        // Jump to end if false
        let exit_jump = self.emit_jump(OpCode::JumpIfFalse(0), line);

        // Compile body
        self.compile_block(body)?;
        self.emit(OpCode::Pop, line); // Discard block result

        // Jump back to start
        self.emit(OpCode::Jump(loop_start), line);

        // Patch exit jump
        self.patch_jump(exit_jump);

        Ok(())
    }

    /// Compile a for loop
    fn compile_for(&mut self, var: &str, iterable: &Expr, body: &Block, line: usize) -> Result<()> {
        self.begin_scope();

        // Create hidden locals for iterator and index
        let iter_name = self.generate_unique_name("__iter");
        let idx_name = self.generate_unique_name("__idx");

        // Compile iterable and store
        self.compile_expr(iterable)?;
        self.add_local(iter_name.clone());
        let iter_slot = self.locals.len() - 1;

        // Initialize index to 0
        self.emit_const(Value::Int(0), line);
        self.add_local(idx_name.clone());
        let idx_slot = self.locals.len() - 1;

        // Initialize loop variable with null (placeholder)
        self.emit_const(Value::Null, line);
        self.add_local(var.to_string());
        let var_slot = self.locals.len() - 1;

        let loop_start = self.chunk.len();

        // Check: idx < len(iter)
        self.emit(OpCode::GetLocal(idx_slot), line);
        self.emit(OpCode::GetLocal(iter_slot), line);
        let len_idx = self.intern_string("len");
        self.emit(OpCode::CallNative(len_idx, 1), line);
        self.emit(OpCode::Lt, line);

        // Exit if false
        let exit_jump = self.emit_jump(OpCode::JumpIfFalse(0), line);

        // Update loop variable: var = iter[idx]
        self.emit(OpCode::GetLocal(iter_slot), line);
        self.emit(OpCode::GetLocal(idx_slot), line);
        self.emit(OpCode::Index, line);
        self.emit(OpCode::SetLocal(var_slot), line);
        self.emit(OpCode::Pop, line);

        // Compile body
        self.compile_block(body)?;
        self.emit(OpCode::Pop, line); // Discard block result

        // Increment index
        self.emit(OpCode::GetLocal(idx_slot), line);
        self.emit_const(Value::Int(1), line);
        self.emit(OpCode::Add, line);
        self.emit(OpCode::SetLocal(idx_slot), line);
        self.emit(OpCode::Pop, line);

        // Jump back
        self.emit(OpCode::Jump(loop_start), line);

        // Patch exit
        self.patch_jump(exit_jump);

        self.end_scope(line);
        Ok(())
    }

    // ========================================================================
    // Expression Compilation (DSL-rewrite-n53)
    // ========================================================================

    /// Compile an expression
    fn compile_expr(&mut self, expr: &Expr) -> Result<()> {
        let line = expr.span.line;
        match &expr.kind {
            ExprKind::Literal(lit) => self.compile_literal(lit, line),
            ExprKind::Var(name) => self.compile_var(name, line),
            ExprKind::EnumVariant { enum_name, variant } => {
                self.compile_enum_variant(enum_name, variant, line)
            }
            ExprKind::Binary { left, op, right } => self.compile_binary(left, *op, right, line),
            ExprKind::Unary { op, operand } => self.compile_unary(*op, operand, line),
            ExprKind::Field { object, field } => self.compile_field(object, field, line),
            ExprKind::Index { object, index } => self.compile_index(object, index, line),
            ExprKind::Call { callee, args } => self.compile_call(callee, args, line),
            ExprKind::List(items) => self.compile_list(items, line),
            ExprKind::Map(pairs) => self.compile_map(pairs, line),
            ExprKind::Struct { name, fields } => self.compile_struct(name, fields, line),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.compile_if_expr(condition, then_branch, else_branch, line),
            ExprKind::Match { scrutinee, arms } => self.compile_match(scrutinee, arms, line),
            ExprKind::Lambda { params, body } => self.compile_lambda(params, body, line),
            ExprKind::Parallel(exprs) => self.compile_parallel(exprs, line),
            ExprKind::ParallelMap { collection, mapper } => {
                self.compile_parallel_map(collection, mapper, line)
            }
            ExprKind::MapColumn { table, input_col, output_col, mapper } => {
                self.compile_map_column(table, input_col, output_col, mapper, line)
            }
            ExprKind::MapRow { table, output_col, mapper } => {
                self.compile_map_row(table, output_col, mapper, line)
            }
            ExprKind::Explode { table, column, prefix } => {
                self.compile_explode(table, column, prefix, line)
            }
            ExprKind::Sql { ty, query } => self.compile_sql(ty, query, line),
            ExprKind::Grouped(inner) => self.compile_expr(inner),
            ExprKind::Block(block) => self.compile_block(block),
            ExprKind::FString(parts) => self.compile_fstring(parts, line),
            ExprKind::DollarField(field_expr) => {
                // DollarField should be desugared before reaching the compiler.
                // If we get here, it means $field was used in a context that doesn't
                // support implicit lambdas. We compile it as: |__row__| __row__[field]
                self.compile_dollar_field_as_lambda(field_expr, line)
            }
        }
    }

    /// Compile a DollarField as an implicit lambda: $field -> |__row__| __row__["field"]
    fn compile_dollar_field_as_lambda(&mut self, field_expr: &Expr, line: usize) -> Result<()> {
        // Create the lambda body: __row__[field_expr]
        let row_var = Expr {
            kind: ExprKind::Var("__row__".to_string()),
            span: field_expr.span,
        };

        let index_expr = Expr {
            kind: ExprKind::Index {
                object: Box::new(row_var),
                index: Box::new(field_expr.clone()),
            },
            span: field_expr.span,
        };

        // Compile as lambda |__row__| __row__[field]
        self.compile_lambda(&["__row__".to_string()], &LambdaBody::Expr(index_expr), line)
    }

    /// Compile a literal value
    fn compile_literal(&mut self, lit: &Literal, line: usize) -> Result<()> {
        let value = match lit {
            Literal::Int(n) => Value::Int(*n),
            Literal::Float(f) => Value::Float(*f),
            Literal::String(s) => Value::string(s.as_str()),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Null => Value::Null,
        };
        self.emit_const(value, line);
        Ok(())
    }

    /// Compile a variable reference
    fn compile_var(&mut self, name: &str, line: usize) -> Result<()> {
        if let Some(slot) = self.resolve_local(name) {
            self.emit(OpCode::GetLocal(slot), line);
        } else {
            let name_idx = self.intern_string(name);
            self.emit(OpCode::GetGlobal(name_idx), line);
        }
        Ok(())
    }

    /// Compile an enum variant constructor (e.g., Color::Red)
    fn compile_enum_variant(&mut self, enum_name: &str, variant: &str, line: usize) -> Result<()> {
        // Enum variants are represented as strings "EnumName::Variant"
        let value = format!("{}::{}", enum_name, variant);
        self.emit_const(Value::string(value), line);
        Ok(())
    }

    /// Compile a binary operation
    fn compile_binary(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
        line: usize,
    ) -> Result<()> {
        // Handle short-circuit operators specially
        match op {
            BinaryOp::And => return self.compile_and(left, right, line),
            BinaryOp::Or => return self.compile_or(left, right, line),
            _ => {}
        }

        // Regular binary operators
        self.compile_expr(left)?;
        self.compile_expr(right)?;

        let opcode = match op {
            BinaryOp::Add => OpCode::Add,
            BinaryOp::Sub => OpCode::Sub,
            BinaryOp::Mul => OpCode::Mul,
            BinaryOp::Div => OpCode::Div,
            BinaryOp::Mod => OpCode::Mod,
            BinaryOp::Eq => OpCode::Eq,
            BinaryOp::Ne => OpCode::Ne,
            BinaryOp::Lt => OpCode::Lt,
            BinaryOp::Le => OpCode::Le,
            BinaryOp::Gt => OpCode::Gt,
            BinaryOp::Ge => OpCode::Ge,
            BinaryOp::And | BinaryOp::Or => unreachable!(), // Handled above
        };

        self.emit(opcode, line);
        Ok(())
    }

    /// Compile short-circuit AND
    /// AND: if left is falsy, result is left; otherwise result is right
    fn compile_and(&mut self, left: &Expr, right: &Expr, line: usize) -> Result<()> {
        self.compile_expr(left)?;

        // Duplicate left value before the conditional jump
        // JumpIfFalse will pop the duplicate; we keep the original
        self.emit(OpCode::Dup, line);
        let end_jump = self.emit_jump(OpCode::JumpIfFalse(0), line);

        // Left was true - pop it and evaluate right instead
        self.emit(OpCode::Pop, line);
        self.compile_expr(right)?;

        self.patch_jump(end_jump);
        Ok(())
    }

    /// Compile short-circuit OR
    /// OR: if left is truthy, result is left; otherwise result is right
    fn compile_or(&mut self, left: &Expr, right: &Expr, line: usize) -> Result<()> {
        self.compile_expr(left)?;

        // Duplicate left value before the conditional jump
        // JumpIfTrue will pop the duplicate; we keep the original
        self.emit(OpCode::Dup, line);
        let end_jump = self.emit_jump(OpCode::JumpIfTrue(0), line);

        // Left was false - pop it and evaluate right instead
        self.emit(OpCode::Pop, line);
        self.compile_expr(right)?;

        self.patch_jump(end_jump);
        Ok(())
    }

    /// Compile a unary operation
    fn compile_unary(&mut self, op: UnaryOp, operand: &Expr, line: usize) -> Result<()> {
        self.compile_expr(operand)?;
        let opcode = match op {
            UnaryOp::Neg => OpCode::Neg,
            UnaryOp::Not => OpCode::Not,
        };
        self.emit(opcode, line);
        Ok(())
    }

    /// Compile a field access
    fn compile_field(&mut self, object: &Expr, field: &str, line: usize) -> Result<()> {
        self.compile_expr(object)?;
        let field_idx = self.intern_string(field);
        self.emit(OpCode::GetField(field_idx), line);
        Ok(())
    }

    /// Compile an index access
    fn compile_index(&mut self, object: &Expr, index: &Expr, line: usize) -> Result<()> {
        self.compile_expr(object)?;
        self.compile_expr(index)?;
        self.emit(OpCode::Index, line);
        Ok(())
    }

    /// Compile a function call
    fn compile_call(&mut self, callee: &Expr, args: &[Expr], line: usize) -> Result<()> {
        // Check if it's a native function call
        if let ExprKind::Var(name) = &callee.kind {
            if is_native(name) {
                // Compile arguments
                for arg in args {
                    self.compile_expr(arg)?;
                }
                let name_idx = self.intern_string(name);
                self.emit(OpCode::CallNative(name_idx, args.len()), line);
                return Ok(());
            }

            // Check if it's a known LLM function (from this cell or previous cells)
            let is_llm_func = self.llm_functions.iter().any(|f| &f.name == name)
                || self.known_llm_functions.contains(name);
            if is_llm_func {
                // Compile arguments
                for arg in args {
                    self.compile_expr(arg)?;
                }
                let name_idx = self.intern_string(name);
                self.emit(OpCode::LlmCall(name_idx), line);
                return Ok(());
            }

            // Check if it's a known user-defined function (either compiled or forward-declared)
            let is_user_func = self.functions.iter().any(|f| &f.name == name)
                || self.known_functions.contains(name);
            if is_user_func {
                // Compile arguments
                for arg in args {
                    self.compile_expr(arg)?;
                }
                let name_idx = self.intern_string(name);
                self.emit(OpCode::CallUser(name_idx, args.len()), line);
                return Ok(());
            }
        }

        // Regular function call (dynamic call via value on stack)
        // Compile arguments first (they'll be on the stack when we call)
        for arg in args {
            self.compile_expr(arg)?;
        }

        // Compile callee (function reference)
        self.compile_expr(callee)?;

        self.emit(OpCode::Call(args.len()), line);
        Ok(())
    }

    /// Compile a list literal
    fn compile_list(&mut self, items: &[Expr], line: usize) -> Result<()> {
        for item in items {
            self.compile_expr(item)?;
        }
        self.emit(OpCode::MakeList(items.len()), line);
        Ok(())
    }

    /// Compile a map literal
    fn compile_map(&mut self, pairs: &[(MapKey, Expr)], line: usize) -> Result<()> {
        for (key, value) in pairs {
            // Push key as string
            let key_str = match key {
                MapKey::String(s) => s.clone(),
                MapKey::Ident(s) => s.clone(),
            };
            self.emit_const(Value::string(key_str), line);
            self.compile_expr(value)?;
        }
        self.emit(OpCode::MakeMap(pairs.len()), line);
        Ok(())
    }

    /// Compile a struct literal
    fn compile_struct(&mut self, name: &str, fields: &[(String, Expr)], line: usize) -> Result<()> {
        // Push field name/value pairs
        for (field_name, field_value) in fields {
            self.emit_const(Value::string(field_name.as_str()), line);
            self.compile_expr(field_value)?;
        }
        let name_idx = self.intern_string(name);
        self.emit(OpCode::MakeStruct(name_idx, fields.len()), line);
        Ok(())
    }

    /// Compile an if expression (returns a value)
    fn compile_if_expr(
        &mut self,
        condition: &Expr,
        then_branch: &Block,
        else_branch: &Option<IfExprElse>,
        line: usize,
    ) -> Result<()> {
        // Compile condition
        self.compile_expr(condition)?;

        // Jump to else (or end) if false
        let jump_to_else = self.emit_jump(OpCode::JumpIfFalse(0), line);

        // Compile then branch (keep result on stack)
        self.compile_block(then_branch)?;

        if let Some(else_clause) = else_branch {
            // Jump over else
            let jump_over_else = self.emit_jump(OpCode::Jump(0), line);

            // Patch jump to else
            self.patch_jump(jump_to_else);

            // Compile else branch (keep result on stack)
            match else_clause {
                IfExprElse::Else(block) => {
                    self.compile_block(block)?;
                }
                IfExprElse::ElseIf(if_expr) => {
                    self.compile_expr(if_expr)?;
                }
            }

            // Patch jump over else
            self.patch_jump(jump_over_else);
        } else {
            // No else branch - push null as the value
            let jump_over_null = self.emit_jump(OpCode::Jump(0), line);
            self.patch_jump(jump_to_else);
            self.emit_const(Value::Null, line);
            self.patch_jump(jump_over_null);
        }

        Ok(())
    }

    /// Compile a match expression
    fn compile_match(&mut self, scrutinee: &Expr, arms: &[MatchArm], line: usize) -> Result<()> {
        // Compile scrutinee
        self.compile_expr(scrutinee)?;

        let mut end_jumps = Vec::new();

        for (i, arm) in arms.iter().enumerate() {
            let is_last = i == arms.len() - 1;

            // Duplicate scrutinee for pattern matching
            if !is_last {
                self.emit(OpCode::Dup, line);
            }

            // Compile pattern test
            let next_arm_jump = self.compile_pattern(&arm.pattern, line)?;

            // Pattern matched - pop scrutinee (copy for non-last arms, original for last arm)
            self.emit(OpCode::Pop, line);

            match &arm.body {
                MatchArmBody::Expr(expr) => self.compile_expr(expr)?,
                MatchArmBody::Block(block) => self.compile_block(block)?,
            }

            // Jump to end
            if !is_last {
                end_jumps.push(self.emit_jump(OpCode::Jump(0), line));
            }

            // Patch jump to next arm
            if let Some(jump) = next_arm_jump {
                self.patch_jump(jump);
            }
        }

        // Patch all end jumps
        for jump in end_jumps {
            self.patch_jump(jump);
        }

        Ok(())
    }

    /// Compile a pattern and return the jump offset for the next arm (if any)
    fn compile_pattern(&mut self, pattern: &Pattern, line: usize) -> Result<Option<usize>> {
        match &pattern.kind {
            PatternKind::Wildcard => {
                // Always matches, no test needed
                Ok(None)
            }
            PatternKind::Binding(name) => {
                // Always matches, bind the value
                self.emit(OpCode::Dup, line);
                self.add_local(name.clone());
                Ok(None)
            }
            PatternKind::Literal(lit) => {
                // Compare with literal
                self.compile_literal(lit, line)?;
                self.emit(OpCode::Eq, line);
                let jump = self.emit_jump(OpCode::JumpIfFalse(0), line);
                Ok(Some(jump))
            }
            PatternKind::Enum { enum_name, variant } => {
                // Compare as string "EnumName::Variant"
                let full_name = format!("{}::{}", enum_name, variant);
                self.emit_const(Value::string(full_name), line);
                self.emit(OpCode::Eq, line);
                let jump = self.emit_jump(OpCode::JumpIfFalse(0), line);
                Ok(Some(jump))
            }
            PatternKind::Result { is_ok, binding } => {
                // Check __type field
                self.emit(OpCode::Dup, line);
                let type_idx = self.intern_string("__type");
                self.emit(OpCode::GetField(type_idx), line);
                let expected = if *is_ok { "Ok" } else { "Err" };
                self.emit_const(Value::string(expected), line);
                self.emit(OpCode::Eq, line);
                let jump = self.emit_jump(OpCode::JumpIfFalse(0), line);

                // Extract and bind the inner value
                self.emit(OpCode::Dup, line);
                let value_idx = self.intern_string("value");
                self.emit(OpCode::GetField(value_idx), line);
                self.add_local(binding.clone());

                Ok(Some(jump))
            }
        }
    }

    /// Compile a lambda expression
    fn compile_lambda(&mut self, params: &[String], body: &LambdaBody, line: usize) -> Result<()> {
        // Create a unique name for the lambda
        let name = self.generate_unique_name("__lambda");

        // Create a new compiler for the lambda body
        let mut lambda_compiler = Compiler::new();
        lambda_compiler.scope_depth = 1;
        // Copy known functions so lambdas can call user-defined functions
        lambda_compiler.known_functions = self.known_functions.clone();
        lambda_compiler.known_llm_functions = self.known_llm_functions.clone();
        // Copy llm_functions so LLM functions defined in same file can be detected for map_row
        lambda_compiler.llm_functions = self.llm_functions.clone();
        // Copy unique_counter to avoid lambda name collisions
        lambda_compiler.unique_counter = self.unique_counter;

        // Add parameters as locals
        for param in params {
            lambda_compiler.add_local(param.clone());
        }

        // Compile body
        match body {
            LambdaBody::Expr(expr) => lambda_compiler.compile_expr(expr)?,
            LambdaBody::Block(block) => lambda_compiler.compile_block(block)?,
        }

        lambda_compiler.emit(OpCode::Return, line);

        let compiled = CompiledFunction {
            name: name.clone(),
            arity: params.len(),
            local_count: lambda_compiler.locals.len(),
            chunk: lambda_compiler.chunk,
        };

        self.functions.push(compiled);

        // Collect any nested lambdas compiled inside the lambda body
        self.functions.extend(lambda_compiler.functions);

        // Update unique_counter to avoid collisions with future compilations
        self.unique_counter = lambda_compiler.unique_counter;

        // For now, push the function name as a string reference
        // The VM will need to look it up
        self.emit_const(Value::string(name), line);

        Ok(())
    }

    /// Compile a parallel block
    fn compile_parallel(&mut self, exprs: &[Expr], line: usize) -> Result<()> {
        for expr in exprs {
            self.compile_expr(expr)?;
        }
        self.emit(OpCode::Parallel(exprs.len()), line);
        Ok(())
    }

    /// Compile a parallel map
    ///
    /// Detects if the mapper is a simple `|x| llm_fn(x)` pattern and emits
    /// the specialized `ParallelLlmMap` opcode for parallel HTTP calls.
    /// Otherwise falls back to the general `ParallelMap` opcode.
    fn compile_parallel_map(&mut self, collection: &Expr, mapper: &Expr, line: usize) -> Result<()> {
        // Try to detect the `|x| llm_fn(x)` pattern for LLM functions
        if let ExprKind::Lambda { params, body } = &mapper.kind {
            if params.len() == 1 {
                if let LambdaBody::Expr(body_expr) = body.as_ref() {
                    if let ExprKind::Call { callee, args } = &body_expr.kind {
                        // Check if it's a simple call with the lambda param as the only arg
                        if args.len() == 1 {
                            if let ExprKind::Var(arg_name) = &args[0].kind {
                                if arg_name == &params[0] {
                                    // Check if callee is an LLM function
                                    if let ExprKind::Var(func_name) = &callee.kind {
                                        let is_llm_fn = self.known_llm_functions.contains(func_name)
                                            || self.llm_functions.iter().any(|f| f.name == *func_name);

                                        if is_llm_fn {
                                            // Emit specialized parallel LLM map
                                            self.compile_expr(collection)?;
                                            let func_name_idx = self.intern_string(func_name);
                                            self.emit(OpCode::ParallelLlmMap(func_name_idx), line);
                                            return Ok(());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback: general parallel map (sequential for now)
        self.compile_expr(collection)?;
        self.compile_expr(mapper)?;
        self.emit(OpCode::ParallelMap, line);
        Ok(())
    }

    /// Compile a map_column expression
    ///
    /// Detects if the mapper is a simple `|x| llm_fn(x)` pattern and emits
    /// the specialized `MapColumnLlm` opcode for parallel LLM column mapping.
    /// Otherwise falls back to the general `MapColumn` opcode.
    fn compile_map_column(
        &mut self,
        table: &Expr,
        input_col: &Expr,
        output_col: &Expr,
        mapper: &Expr,
        line: usize,
    ) -> Result<()> {
        // Try to detect the `|x| llm_fn(x)` pattern for LLM functions
        if let ExprKind::Lambda { params, body } = &mapper.kind {
            if params.len() == 1 {
                if let LambdaBody::Expr(body_expr) = body.as_ref() {
                    if let ExprKind::Call { callee, args } = &body_expr.kind {
                        // Check if it's a simple call with the lambda param as the only arg
                        if args.len() == 1 {
                            if let ExprKind::Var(arg_name) = &args[0].kind {
                                if arg_name == &params[0] {
                                    // Check if callee is an LLM function
                                    if let ExprKind::Var(func_name) = &callee.kind {
                                        let is_llm_fn = self.known_llm_functions.contains(func_name)
                                            || self.llm_functions.iter().any(|f| f.name == *func_name);

                                        if is_llm_fn {
                                            // Emit specialized parallel LLM map column
                                            self.compile_expr(table)?;
                                            self.compile_expr(input_col)?;
                                            self.compile_expr(output_col)?;
                                            let func_name_idx = self.intern_string(func_name);
                                            self.emit(OpCode::MapColumnLlm(func_name_idx), line);
                                            return Ok(());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback: general map column (sequential for now)
        self.compile_expr(table)?;
        self.compile_expr(input_col)?;
        self.compile_expr(output_col)?;
        self.compile_expr(mapper)?;
        self.emit(OpCode::MapColumn, line);
        Ok(())
    }

    /// Compile a map_row expression
    ///
    /// map_row applies a lambda to each entire row, adding the result as a new column.
    /// Unlike map_column which extracts a single column value, map_row passes the
    /// entire row map to the lambda, allowing access to multiple columns.
    ///
    /// This function also supports the `$field` syntax. If the mapper expression contains
    /// `$field` references, it will be automatically wrapped in a lambda:
    ///   `$a + $b` becomes `|__row__| __row__["a"] + __row__["b"]`
    fn compile_map_row(
        &mut self,
        table: &Expr,
        output_col: &Expr,
        mapper: &Expr,
        line: usize,
    ) -> Result<()> {
        // Check if mapper contains $field and needs desugaring
        let mapper = if contains_dollar_field(mapper) {
            wrap_dollar_expr_in_lambda(mapper)
        } else {
            mapper.clone()
        };
        let mapper = &mapper;

        // Try to detect the `|row| llm_fn(row["col1"], row["col2"])` pattern for LLM functions
        if let ExprKind::Lambda { params, body } = &mapper.kind {
            if params.len() == 1 {
                let row_param = &params[0];
                if let LambdaBody::Expr(body_expr) = body.as_ref() {
                    if let ExprKind::Call { callee, args } = &body_expr.kind {
                        // Check if callee is an LLM function
                        if let ExprKind::Var(func_name) = &callee.kind {
                            let is_llm_fn = self.known_llm_functions.contains(func_name)
                                || self.llm_functions.iter().any(|f| f.name == *func_name);

                            if is_llm_fn {
                                // Extract column names from the lambda call arguments
                                // Each arg should be row["col_name"] or row.col_name
                                let column_names: Vec<String> = args.iter().map(|arg| {
                                    Self::extract_column_name(arg, row_param)
                                }).collect();

                                // Join column names with comma separator
                                let mappings_str = column_names.join(",");
                                let mappings_idx = self.intern_string(&mappings_str);

                                // Emit specialized parallel LLM map row
                                self.compile_expr(table)?;
                                self.compile_expr(output_col)?;
                                self.compile_expr(mapper)?;
                                let func_name_idx = self.intern_string(func_name);
                                self.emit(OpCode::MapRowLlm(func_name_idx, mappings_idx), line);
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }

        // Fallback: general map row (sequential for now)
        self.compile_expr(table)?;
        self.compile_expr(output_col)?;
        self.compile_expr(mapper)?;
        self.emit(OpCode::MapRow, line);
        Ok(())
    }

    /// Extract column name from a lambda argument expression.
    ///
    /// Handles patterns like:
    /// - `row["column_name"]` -> "column_name"
    /// - `row.column_name` -> "column_name"
    ///
    /// Returns empty string if the pattern is not recognized.
    fn extract_column_name(expr: &Expr, row_param: &str) -> String {
        match &expr.kind {
            // row["column_name"] - Index access with string literal
            ExprKind::Index { object, index } => {
                if let ExprKind::Var(var_name) = &object.kind {
                    if var_name == row_param {
                        if let ExprKind::Literal(Literal::String(col_name)) = &index.kind {
                            return col_name.clone();
                        }
                    }
                }
            }
            // row.column_name - Field access
            ExprKind::Field { object, field } => {
                if let ExprKind::Var(var_name) = &object.kind {
                    if var_name == row_param {
                        return field.clone();
                    }
                }
            }
            _ => {}
        }
        // Return empty string for unrecognized patterns
        // The VM will handle this by using Value::Null
        String::new()
    }

    /// Compile an explode expression
    fn compile_explode(
        &mut self,
        table: &Expr,
        column: &Expr,
        prefix: &Option<Box<Expr>>,
        line: usize,
    ) -> Result<()> {
        // Compile table expression
        self.compile_expr(table)?;

        // Compile column name
        self.compile_expr(column)?;

        // Compile prefix (or push null if not provided)
        if let Some(prefix_expr) = prefix {
            self.compile_expr(prefix_expr)?;
        } else {
            self.emit_const(Value::Null, line);
        }

        self.emit(OpCode::Explode, line);
        Ok(())
    }

    /// Compile a SQL expression
    fn compile_sql(&mut self, ty: &Option<TypeAnnotation>, query: &Expr, line: usize) -> Result<()> {
        self.compile_expr(query)?;
        if let Some(type_ann) = ty {
            let type_name = get_type_name(&type_ann.ty);
            let type_name_idx = self.intern_string(&type_name);
            self.emit(OpCode::SqlQueryTyped(type_name_idx), line);
        } else {
            self.emit(OpCode::SqlQuery, line);
        }
        Ok(())
    }

    /// Compile an f-string (interpolated string)
    ///
    /// F-strings are compiled as a series of string concatenations.
    /// Each part (text or expression) is pushed onto the stack, then
    /// concatenated together using the Add operation.
    fn compile_fstring(&mut self, parts: &[FStringPart], line: usize) -> Result<()> {
        if parts.is_empty() {
            // Empty f-string: f""
            self.emit_const(Value::string(""), line);
            return Ok(());
        }

        // Compile the first part
        match &parts[0] {
            FStringPart::Text(s) => {
                self.emit_const(Value::string(s.as_str()), line);
            }
            FStringPart::Expr(expr) => {
                self.compile_expr(expr)?;
                // Convert to string using the str() builtin
                self.emit(OpCode::Stringify, line);
            }
        }

        // Compile remaining parts with concatenation
        for part in &parts[1..] {
            match part {
                FStringPart::Text(s) => {
                    self.emit_const(Value::string(s.as_str()), line);
                }
                FStringPart::Expr(expr) => {
                    self.compile_expr(expr)?;
                    // Convert to string
                    self.emit(OpCode::Stringify, line);
                }
            }
            // Concatenate with previous result
            self.emit(OpCode::Add, line);
        }

        Ok(())
    }

    /// Compile a block
    fn compile_block(&mut self, block: &Block) -> Result<()> {
        let line = block.span.line;
        self.begin_scope();

        // Compile all statements
        for stmt in &block.stmts {
            self.compile_stmt(stmt)?;
        }

        // If there's a trailing expression, compile it (leaving value on stack)
        // and use PopBelow to preserve it while cleaning up locals
        if let Some(expr) = &block.expr {
            self.compile_expr(expr)?;
            self.end_scope_preserve_top(line);
        } else {
            // No trailing expression - push null, then pop normally
            self.emit_const(Value::Null, line);
            self.end_scope(line);
        }

        Ok(())
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Emit an opcode
    fn emit(&mut self, op: OpCode, line: usize) {
        self.chunk.write(op, line);
    }

    /// Emit a constant and return its index
    fn emit_const(&mut self, value: Value, line: usize) {
        let idx = self.chunk.add_constant(value);
        self.emit(OpCode::Const(idx), line);
    }

    /// Intern a string in the chunk's string table and return its index
    fn intern_string(&mut self, s: &str) -> usize {
        self.chunk.intern_string(s)
    }

    /// Emit a jump instruction and return its offset for patching
    fn emit_jump(&mut self, op: OpCode, line: usize) -> usize {
        self.emit(op, line);
        self.chunk.len() - 1
    }

    /// Patch a jump instruction to jump to the current location
    fn patch_jump(&mut self, offset: usize) {
        self.chunk.patch_jump(offset);
    }

    /// Begin a new scope
    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    /// End the current scope
    fn end_scope(&mut self, line: usize) {
        self.scope_depth -= 1;

        // Pop locals that are going out of scope
        while !self.locals.is_empty() && self.locals.last().unwrap().depth > self.scope_depth {
            self.emit(OpCode::Pop, line);
            self.locals.pop();
        }
    }

    /// End the current scope while preserving the top of stack (return value)
    fn end_scope_preserve_top(&mut self, line: usize) {
        self.scope_depth -= 1;

        // Count how many locals need to be popped
        let mut pop_count = 0;
        while !self.locals.is_empty() && self.locals.last().unwrap().depth > self.scope_depth {
            pop_count += 1;
            self.locals.pop();
        }

        // Use PopBelow to pop locals while preserving the top value
        if pop_count > 0 {
            self.emit(OpCode::PopBelow(pop_count), line);
        }
    }

    /// Add a local variable
    fn add_local(&mut self, name: String) {
        self.locals.push(Local {
            name,
            depth: self.scope_depth,
        });
    }

    /// Resolve a local variable, returning its slot index if found
    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }
        None
    }

    /// Generate a unique name for hidden variables
    fn generate_unique_name(&mut self, prefix: &str) -> String {
        let name = format!("{}_{}", prefix, self.unique_counter);
        self.unique_counter += 1;
        name
    }
}

// ========================================================================
// Type Conversion Helpers
// ========================================================================

/// Convert an AST type annotation to IR FieldType
fn convert_type_annotation(ty: &TypeAnnotation) -> FieldType {
    convert_type_expr(&ty.ty)
}

/// Convert an AST type expression to IR FieldType
fn convert_type_expr(ty: &TypeExpr) -> FieldType {
    match ty {
        TypeExpr::Primitive(PrimitiveType::String) => FieldType::String,
        TypeExpr::Primitive(PrimitiveType::Int) => FieldType::Int,
        TypeExpr::Primitive(PrimitiveType::Float) => FieldType::Float,
        TypeExpr::Primitive(PrimitiveType::Bool) => FieldType::Bool,
        TypeExpr::Primitive(PrimitiveType::Null) => FieldType::String, // No direct null type, use String
        TypeExpr::Primitive(PrimitiveType::Path) => FieldType::Path,
        TypeExpr::Named(name) => FieldType::Class(name.clone()),
        TypeExpr::List(inner) => FieldType::List(Box::new(convert_type_annotation(inner))),
        TypeExpr::Map(k, v) => FieldType::Map(
            Box::new(convert_type_annotation(k)),
            Box::new(convert_type_annotation(v)),
        ),
        TypeExpr::Result(ok, _err) => {
            // For Result<T, E>, we use the Ok type
            // A proper implementation would use Union or a Result type
            convert_type_annotation(ok)
        }
    }
}

/// Convert an AST ProviderConfig to bytecode ProviderConfig
fn convert_provider_config(
    ast_provider: &crate::syntax::ast::ProviderConfig,
) -> crate::vm::bytecode::ProviderConfig {
    crate::vm::bytecode::ProviderConfig {
        order: ast_provider.order.clone(),
        only: ast_provider.only.clone(),
        ignore: ast_provider.ignore.clone(),
        allow_fallbacks: ast_provider.allow_fallbacks,
        require_parameters: ast_provider.require_parameters,
        data_collection: ast_provider.data_collection.clone(),
        zdr: ast_provider.zdr,
        sort: ast_provider.sort.clone(),
        quantizations: ast_provider.quantizations.clone(),
    }
}

/// Check if a prompt expression contains complex expressions that need runtime evaluation.
///
/// Complex expressions are anything other than:
/// - Simple variable references: {x}
/// - Field accesses: {x.field}
///
/// Examples of complex expressions:
/// - If expressions: {if x > 10 { "big" } else { "small" }}
/// - Binary operations: {x + y}
/// - Function calls: {format(x)}
fn prompt_has_complex_expressions(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Literal(Literal::String(_)) => false,
        ExprKind::FString(parts) => {
            for part in parts {
                if let FStringPart::Expr(e) = part {
                    if !is_simple_template_expr(e) {
                        return true;
                    }
                }
            }
            false
        }
        // Non-string prompt expressions are complex
        _ => true,
    }
}

/// Check if an expression is simple enough for template substitution.
/// Simple expressions are variables and field accesses.
fn is_simple_template_expr(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Var(_) => true,
        ExprKind::Field { object, .. } => is_simple_template_expr(object),
        _ => false,
    }
}

/// Convert a prompt expression to a template string
///
/// This converts:
/// - String literals: "hello" -> "hello"
/// - Raw strings: """hello""" -> "hello"
/// - F-strings: f"hello {x}" -> "hello {{ x }}"
///
/// The f-string interpolations {var} are converted to Tera template syntax {{ var }}
fn prompt_expr_to_template(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Literal(Literal::String(s)) => s.clone(),
        ExprKind::FString(parts) => {
            let mut result = String::new();
            for part in parts {
                match part {
                    FStringPart::Text(s) => result.push_str(s),
                    FStringPart::Expr(e) => {
                        // Convert expression to template variable
                        // For simple variable references, use the variable name directly
                        // For more complex expressions, we stringify them
                        result.push_str("{{ ");
                        result.push_str(&expr_to_template_var(e));
                        result.push_str(" }}");
                    }
                }
            }
            result
        }
        // Fallback: return empty string (shouldn't happen in well-formed code)
        _ => String::new(),
    }
}

/// Convert an expression to a template variable name
///
/// This handles simple variable references and field accesses.
/// For complex expressions, we fallback to a simple representation.
fn expr_to_template_var(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Var(name) => name.clone(),
        ExprKind::Field { object, field } => {
            format!("{}.{}", expr_to_template_var(object), field)
        }
        // For other expressions, we can't directly translate them to template syntax
        // In a more complete implementation, we might error here or use a different approach
        _ => "???".to_string(),
    }
}

/// Get the type name from a type expression (for SQL typed queries)
fn get_type_name(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Primitive(p) => match p {
            PrimitiveType::String => "String".to_string(),
            PrimitiveType::Int => "Int".to_string(),
            PrimitiveType::Float => "Float".to_string(),
            PrimitiveType::Bool => "Bool".to_string(),
            PrimitiveType::Null => "Null".to_string(),
            PrimitiveType::Path => "Path".to_string(),
        },
        TypeExpr::Named(name) => name.clone(),
        TypeExpr::List(inner) => format!("[{}]", get_type_name(&inner.ty)),
        TypeExpr::Map(k, v) => format!("Map<{}, {}>", get_type_name(&k.ty), get_type_name(&v.ty)),
        TypeExpr::Result(ok, err) => {
            format!("Result<{}, {}>", get_type_name(&ok.ty), get_type_name(&err.ty))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::LatticeError;
    use crate::syntax::parser;
    use crate::vm::VM;

    fn compile_and_run(source: &str) -> Result<Value> {
        let program = parser::parse(source).map_err(|e| LatticeError::Parse(e.to_string()))?;
        let result = Compiler::compile(&program)?;
        let mut vm = VM::new();

        // Register types
        for class in result.classes {
            vm.ir_mut().classes.push(class);
        }
        for enum_def in result.enums {
            vm.ir_mut().enums.push(enum_def);
        }

        // Register functions
        for func in result.functions {
            vm.register_function(func);
        }
        for llm_func in result.llm_functions {
            vm.register_llm_function(llm_func);
        }

        vm.run(&result.chunk)
    }

    // ========================================================================
    // Literal Tests
    // ========================================================================

    #[test]
    fn test_int_literal() {
        let result = compile_and_run("42").unwrap();
        assert!(matches!(result, Value::Int(42)));
    }

    #[test]
    fn test_float_literal() {
        let result = compile_and_run("3.14").unwrap();
        match result {
            Value::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_string_literal() {
        let result = compile_and_run("\"hello\"").unwrap();
        match result {
            Value::String(s) => assert_eq!(&*s, "hello"),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_bool_literal() {
        let result = compile_and_run("true").unwrap();
        assert!(matches!(result, Value::Bool(true)));

        let result = compile_and_run("false").unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_null_literal() {
        let result = compile_and_run("null").unwrap();
        assert!(matches!(result, Value::Null));
    }

    // ========================================================================
    // Arithmetic Tests
    // ========================================================================

    #[test]
    fn test_addition() {
        let result = compile_and_run("1 + 2").unwrap();
        assert!(matches!(result, Value::Int(3)));
    }

    #[test]
    fn test_subtraction() {
        let result = compile_and_run("5 - 3").unwrap();
        assert!(matches!(result, Value::Int(2)));
    }

    #[test]
    fn test_multiplication() {
        let result = compile_and_run("6 * 7").unwrap();
        assert!(matches!(result, Value::Int(42)));
    }

    #[test]
    fn test_division() {
        let result = compile_and_run("10 / 2").unwrap();
        assert!(matches!(result, Value::Int(5)));
    }

    #[test]
    fn test_modulo() {
        let result = compile_and_run("17 % 5").unwrap();
        assert!(matches!(result, Value::Int(2)));
    }

    #[test]
    fn test_negation() {
        let result = compile_and_run("-42").unwrap();
        assert!(matches!(result, Value::Int(-42)));
    }

    #[test]
    fn test_operator_precedence() {
        let result = compile_and_run("1 + 2 * 3").unwrap();
        assert!(matches!(result, Value::Int(7)));

        let result = compile_and_run("(1 + 2) * 3").unwrap();
        assert!(matches!(result, Value::Int(9)));
    }

    // ========================================================================
    // Comparison Tests
    // ========================================================================

    #[test]
    fn test_equality() {
        let result = compile_and_run("1 == 1").unwrap();
        assert!(matches!(result, Value::Bool(true)));

        let result = compile_and_run("1 == 2").unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_inequality() {
        let result = compile_and_run("1 != 2").unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_less_than() {
        let result = compile_and_run("1 < 2").unwrap();
        assert!(matches!(result, Value::Bool(true)));

        let result = compile_and_run("2 < 1").unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_greater_than() {
        let result = compile_and_run("2 > 1").unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    // ========================================================================
    // Logic Tests
    // ========================================================================

    #[test]
    fn test_not() {
        let result = compile_and_run("!true").unwrap();
        assert!(matches!(result, Value::Bool(false)));

        let result = compile_and_run("!false").unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_and_short_circuit() {
        let result = compile_and_run("true && true").unwrap();
        assert!(matches!(result, Value::Bool(true)));

        let result = compile_and_run("true && false").unwrap();
        assert!(matches!(result, Value::Bool(false)));

        let result = compile_and_run("false && true").unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    #[test]
    fn test_or_short_circuit() {
        let result = compile_and_run("true || false").unwrap();
        assert!(matches!(result, Value::Bool(true)));

        let result = compile_and_run("false || true").unwrap();
        assert!(matches!(result, Value::Bool(true)));

        let result = compile_and_run("false || false").unwrap();
        assert!(matches!(result, Value::Bool(false)));
    }

    // ========================================================================
    // Variable Tests
    // ========================================================================

    #[test]
    fn test_let_binding() {
        let result = compile_and_run("let x = 42\nx").unwrap();
        assert!(matches!(result, Value::Int(42)));
    }

    #[test]
    fn test_multiple_bindings() {
        let result = compile_and_run("let x = 10\nlet y = 20\nx + y").unwrap();
        assert!(matches!(result, Value::Int(30)));
    }

    #[test]
    fn test_assignment() {
        let result = compile_and_run("let x = 1\nx = 42\nx").unwrap();
        assert!(matches!(result, Value::Int(42)));
    }

    // ========================================================================
    // Collection Tests
    // ========================================================================

    #[test]
    fn test_list_literal() {
        let result = compile_and_run("[1, 2, 3]").unwrap();
        match result {
            Value::List(l) => {
                assert_eq!(l.len(), 3);
                assert!(matches!(l[0], Value::Int(1)));
                assert!(matches!(l[1], Value::Int(2)));
                assert!(matches!(l[2], Value::Int(3)));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_map_literal() {
        let result = compile_and_run("{\"a\": 1, \"b\": 2}").unwrap();
        match result {
            Value::Map(m) => {
                assert_eq!(m.len(), 2);
                assert!(matches!(m.get("a"), Some(Value::Int(1))));
                assert!(matches!(m.get("b"), Some(Value::Int(2))));
            }
            _ => panic!("Expected Map"),
        }
    }

    #[test]
    fn test_index_access() {
        let result = compile_and_run("let x = [10, 20, 30]\nx[1]").unwrap();
        assert!(matches!(result, Value::Int(20)));
    }

    // ========================================================================
    // Control Flow Tests
    // ========================================================================

    #[test]
    fn test_if_true() {
        let result = compile_and_run("let x = 0\nif true { x = 42 }\nx").unwrap();
        assert!(matches!(result, Value::Int(42)));
    }

    #[test]
    fn test_if_false() {
        let result = compile_and_run("let x = 0\nif false { x = 42 }\nx").unwrap();
        assert!(matches!(result, Value::Int(0)));
    }

    #[test]
    fn test_if_else() {
        let result = compile_and_run("let x = 0\nif false { x = 1 } else { x = 2 }\nx").unwrap();
        assert!(matches!(result, Value::Int(2)));
    }

    #[test]
    fn test_while_loop() {
        let result = compile_and_run(
            "let x = 0
while x < 5 {
    x = x + 1
}
x",
        )
        .unwrap();
        assert!(matches!(result, Value::Int(5)));
    }

    #[test]
    fn test_for_loop() {
        let result = compile_and_run(
            "let sum = 0
for i in [1, 2, 3] {
    sum = sum + i
}
sum",
        )
        .unwrap();
        assert!(matches!(result, Value::Int(6)));
    }

    // ========================================================================
    // Native Function Tests
    // ========================================================================

    #[test]
    fn test_len() {
        let result = compile_and_run("len([1, 2, 3])").unwrap();
        assert!(matches!(result, Value::Int(3)));
    }

    // Note: `type` is a keyword in Lattice, so we can't use type() as a function call
    // The native function is still available but we'd need to use it differently
    // For now, we skip this test since it conflicts with the type keyword

    // ========================================================================
    // Type Definition Tests
    // ========================================================================

    #[test]
    fn test_type_definition() {
        let program = parser::parse(
            "type Person {
    name: String,
    age: Int
}",
        )
        .unwrap();
        let result = Compiler::compile(&program).unwrap();
        assert_eq!(result.classes.len(), 1);
        assert_eq!(result.classes[0].name, "Person");
        assert_eq!(result.classes[0].fields.len(), 2);
    }

    #[test]
    fn test_enum_definition() {
        let program = parser::parse(
            "enum Color {
    Red,
    Green,
    Blue
}",
        )
        .unwrap();
        let result = Compiler::compile(&program).unwrap();
        assert_eq!(result.enums.len(), 1);
        assert_eq!(result.enums[0].name, "Color");
        assert_eq!(result.enums[0].values.len(), 3);
    }

    // ========================================================================
    // Function Tests
    // ========================================================================

    #[test]
    fn test_function_definition() {
        let program = parser::parse(
            "def add(a: Int, b: Int) -> Int {
    a + b
}",
        )
        .unwrap();
        let result = Compiler::compile(&program).unwrap();
        assert_eq!(result.functions.len(), 1);
        assert_eq!(result.functions[0].name, "add");
        assert_eq!(result.functions[0].arity, 2);
    }

    // Note: Full function call tests require VM support for user-defined function calls

    // ========================================================================
    // F-String Tests
    // ========================================================================

    #[test]
    fn test_fstring_simple() {
        let result = compile_and_run(r#"f"hello world""#).unwrap();
        match result {
            Value::String(s) => assert_eq!(&*s, "hello world"),
            _ => panic!("Expected String, got {:?}", result),
        }
    }

    #[test]
    fn test_fstring_with_variable() {
        let result = compile_and_run(r#"let name = "Alice"
f"Hello, {name}!""#).unwrap();
        match result {
            Value::String(s) => assert_eq!(&*s, "Hello, Alice!"),
            _ => panic!("Expected String, got {:?}", result),
        }
    }

    #[test]
    fn test_fstring_with_expression() {
        let result = compile_and_run(r#"f"The answer is {40 + 2}""#).unwrap();
        match result {
            Value::String(s) => assert_eq!(&*s, "The answer is 42"),
            _ => panic!("Expected String, got {:?}", result),
        }
    }

    #[test]
    fn test_fstring_multiple_interpolations() {
        let result = compile_and_run(r#"let x = 10
let y = 20
f"{x} + {y} = {x + y}""#).unwrap();
        match result {
            Value::String(s) => assert_eq!(&*s, "10 + 20 = 30"),
            _ => panic!("Expected String, got {:?}", result),
        }
    }

    #[test]
    fn test_fstring_empty() {
        let result = compile_and_run(r#"f"""#).unwrap();
        match result {
            Value::String(s) => assert_eq!(&*s, ""),
            _ => panic!("Expected String, got {:?}", result),
        }
    }

    #[test]
    fn test_fstring_in_sql() {
        // This tests that f-strings work with SQL (the main use case)
        let result = compile_and_run(r#"let table = "users"
f"SELECT * FROM {table}""#).unwrap();
        match result {
            Value::String(s) => assert_eq!(&*s, "SELECT * FROM users"),
            _ => panic!("Expected String, got {:?}", result),
        }
    }

    // ========================================================================
    // Parallel LLM Map Tests
    // ========================================================================

    /// Helper to check if compiled code contains ParallelLlmMap opcode
    fn has_parallel_llm_map(source: &str) -> bool {
        use crate::vm::bytecode::OpCode;

        let program = parser::parse(source).expect("parse failed");
        let result = Compiler::compile(&program).expect("compile failed");

        result.chunk.code.iter().any(|op| matches!(op, OpCode::ParallelLlmMap(_)))
    }

    /// Helper to check if compiled code contains ParallelMap opcode
    fn has_parallel_map(source: &str) -> bool {
        use crate::vm::bytecode::OpCode;

        let program = parser::parse(source).expect("parse failed");
        let result = Compiler::compile(&program).expect("compile failed");

        result.chunk.code.iter().any(|op| matches!(op, OpCode::ParallelMap))
    }

    #[test]
    fn test_parallel_map_llm_detection() {
        // When parallel_map is called with a lambda that calls an LLM function,
        // the compiler should emit ParallelLlmMap instead of ParallelMap
        let source = r#"def summarize(text: String) -> String {
    base_url: "https://api.openai.com/v1"
    model: "gpt-4"
    api_key_env: "OPENAI_API_KEY"
    prompt: "Summarize: ${text}"
}

let docs = ["doc1", "doc2", "doc3"]
parallel_map(docs, |d| summarize(d))"#;

        assert!(has_parallel_llm_map(source), "Expected ParallelLlmMap opcode for LLM function pattern");
        assert!(!has_parallel_map(source), "Should not have ParallelMap when ParallelLlmMap is used");
    }

    #[test]
    fn test_parallel_map_regular_function_fallback() {
        // When parallel_map is called with a lambda that calls a regular function,
        // the compiler should emit ParallelMap (fallback)
        let source = r#"def double(x: Int) -> Int {
    x * 2
}

let nums = [1, 2, 3]
parallel_map(nums, |n| double(n))"#;

        assert!(has_parallel_map(source), "Expected ParallelMap opcode for regular function");
        assert!(!has_parallel_llm_map(source), "Should not have ParallelLlmMap for regular function");
    }

    #[test]
    fn test_parallel_map_complex_lambda_fallback() {
        // When the lambda body is more complex than just a function call,
        // fall back to ParallelMap
        let source = r#"def summarize(text: String) -> String {
    base_url: "https://api.openai.com/v1"
    model: "gpt-4"
    api_key_env: "OPENAI_API_KEY"
    prompt: "Summarize: ${text}"
}

let docs = ["doc1", "doc2", "doc3"]
parallel_map(docs, |d| summarize(d + " suffix"))"#;

        // The lambda body is `summarize(d + " suffix")` which is more complex
        // than just `summarize(d)`, so it should fall back to ParallelMap
        assert!(has_parallel_map(source), "Expected ParallelMap for complex lambda");
    }

    // ========================================================================
    // map_column Compiler Tests
    // ========================================================================

    /// Helper to check if compiled code contains MapColumn opcode
    fn has_map_column(source: &str) -> bool {
        use crate::vm::bytecode::OpCode;

        let program = parser::parse(source).expect("parse failed");
        let result = Compiler::compile(&program).expect("compile failed");

        result.chunk.code.iter().any(|op| matches!(op, OpCode::MapColumn))
    }

    /// Helper to check if compiled code contains MapColumnLlm opcode
    fn has_map_column_llm(source: &str) -> bool {
        use crate::vm::bytecode::OpCode;

        let program = parser::parse(source).expect("parse failed");
        let result = Compiler::compile(&program).expect("compile failed");

        result.chunk.code.iter().any(|op| matches!(op, OpCode::MapColumnLlm(_)))
    }

    #[test]
    fn test_map_column_regular_function() {
        let source = r#"def double(x: Int) -> Int {
    x * 2
}

let data = [{"val": 1}, {"val": 2}]
map_column(data, "val", "doubled", |x| double(x))"#;

        assert!(has_map_column(source), "Expected MapColumn opcode for regular function");
        assert!(!has_map_column_llm(source), "Should not have MapColumnLlm for regular function");
    }

    #[test]
    fn test_map_column_llm_detection() {
        let source = r#"def summarize(text: String) -> String {
    base_url: "https://api.openai.com/v1"
    model: "gpt-4"
    api_key_env: "OPENAI_API_KEY"
    prompt: "Summarize: ${text}"
}

let docs = [{"content": "doc1"}, {"content": "doc2"}]
map_column(docs, "content", "summary", |t| summarize(t))"#;

        assert!(has_map_column_llm(source), "Expected MapColumnLlm opcode for LLM function");
        assert!(!has_map_column(source), "Should not have MapColumn when LLM path is taken");
    }

    #[test]
    fn test_map_column_pipe_syntax() {
        let source = r#"def process(x: String) -> String {
    x + "!"
}

let data = [{"msg": "hello"}]
data |> map_column("msg", "processed", |m| process(m))"#;

        assert!(has_map_column(source), "Expected MapColumn opcode for pipe syntax");
    }

    // ========================================================================
    // map_row Compiler Tests
    // ========================================================================

    /// Helper to check if compiled code contains MapRow opcode
    fn has_map_row(source: &str) -> bool {
        use crate::vm::bytecode::OpCode;

        let program = parser::parse(source).expect("parse failed");
        let result = Compiler::compile(&program).expect("compile failed");

        result.chunk.code.iter().any(|op| matches!(op, OpCode::MapRow))
    }

    /// Helper to check if compiled code contains MapRowLlm opcode
    fn has_map_row_llm(source: &str) -> bool {
        use crate::vm::bytecode::OpCode;

        let program = parser::parse(source).expect("parse failed");
        let result = Compiler::compile(&program).expect("compile failed");

        result.chunk.code.iter().any(|op| matches!(op, OpCode::MapRowLlm(_, _)))
    }

    #[test]
    fn test_map_row_regular_function() {
        let source = r#"def combine(row: Map<String, String>) -> String {
    row["a"] + row["b"]
}

let data = [{"a": "hello", "b": "world"}]
map_row(data, "combined", |r| combine(r))"#;

        assert!(has_map_row(source), "Expected MapRow opcode for regular function");
        assert!(!has_map_row_llm(source), "Should not have MapRowLlm for regular function");
    }

    #[test]
    fn test_map_row_llm_detection() {
        let source = r#"def analyze(title: String, content: String) -> String {
    base_url: "https://api.openai.com/v1"
    model: "gpt-4"
    api_key_env: "OPENAI_API_KEY"
    prompt: "Analyze: ${title} - ${content}"
}

let docs = [{"title": "t1", "content": "c1"}]
map_row(docs, "analysis", |r| analyze(r["title"], r["content"]))"#;

        assert!(has_map_row_llm(source), "Expected MapRowLlm opcode for LLM function");
        assert!(!has_map_row(source), "Should not have MapRow when LLM path is taken");
    }

    #[test]
    fn test_map_row_pipe_syntax() {
        let source = r#"def format(row: Map<String, String>) -> String {
    "Result: " + row["x"]
}

let data = [{"x": "test"}]
data |> map_row("formatted", |r| format(r))"#;

        assert!(has_map_row(source), "Expected MapRow opcode for pipe syntax");
    }
}
