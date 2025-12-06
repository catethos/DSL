//! Type checker for Lattice
//!
//! Provides static type checking including:
//! - Type inference for let bindings
//! - Function signature validation
//! - Expression type checking
//! - Type compatibility checks

use std::collections::HashMap;

use crate::error::{LatticeError, Result};
use crate::syntax::ast::{
    BinaryOp, Block, Expr, ExprKind, FStringPart, FunctionBody, FunctionDef, Item, LambdaBody,
    Literal, MatchArmBody, PatternKind, PrimitiveType, Program, Stmt, StmtKind, TypeAnnotation,
    TypeExpr, UnaryOp,
};
use crate::types::ir::{Class, Enum, FieldType};

// ============================================================================
// Type Representation
// ============================================================================

/// Represents a type in the Lattice type system
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Primitive types
    String,
    Int,
    Float,
    Bool,
    Null,
    Path,
    /// Named class type
    Class(String),
    /// Named enum type
    Enum(String),
    /// List type
    List(Box<Type>),
    /// Map type
    Map(Box<Type>, Box<Type>),
    /// Result type
    Result(Box<Type>, Box<Type>),
    /// Function type (params -> return)
    Function {
        params: Vec<Type>,
        ret: Box<Type>,
    },
    /// Union type (for optional types, represented as T | Null)
    Union(Vec<Type>),
    /// Unknown type (for type inference)
    Unknown,
    /// Any type (compatible with everything, used for gradual typing)
    Any,
}

impl Type {
    /// Check if this type is optional (nullable)
    pub fn is_optional(&self) -> bool {
        match self {
            Type::Null => true,
            Type::Union(types) => types.iter().any(|t| matches!(t, Type::Null)),
            _ => false,
        }
    }

    /// Make this type optional by wrapping in Union with Null
    pub fn make_optional(self) -> Type {
        if self.is_optional() {
            self
        } else {
            Type::Union(vec![self, Type::Null])
        }
    }

    /// Convert to FieldType (for IR)
    pub fn to_field_type(&self) -> Option<FieldType> {
        match self {
            Type::String => Some(FieldType::String),
            Type::Int => Some(FieldType::Int),
            Type::Float => Some(FieldType::Float),
            Type::Bool => Some(FieldType::Bool),
            Type::Path => Some(FieldType::Path),
            Type::Class(name) => Some(FieldType::Class(name.clone())),
            Type::Enum(name) => Some(FieldType::Enum(name.clone())),
            Type::List(inner) => inner.to_field_type().map(|t| FieldType::List(Box::new(t))),
            Type::Map(k, v) => {
                let k_type = k.to_field_type()?;
                let v_type = v.to_field_type()?;
                Some(FieldType::Map(Box::new(k_type), Box::new(v_type)))
            }
            Type::Union(types) => {
                let field_types: Vec<_> = types.iter().filter_map(|t| t.to_field_type()).collect();
                if field_types.len() == types.len() {
                    Some(FieldType::Union(field_types))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Convert from FieldType
    pub fn from_field_type(ft: &FieldType) -> Type {
        match ft {
            FieldType::String => Type::String,
            FieldType::Int => Type::Int,
            FieldType::Float => Type::Float,
            FieldType::Bool => Type::Bool,
            FieldType::Path => Type::Path,
            FieldType::Class(name) => Type::Class(name.clone()),
            FieldType::Enum(name) => Type::Enum(name.clone()),
            FieldType::List(inner) => Type::List(Box::new(Type::from_field_type(inner))),
            FieldType::Map(k, v) => Type::Map(
                Box::new(Type::from_field_type(k)),
                Box::new(Type::from_field_type(v)),
            ),
            FieldType::Union(types) => {
                Type::Union(types.iter().map(Type::from_field_type).collect())
            }
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::String => write!(f, "String"),
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::Bool => write!(f, "Bool"),
            Type::Null => write!(f, "Null"),
            Type::Path => write!(f, "Path"),
            Type::Class(name) => write!(f, "{}", name),
            Type::Enum(name) => write!(f, "{}", name),
            Type::List(inner) => write!(f, "[{}]", inner),
            Type::Map(k, v) => write!(f, "Map<{}, {}>", k, v),
            Type::Result(ok, err) => write!(f, "Result<{}, {}>", ok, err),
            Type::Function { params, ret } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Union(types) => {
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", t)?;
                }
                Ok(())
            }
            Type::Unknown => write!(f, "?"),
            Type::Any => write!(f, "Any"),
        }
    }
}

// ============================================================================
// Type Context
// ============================================================================

/// Stores type information for type checking
#[derive(Debug, Clone)]
pub struct TypeContext {
    /// Variable bindings (name -> type)
    variables: HashMap<String, Type>,
    /// Class definitions
    classes: HashMap<String, Class>,
    /// Enum definitions
    enums: HashMap<String, Enum>,
    /// Function signatures (name -> (params, return_type))
    functions: HashMap<String, (Vec<(String, Type)>, Type)>,
    /// Parent context for scoped lookups
    parent: Option<Box<TypeContext>>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            classes: HashMap::new(),
            enums: HashMap::new(),
            functions: HashMap::new(),
            parent: None,
        }
    }

    /// Create a child scope
    pub fn child(&self) -> TypeContext {
        TypeContext {
            variables: HashMap::new(),
            classes: HashMap::new(),
            enums: HashMap::new(),
            functions: HashMap::new(),
            parent: Some(Box::new(self.clone())),
        }
    }

    /// Define a variable in the current scope
    pub fn define_var(&mut self, name: String, ty: Type) {
        self.variables.insert(name, ty);
    }

    /// Look up a variable type
    pub fn get_var(&self, name: &str) -> Option<&Type> {
        self.variables
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get_var(name)))
    }

    /// Define a class
    pub fn define_class(&mut self, class: Class) {
        self.classes.insert(class.name.clone(), class);
    }

    /// Look up a class
    pub fn get_class(&self, name: &str) -> Option<&Class> {
        self.classes
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get_class(name)))
    }

    /// Define an enum
    pub fn define_enum(&mut self, enum_def: Enum) {
        self.enums.insert(enum_def.name.clone(), enum_def);
    }

    /// Look up an enum
    pub fn get_enum(&self, name: &str) -> Option<&Enum> {
        self.enums
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get_enum(name)))
    }

    /// Define a function
    pub fn define_function(&mut self, name: String, params: Vec<(String, Type)>, ret: Type) {
        self.functions.insert(name, (params, ret));
    }

    /// Look up a function signature
    pub fn get_function(&self, name: &str) -> Option<&(Vec<(String, Type)>, Type)> {
        self.functions
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get_function(name)))
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Type Checker
// ============================================================================

/// The main type checker
pub struct TypeChecker {
    ctx: TypeContext,
    errors: Vec<TypeError>,
}

/// A type error with location information
#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl TypeError {
    pub fn new(message: String, line: usize, column: usize) -> Self {
        Self {
            message,
            line,
            column,
        }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.message)
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            ctx: TypeContext::new(),
            errors: Vec::new(),
        }
    }

    /// Check a complete program
    pub fn check_program(&mut self, program: &Program) -> Result<()> {
        // First pass: collect all type and function definitions
        for item in &program.items {
            match item {
                Item::TypeDef(td) => {
                    let class = Class {
                        name: td.name.node.clone(),
                        description: None,
                        fields: td
                            .fields
                            .iter()
                            .map(|f| crate::types::ir::Field {
                                name: f.name.node.clone(),
                                field_type: self.type_annotation_to_field_type(&f.ty),
                                optional: f.ty.optional,
                                description: f.description.clone(),
                            })
                            .collect(),
                    };
                    self.ctx.define_class(class);
                }
                Item::EnumDef(ed) => {
                    let enum_def = Enum {
                        name: ed.name.node.clone(),
                        description: None,
                        values: ed.variants.iter().map(|v| v.node.clone()).collect(),
                    };
                    self.ctx.define_enum(enum_def);
                }
                Item::FunctionDef(fd) => {
                    self.register_function(fd);
                }
                Item::LlmConfigDecl(_) => {
                    // LLM config declarations don't need type checking
                    // They are validated at compile time when referenced
                }
                Item::Statement(_) => {}
            }
        }

        // Second pass: check all items
        for item in &program.items {
            match item {
                Item::FunctionDef(fd) => {
                    self.check_function(fd)?;
                }
                Item::Statement(stmt) => {
                    self.check_stmt(stmt)?;
                }
                _ => {}
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            let messages: Vec<String> = self.errors.iter().map(|e| e.to_string()).collect();
            Err(LatticeError::Type(messages.join("\n")))
        }
    }

    /// Register a function's type signature
    fn register_function(&mut self, fd: &FunctionDef) {
        let params: Vec<(String, Type)> = fd
            .params
            .iter()
            .map(|p| (p.name.node.clone(), self.type_annotation_to_type(&p.ty)))
            .collect();

        let ret = fd
            .return_type
            .as_ref()
            .map(|t| self.type_annotation_to_type(t))
            .unwrap_or(Type::Null);

        self.ctx.define_function(fd.name.node.clone(), params, ret);
    }

    /// Check a function definition
    fn check_function(&mut self, fd: &FunctionDef) -> Result<()> {
        // Create a new scope for the function
        let mut fn_ctx = self.ctx.child();

        // Add parameters to scope
        for param in &fd.params {
            let ty = self.type_annotation_to_type(&param.ty);
            fn_ctx.define_var(param.name.node.clone(), ty);
        }

        // Check the function body
        match &fd.body {
            FunctionBody::Block(block) => {
                let old_ctx = std::mem::replace(&mut self.ctx, fn_ctx);
                let body_type = self.check_block(block)?;
                self.ctx = old_ctx;

                // Check return type matches
                if let Some(ret_type) = &fd.return_type {
                    let expected = self.type_annotation_to_type(ret_type);
                    if !self.types_compatible(&body_type, &expected) {
                        self.errors.push(TypeError::new(
                            format!(
                                "Function '{}' should return {} but returns {}",
                                fd.name.node, expected, body_type
                            ),
                            fd.span.line,
                            fd.span.column,
                        ));
                    }
                }
            }
            FunctionBody::LlmConfig(_) => {
                // LLM functions are validated differently
            }
        }

        Ok(())
    }

    /// Check a statement
    fn check_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match &stmt.kind {
            StmtKind::Let { name, ty, value } => {
                let value_type = self.infer_expr(value)?;

                if let Some(type_ann) = ty {
                    let declared_type = self.type_annotation_to_type(type_ann);
                    if !self.types_compatible(&value_type, &declared_type) {
                        self.errors.push(TypeError::new(
                            format!(
                                "Cannot assign {} to variable '{}' of type {}",
                                value_type, name.node, declared_type
                            ),
                            stmt.span.line,
                            stmt.span.column,
                        ));
                    }
                    self.ctx.define_var(name.node.clone(), declared_type);
                } else {
                    // Type inference
                    self.ctx.define_var(name.node.clone(), value_type);
                }
            }
            StmtKind::Assign { target, value } => {
                let value_type = self.infer_expr(value)?;

                if let Some(var_type) = self.ctx.get_var(&target.base.node).cloned() {
                    // For simple assignment, check type compatibility
                    if target.accessors.is_empty()
                        && !self.types_compatible(&value_type, &var_type) {
                            self.errors.push(TypeError::new(
                                format!(
                                    "Cannot assign {} to variable '{}' of type {}",
                                    value_type, target.base.node, var_type
                                ),
                                stmt.span.line,
                                stmt.span.column,
                            ));
                        }
                    // TODO: Check field/index access types
                } else {
                    self.errors.push(TypeError::new(
                        format!("Undefined variable '{}'", target.base.node),
                        stmt.span.line,
                        stmt.span.column,
                    ));
                }
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_type = self.infer_expr(condition)?;
                if !self.types_compatible(&cond_type, &Type::Bool) {
                    self.errors.push(TypeError::new(
                        format!("If condition must be Bool, got {}", cond_type),
                        stmt.span.line,
                        stmt.span.column,
                    ));
                }
                self.check_block(then_branch)?;
                if let Some(else_clause) = else_branch {
                    match else_clause {
                        crate::syntax::ast::ElseClause::ElseIf(else_if) => {
                            self.check_stmt(else_if)?;
                        }
                        crate::syntax::ast::ElseClause::Else(block) => {
                            self.check_block(block)?;
                        }
                    }
                }
            }
            StmtKind::While { condition, body } => {
                let cond_type = self.infer_expr(condition)?;
                if !self.types_compatible(&cond_type, &Type::Bool) {
                    self.errors.push(TypeError::new(
                        format!("While condition must be Bool, got {}", cond_type),
                        stmt.span.line,
                        stmt.span.column,
                    ));
                }
                self.check_block(body)?;
            }
            StmtKind::For { var, iterable, body } => {
                let iter_type = self.infer_expr(iterable)?;

                // Determine element type
                let elem_type = match &iter_type {
                    Type::List(inner) => (**inner).clone(),
                    Type::String => Type::String, // Iterating over string yields strings (chars)
                    _ => {
                        self.errors.push(TypeError::new(
                            format!("Cannot iterate over {}", iter_type),
                            stmt.span.line,
                            stmt.span.column,
                        ));
                        Type::Unknown
                    }
                };

                // Create scope with loop variable
                let mut loop_ctx = self.ctx.child();
                loop_ctx.define_var(var.node.clone(), elem_type);

                let old_ctx = std::mem::replace(&mut self.ctx, loop_ctx);
                self.check_block(body)?;
                self.ctx = old_ctx;
            }
            StmtKind::Return { value } => {
                if let Some(expr) = value {
                    self.infer_expr(expr)?;
                }
                // TODO: Check against function return type
            }
            StmtKind::Expr { expr } => {
                self.infer_expr(expr)?;
            }
        }

        Ok(())
    }

    /// Check a block and return its type (the type of the trailing expression)
    fn check_block(&mut self, block: &Block) -> Result<Type> {
        let block_ctx = self.ctx.child();
        let old_ctx = std::mem::replace(&mut self.ctx, block_ctx);

        for stmt in &block.stmts {
            self.check_stmt(stmt)?;
        }

        let result_type = if let Some(expr) = &block.expr {
            self.infer_expr(expr)?
        } else {
            Type::Null
        };

        self.ctx = old_ctx;
        Ok(result_type)
    }

    /// Infer the type of an expression
    fn infer_expr(&mut self, expr: &Expr) -> Result<Type> {
        match &expr.kind {
            ExprKind::Literal(lit) => Ok(self.literal_type(lit)),
            ExprKind::Var(name) => {
                if let Some(ty) = self.ctx.get_var(name) {
                    Ok(ty.clone())
                } else {
                    self.errors.push(TypeError::new(
                        format!("Undefined variable '{}'", name),
                        expr.span.line,
                        expr.span.column,
                    ));
                    Ok(Type::Unknown)
                }
            }
            ExprKind::EnumVariant { enum_name, variant } => {
                if let Some(enum_def) = self.ctx.get_enum(enum_name) {
                    if enum_def.values.contains(variant) {
                        Ok(Type::Enum(enum_name.clone()))
                    } else {
                        self.errors.push(TypeError::new(
                            format!("Enum '{}' has no variant '{}'", enum_name, variant),
                            expr.span.line,
                            expr.span.column,
                        ));
                        Ok(Type::Unknown)
                    }
                } else {
                    self.errors.push(TypeError::new(
                        format!("Undefined enum '{}'", enum_name),
                        expr.span.line,
                        expr.span.column,
                    ));
                    Ok(Type::Unknown)
                }
            }
            ExprKind::Binary { left, op, right } => {
                let left_type = self.infer_expr(left)?;
                let right_type = self.infer_expr(right)?;
                self.check_binary_op(*op, &left_type, &right_type, expr)
            }
            ExprKind::Unary { op, operand } => {
                let operand_type = self.infer_expr(operand)?;
                self.check_unary_op(*op, &operand_type, expr)
            }
            ExprKind::Field { object, field } => {
                let obj_type = self.infer_expr(object)?;
                self.check_field_access(&obj_type, field, expr)
            }
            ExprKind::Index { object, index } => {
                let obj_type = self.infer_expr(object)?;
                let idx_type = self.infer_expr(index)?;
                self.check_index_access(&obj_type, &idx_type, expr)
            }
            ExprKind::Call { callee, args } => self.check_call(callee, args, expr),
            ExprKind::List(elements) => {
                if elements.is_empty() {
                    Ok(Type::List(Box::new(Type::Unknown)))
                } else {
                    let first_type = self.infer_expr(&elements[0])?;
                    for elem in &elements[1..] {
                        let elem_type = self.infer_expr(elem)?;
                        if !self.types_compatible(&elem_type, &first_type) {
                            self.errors.push(TypeError::new(
                                format!(
                                    "List elements must have consistent types: expected {}, got {}",
                                    first_type, elem_type
                                ),
                                elem.span.line,
                                elem.span.column,
                            ));
                        }
                    }
                    Ok(Type::List(Box::new(first_type)))
                }
            }
            ExprKind::Map(entries) => {
                if entries.is_empty() {
                    Ok(Type::Map(Box::new(Type::String), Box::new(Type::Unknown)))
                } else {
                    let (_, first_val) = &entries[0];
                    let val_type = self.infer_expr(first_val)?;
                    for (_, v) in &entries[1..] {
                        let v_type = self.infer_expr(v)?;
                        if !self.types_compatible(&v_type, &val_type) {
                            self.errors.push(TypeError::new(
                                format!(
                                    "Map values must have consistent types: expected {}, got {}",
                                    val_type, v_type
                                ),
                                v.span.line,
                                v.span.column,
                            ));
                        }
                    }
                    Ok(Type::Map(Box::new(Type::String), Box::new(val_type)))
                }
            }
            ExprKind::Struct { name, fields } => {
                if let Some(class) = self.ctx.get_class(name).cloned() {
                    // Check all required fields are present
                    for class_field in &class.fields {
                        if !class_field.optional
                            && !fields.iter().any(|(n, _)| n == &class_field.name) {
                                self.errors.push(TypeError::new(
                                    format!(
                                        "Missing required field '{}' in struct '{}'",
                                        class_field.name, name
                                    ),
                                    expr.span.line,
                                    expr.span.column,
                                ));
                            }
                    }
                    // Check field types
                    for (field_name, field_val) in fields {
                        let val_type = self.infer_expr(field_val)?;
                        if let Some(class_field) =
                            class.fields.iter().find(|f| &f.name == field_name)
                        {
                            let expected = Type::from_field_type(&class_field.field_type);
                            if !self.types_compatible(&val_type, &expected) {
                                self.errors.push(TypeError::new(
                                    format!(
                                        "Field '{}' expects {}, got {}",
                                        field_name, expected, val_type
                                    ),
                                    field_val.span.line,
                                    field_val.span.column,
                                ));
                            }
                        } else {
                            self.errors.push(TypeError::new(
                                format!("Unknown field '{}' in struct '{}'", field_name, name),
                                expr.span.line,
                                expr.span.column,
                            ));
                        }
                    }
                    Ok(Type::Class(name.clone()))
                } else {
                    self.errors.push(TypeError::new(
                        format!("Undefined type '{}'", name),
                        expr.span.line,
                        expr.span.column,
                    ));
                    Ok(Type::Unknown)
                }
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_type = self.infer_expr(condition)?;
                if !self.types_compatible(&cond_type, &Type::Bool) {
                    self.errors.push(TypeError::new(
                        format!("If condition must be Bool, got {}", cond_type),
                        expr.span.line,
                        expr.span.column,
                    ));
                }

                let then_type = self.check_block(then_branch)?;

                if let Some(else_clause) = else_branch {
                    let else_type = match else_clause {
                        crate::syntax::ast::IfExprElse::ElseIf(else_if) => self.infer_expr(else_if)?,
                        crate::syntax::ast::IfExprElse::Else(block) => self.check_block(block)?,
                    };

                    // Both branches should have compatible types
                    if self.types_compatible(&then_type, &else_type) {
                        Ok(then_type)
                    } else if self.types_compatible(&else_type, &then_type) {
                        Ok(else_type)
                    } else {
                        // Return union type
                        Ok(Type::Union(vec![then_type, else_type]))
                    }
                } else {
                    // No else branch means the expression might return null
                    Ok(then_type.make_optional())
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                let scrutinee_type = self.infer_expr(scrutinee)?;
                let mut result_type: Option<Type> = None;

                for arm in arms {
                    // Check pattern compatibility
                    self.check_pattern(&arm.pattern.kind, &scrutinee_type, &arm.pattern.span)?;

                    // Create scope with pattern bindings
                    let mut arm_ctx = self.ctx.child();
                    self.add_pattern_bindings(&arm.pattern.kind, &scrutinee_type, &mut arm_ctx);

                    let old_ctx = std::mem::replace(&mut self.ctx, arm_ctx);
                    let arm_type = match &arm.body {
                        MatchArmBody::Expr(e) => self.infer_expr(e)?,
                        MatchArmBody::Block(b) => self.check_block(b)?,
                    };
                    self.ctx = old_ctx;

                    if let Some(ref rt) = result_type {
                        if !self.types_compatible(&arm_type, rt) {
                            // Widen to union
                            result_type = Some(Type::Union(vec![rt.clone(), arm_type]));
                        }
                    } else {
                        result_type = Some(arm_type);
                    }
                }

                Ok(result_type.unwrap_or(Type::Null))
            }
            ExprKind::Lambda { params, body } => {
                // Create scope with lambda parameters (inferred as Any for now)
                let mut lambda_ctx = self.ctx.child();
                for param in params {
                    lambda_ctx.define_var(param.clone(), Type::Any);
                }

                let old_ctx = std::mem::replace(&mut self.ctx, lambda_ctx);
                let body_type = match body.as_ref() {
                    LambdaBody::Expr(e) => self.infer_expr(e)?,
                    LambdaBody::Block(b) => self.check_block(b)?,
                };
                self.ctx = old_ctx;

                Ok(Type::Function {
                    params: vec![Type::Any; params.len()],
                    ret: Box::new(body_type),
                })
            }
            ExprKind::Parallel(exprs) => {
                let types: Vec<Type> = exprs
                    .iter()
                    .map(|e| self.infer_expr(e))
                    .collect::<Result<_>>()?;
                Ok(Type::List(Box::new(if types.is_empty() {
                    Type::Unknown
                } else {
                    types[0].clone()
                })))
            }
            ExprKind::ParallelMap { collection, mapper } => {
                let coll_type = self.infer_expr(collection)?;
                let mapper_type = self.infer_expr(mapper)?;

                if let Type::List(elem_type) = &coll_type {
                    if let Type::Function { ret, .. } = mapper_type {
                        Ok(Type::List(ret))
                    } else {
                        self.errors.push(TypeError::new(
                            "parallel_map requires a function as mapper".to_string(),
                            expr.span.line,
                            expr.span.column,
                        ));
                        Ok(Type::List(elem_type.clone()))
                    }
                } else {
                    self.errors.push(TypeError::new(
                        format!("parallel_map requires a list, got {}", coll_type),
                        expr.span.line,
                        expr.span.column,
                    ));
                    Ok(Type::Unknown)
                }
            }
            ExprKind::MapColumn { table, input_col, output_col, mapper } => {
                // Type-check all arguments
                let table_type = self.infer_expr(table)?;
                let _input_col_type = self.infer_expr(input_col)?;
                let _output_col_type = self.infer_expr(output_col)?;
                let _mapper_type = self.infer_expr(mapper)?;

                // map_column returns a list (the modified table)
                if let Type::List(_) = &table_type {
                    Ok(table_type)
                } else {
                    self.errors.push(TypeError::new(
                        format!("map_column requires a table (list), got {}", table_type),
                        expr.span.line,
                        expr.span.column,
                    ));
                    Ok(Type::Unknown)
                }
            }
            ExprKind::MapRow { table, output_col, mapper } => {
                // Type-check all arguments
                let table_type = self.infer_expr(table)?;
                let _output_col_type = self.infer_expr(output_col)?;
                let _mapper_type = self.infer_expr(mapper)?;

                // map_row returns a list (the modified table)
                if let Type::List(_) = &table_type {
                    Ok(table_type)
                } else {
                    self.errors.push(TypeError::new(
                        format!("map_row requires a table (list), got {}", table_type),
                        expr.span.line,
                        expr.span.column,
                    ));
                    Ok(Type::Unknown)
                }
            }
            ExprKind::Explode { table, column, prefix } => {
                // Type-check all arguments
                let table_type = self.infer_expr(table)?;
                let _column_type = self.infer_expr(column)?;
                if let Some(prefix_expr) = prefix {
                    self.infer_expr(prefix_expr)?;
                }

                // explode returns a list (the modified table)
                if let Type::List(_) = &table_type {
                    Ok(table_type)
                } else {
                    self.errors.push(TypeError::new(
                        "explode requires a list as first argument".to_string(),
                        expr.span.line,
                        expr.span.column,
                    ));
                    Ok(Type::Unknown)
                }
            }
            ExprKind::Sql { ty, query: _ } => {
                // SQL returns the specified type or a generic list
                if let Some(type_ann) = ty {
                    Ok(self.type_annotation_to_type(type_ann))
                } else {
                    Ok(Type::List(Box::new(Type::Any)))
                }
            }
            ExprKind::Grouped(inner) => self.infer_expr(inner),
            ExprKind::Block(block) => self.check_block(block),
            ExprKind::FString(parts) => {
                // Type-check all interpolated expressions
                for part in parts {
                    if let FStringPart::Expr(expr) = part {
                        self.infer_expr(expr)?;
                    }
                }
                // F-strings always produce a String
                Ok(Type::String)
            }
            ExprKind::DollarField(field_expr) => {
                // $field represents an implicit lambda: |row| row[field]
                // Type-check the field expression (should be a string key)
                self.infer_expr(field_expr)?;
                // The result type is a function from Row to Value
                // For now, we return Unknown since this is dynamic
                // The actual type depends on context (will be desugared in map_row)
                Ok(Type::Unknown)
            }
        }
    }

    /// Get the type of a literal
    fn literal_type(&self, lit: &Literal) -> Type {
        match lit {
            Literal::Int(_) => Type::Int,
            Literal::Float(_) => Type::Float,
            Literal::String(_) => Type::String,
            Literal::Bool(_) => Type::Bool,
            Literal::Null => Type::Null,
        }
    }

    /// Check binary operation types
    fn check_binary_op(
        &mut self,
        op: BinaryOp,
        left: &Type,
        right: &Type,
        expr: &Expr,
    ) -> Result<Type> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                // Arithmetic: both operands must be numeric (Bool is allowed for Add, treated as 0/1)
                let is_numeric = |t: &Type| matches!(t, Type::Int | Type::Float | Type::Any);
                let is_numeric_or_bool = |t: &Type| matches!(t, Type::Int | Type::Float | Type::Bool | Type::Any);

                // For Add, allow Bool (true=1, false=0)
                if op == BinaryOp::Add {
                    if !is_numeric_or_bool(left) || !is_numeric_or_bool(right) {
                        self.errors.push(TypeError::new(
                            format!(
                                "Arithmetic operation requires numeric types, got {} {} {}",
                                left,
                                op.as_str(),
                                right
                            ),
                            expr.span.line,
                            expr.span.column,
                        ));
                    }
                } else if !is_numeric(left) || !is_numeric(right) {
                    self.errors.push(TypeError::new(
                        format!(
                            "Arithmetic operation requires numeric types, got {} {} {}",
                            left,
                            op.as_str(),
                            right
                        ),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                // String concatenation with +
                if op == BinaryOp::Add
                    && matches!(left, Type::String)
                    && matches!(right, Type::String)
                {
                    return Ok(Type::String);
                }
                // If either is Float, result is Float
                if matches!(left, Type::Float) || matches!(right, Type::Float) {
                    Ok(Type::Float)
                } else {
                    Ok(Type::Int)
                }
            }
            BinaryOp::Eq | BinaryOp::Ne => {
                // Equality: any types can be compared
                Ok(Type::Bool)
            }
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                // Comparison: both must be comparable (numeric or string)
                let is_comparable =
                    |t: &Type| matches!(t, Type::Int | Type::Float | Type::String | Type::Any);
                if !is_comparable(left) || !is_comparable(right) {
                    self.errors.push(TypeError::new(
                        format!(
                            "Comparison operation requires comparable types, got {} {} {}",
                            left,
                            op.as_str(),
                            right
                        ),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                Ok(Type::Bool)
            }
            BinaryOp::And | BinaryOp::Or => {
                // Logical: both must be boolean
                if !matches!(left, Type::Bool | Type::Any) {
                    self.errors.push(TypeError::new(
                        format!("Logical '{}' requires Bool, got {}", op.as_str(), left),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                if !matches!(right, Type::Bool | Type::Any) {
                    self.errors.push(TypeError::new(
                        format!("Logical '{}' requires Bool, got {}", op.as_str(), right),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                Ok(Type::Bool)
            }
        }
    }

    /// Check unary operation types
    fn check_unary_op(&mut self, op: UnaryOp, operand: &Type, expr: &Expr) -> Result<Type> {
        match op {
            UnaryOp::Neg => {
                if !matches!(operand, Type::Int | Type::Float | Type::Any) {
                    self.errors.push(TypeError::new(
                        format!("Negation requires numeric type, got {}", operand),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                Ok(operand.clone())
            }
            UnaryOp::Not => {
                if !matches!(operand, Type::Bool | Type::Any) {
                    self.errors.push(TypeError::new(
                        format!("Logical not requires Bool, got {}", operand),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                Ok(Type::Bool)
            }
        }
    }

    /// Check field access
    fn check_field_access(&mut self, obj_type: &Type, field: &str, expr: &Expr) -> Result<Type> {
        match obj_type {
            Type::Class(name) => {
                if let Some(class) = self.ctx.get_class(name).cloned() {
                    if let Some(f) = class.fields.iter().find(|f| f.name == field) {
                        let ty = Type::from_field_type(&f.field_type);
                        if f.optional {
                            Ok(ty.make_optional())
                        } else {
                            Ok(ty)
                        }
                    } else {
                        self.errors.push(TypeError::new(
                            format!("Type '{}' has no field '{}'", name, field),
                            expr.span.line,
                            expr.span.column,
                        ));
                        Ok(Type::Unknown)
                    }
                } else {
                    self.errors.push(TypeError::new(
                        format!("Unknown type '{}'", name),
                        expr.span.line,
                        expr.span.column,
                    ));
                    Ok(Type::Unknown)
                }
            }
            Type::Map(_, v) => {
                // Map field access returns the value type
                Ok((**v).clone().make_optional())
            }
            Type::Any => Ok(Type::Any),
            _ => {
                self.errors.push(TypeError::new(
                    format!("Cannot access field '{}' on type {}", field, obj_type),
                    expr.span.line,
                    expr.span.column,
                ));
                Ok(Type::Unknown)
            }
        }
    }

    /// Check index access
    fn check_index_access(
        &mut self,
        obj_type: &Type,
        idx_type: &Type,
        expr: &Expr,
    ) -> Result<Type> {
        match obj_type {
            Type::List(elem) => {
                if !matches!(idx_type, Type::Int | Type::Any) {
                    self.errors.push(TypeError::new(
                        format!("List index must be Int, got {}", idx_type),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                Ok((**elem).clone())
            }
            Type::Map(k, v) => {
                if !self.types_compatible(idx_type, k) {
                    self.errors.push(TypeError::new(
                        format!("Map key must be {}, got {}", k, idx_type),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                Ok((**v).clone().make_optional())
            }
            Type::String => {
                if !matches!(idx_type, Type::Int | Type::Any) {
                    self.errors.push(TypeError::new(
                        format!("String index must be Int, got {}", idx_type),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                Ok(Type::String)
            }
            Type::Any => Ok(Type::Any),
            _ => {
                self.errors.push(TypeError::new(
                    format!("Cannot index type {}", obj_type),
                    expr.span.line,
                    expr.span.column,
                ));
                Ok(Type::Unknown)
            }
        }
    }

    /// Check function call
    fn check_call(&mut self, callee: &Expr, args: &[Expr], expr: &Expr) -> Result<Type> {
        // Check if it's a named function call
        if let ExprKind::Var(name) = &callee.kind {
            // Check built-in functions
            if let Some(ret_type) = self.check_builtin_call(name, args, expr)? {
                return Ok(ret_type);
            }

            // Check user-defined functions
            if let Some((params, ret)) = self.ctx.get_function(name).cloned() {
                // Check argument count
                if args.len() != params.len() {
                    self.errors.push(TypeError::new(
                        format!(
                            "Function '{}' expects {} arguments, got {}",
                            name,
                            params.len(),
                            args.len()
                        ),
                        expr.span.line,
                        expr.span.column,
                    ));
                }

                // Check argument types
                for (i, (arg, (_, param_type))) in
                    args.iter().zip(params.iter()).enumerate()
                {
                    let arg_type = self.infer_expr(arg)?;
                    if !self.types_compatible(&arg_type, param_type) {
                        self.errors.push(TypeError::new(
                            format!(
                                "Argument {} of '{}' expects {}, got {}",
                                i + 1,
                                name,
                                param_type,
                                arg_type
                            ),
                            arg.span.line,
                            arg.span.column,
                        ));
                    }
                }

                return Ok(ret);
            }
        }

        // Check if callee is a function type
        let callee_type = self.infer_expr(callee)?;
        if let Type::Function { params, ret } = callee_type {
            if args.len() != params.len() {
                self.errors.push(TypeError::new(
                    format!(
                        "Function expects {} arguments, got {}",
                        params.len(),
                        args.len()
                    ),
                    expr.span.line,
                    expr.span.column,
                ));
            }
            return Ok(*ret);
        }

        self.errors.push(TypeError::new(
            format!("Cannot call non-function type {}", callee_type),
            expr.span.line,
            expr.span.column,
        ));
        Ok(Type::Unknown)
    }

    /// Check built-in function calls
    fn check_builtin_call(
        &mut self,
        name: &str,
        args: &[Expr],
        expr: &Expr,
    ) -> Result<Option<Type>> {
        match name {
            "print" | "println" => {
                // Print accepts any type
                for arg in args {
                    self.infer_expr(arg)?;
                }
                Ok(Some(Type::Null))
            }
            "len" => {
                if args.len() != 1 {
                    self.errors.push(TypeError::new(
                        format!("len() expects 1 argument, got {}", args.len()),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                if !args.is_empty() {
                    let arg_type = self.infer_expr(&args[0])?;
                    if !matches!(
                        arg_type,
                        Type::String | Type::List(_) | Type::Map(_, _) | Type::Any
                    ) {
                        self.errors.push(TypeError::new(
                            format!("len() expects String, List, or Map, got {}", arg_type),
                            expr.span.line,
                            expr.span.column,
                        ));
                    }
                }
                Ok(Some(Type::Int))
            }
            "push" => {
                if args.len() != 2 {
                    self.errors.push(TypeError::new(
                        format!("push() expects 2 arguments, got {}", args.len()),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                if !args.is_empty() {
                    let list_type = self.infer_expr(&args[0])?;
                    if !matches!(list_type, Type::List(_) | Type::Any) {
                        self.errors.push(TypeError::new(
                            format!("push() expects a List, got {}", list_type),
                            expr.span.line,
                            expr.span.column,
                        ));
                    }
                }
                if args.len() >= 2 {
                    self.infer_expr(&args[1])?;
                }
                Ok(Some(Type::Null))
            }
            "pop" => {
                if args.len() != 1 {
                    self.errors.push(TypeError::new(
                        format!("pop() expects 1 argument, got {}", args.len()),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                if !args.is_empty() {
                    let list_type = self.infer_expr(&args[0])?;
                    if let Type::List(elem) = list_type {
                        return Ok(Some((*elem).make_optional()));
                    } else if !matches!(list_type, Type::Any) {
                        self.errors.push(TypeError::new(
                            format!("pop() expects a List, got {}", list_type),
                            expr.span.line,
                            expr.span.column,
                        ));
                    }
                }
                Ok(Some(Type::Any))
            }
            "keys" => {
                if args.len() != 1 {
                    self.errors.push(TypeError::new(
                        format!("keys() expects 1 argument, got {}", args.len()),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                if !args.is_empty() {
                    let map_type = self.infer_expr(&args[0])?;
                    if let Type::Map(k, _) = map_type {
                        return Ok(Some(Type::List(k)));
                    } else if !matches!(map_type, Type::Any) {
                        self.errors.push(TypeError::new(
                            format!("keys() expects a Map, got {}", map_type),
                            expr.span.line,
                            expr.span.column,
                        ));
                    }
                }
                Ok(Some(Type::List(Box::new(Type::String))))
            }
            "values" => {
                if args.len() != 1 {
                    self.errors.push(TypeError::new(
                        format!("values() expects 1 argument, got {}", args.len()),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                if !args.is_empty() {
                    let map_type = self.infer_expr(&args[0])?;
                    if let Type::Map(_, v) = map_type {
                        return Ok(Some(Type::List(v)));
                    } else if !matches!(map_type, Type::Any) {
                        self.errors.push(TypeError::new(
                            format!("values() expects a Map, got {}", map_type),
                            expr.span.line,
                            expr.span.column,
                        ));
                    }
                }
                Ok(Some(Type::List(Box::new(Type::Any))))
            }
            "int" => {
                if args.len() != 1 {
                    self.errors.push(TypeError::new(
                        format!("int() expects 1 argument, got {}", args.len()),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                if !args.is_empty() {
                    self.infer_expr(&args[0])?;
                }
                Ok(Some(Type::Int))
            }
            "float" => {
                if args.len() != 1 {
                    self.errors.push(TypeError::new(
                        format!("float() expects 1 argument, got {}", args.len()),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                if !args.is_empty() {
                    self.infer_expr(&args[0])?;
                }
                Ok(Some(Type::Float))
            }
            "str" => {
                if args.len() != 1 {
                    self.errors.push(TypeError::new(
                        format!("str() expects 1 argument, got {}", args.len()),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                if !args.is_empty() {
                    self.infer_expr(&args[0])?;
                }
                Ok(Some(Type::String))
            }
            "bool" => {
                if args.len() != 1 {
                    self.errors.push(TypeError::new(
                        format!("bool() expects 1 argument, got {}", args.len()),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                if !args.is_empty() {
                    self.infer_expr(&args[0])?;
                }
                Ok(Some(Type::Bool))
            }
            "range" => {
                if args.is_empty() || args.len() > 3 {
                    self.errors.push(TypeError::new(
                        format!("range() expects 1-3 arguments, got {}", args.len()),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                for arg in args {
                    let arg_type = self.infer_expr(arg)?;
                    if !matches!(arg_type, Type::Int | Type::Any) {
                        self.errors.push(TypeError::new(
                            format!("range() expects Int arguments, got {}", arg_type),
                            arg.span.line,
                            arg.span.column,
                        ));
                    }
                }
                Ok(Some(Type::List(Box::new(Type::Int))))
            }
            "map" | "filter" => {
                if args.len() != 2 {
                    self.errors.push(TypeError::new(
                        format!("{}() expects 2 arguments, got {}", name, args.len()),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                if !args.is_empty() {
                    let list_type = self.infer_expr(&args[0])?;
                    if !matches!(list_type, Type::List(_) | Type::Any) {
                        self.errors.push(TypeError::new(
                            format!("{}() expects a List as first argument, got {}", name, list_type),
                            expr.span.line,
                            expr.span.column,
                        ));
                    }
                    if args.len() >= 2 {
                        let fn_type = self.infer_expr(&args[1])?;
                        if name == "map" {
                            if let Type::Function { ret, .. } = fn_type {
                                return Ok(Some(Type::List(ret)));
                            }
                        }
                    }
                }
                if name == "filter" {
                    if let Some(arg) = args.first() {
                        return Ok(Some(self.infer_expr(arg)?));
                    }
                }
                Ok(Some(Type::List(Box::new(Type::Any))))
            }
            "reduce" => {
                if args.len() != 3 {
                    self.errors.push(TypeError::new(
                        format!("reduce() expects 3 arguments, got {}", args.len()),
                        expr.span.line,
                        expr.span.column,
                    ));
                }
                if args.len() >= 2 {
                    return Ok(Some(self.infer_expr(&args[1])?));
                }
                Ok(Some(Type::Any))
            }
            _ => Ok(None),
        }
    }

    /// Check pattern against scrutinee type
    fn check_pattern(
        &mut self,
        pattern: &PatternKind,
        scrutinee_type: &Type,
        span: &crate::syntax::ast::Span,
    ) -> Result<()> {
        match pattern {
            PatternKind::Result { is_ok, .. } => {
                if !matches!(scrutinee_type, Type::Result(_, _) | Type::Any) {
                    self.errors.push(TypeError::new(
                        format!(
                            "Cannot match {} pattern against {}",
                            if *is_ok { "Ok" } else { "Err" },
                            scrutinee_type
                        ),
                        span.line,
                        span.column,
                    ));
                }
            }
            PatternKind::Enum { enum_name, variant } => {
                if let Some(enum_def) = self.ctx.get_enum(enum_name) {
                    if !enum_def.values.contains(variant) {
                        self.errors.push(TypeError::new(
                            format!("Enum '{}' has no variant '{}'", enum_name, variant),
                            span.line,
                            span.column,
                        ));
                    }
                } else {
                    self.errors.push(TypeError::new(
                        format!("Unknown enum '{}'", enum_name),
                        span.line,
                        span.column,
                    ));
                }
            }
            PatternKind::Literal(lit) => {
                let lit_type = self.literal_type(lit);
                if !self.types_compatible(&lit_type, scrutinee_type) {
                    self.errors.push(TypeError::new(
                        format!(
                            "Cannot match {} pattern against {}",
                            lit_type, scrutinee_type
                        ),
                        span.line,
                        span.column,
                    ));
                }
            }
            PatternKind::Wildcard | PatternKind::Binding(_) => {
                // These match anything
            }
        }
        Ok(())
    }

    /// Add pattern bindings to context
    fn add_pattern_bindings(
        &self,
        pattern: &PatternKind,
        scrutinee_type: &Type,
        ctx: &mut TypeContext,
    ) {
        match pattern {
            PatternKind::Result { is_ok, binding } => {
                let bound_type = if let Type::Result(ok, err) = scrutinee_type {
                    if *is_ok {
                        (**ok).clone()
                    } else {
                        (**err).clone()
                    }
                } else {
                    Type::Any
                };
                ctx.define_var(binding.clone(), bound_type);
            }
            PatternKind::Binding(name) => {
                ctx.define_var(name.clone(), scrutinee_type.clone());
            }
            _ => {}
        }
    }

    /// Check if two types are compatible
    fn types_compatible(&self, actual: &Type, expected: &Type) -> bool {
        types_compatible(actual, expected)
    }

    /// Convert a type annotation to a Type
    fn type_annotation_to_type(&self, ann: &TypeAnnotation) -> Type {
        let base = self.type_expr_to_type(&ann.ty);
        if ann.optional {
            base.make_optional()
        } else {
            base
        }
    }

    /// Convert a type expression to a Type
    fn type_expr_to_type(&self, expr: &TypeExpr) -> Type {
        match expr {
            TypeExpr::Primitive(p) => match p {
                PrimitiveType::String => Type::String,
                PrimitiveType::Int => Type::Int,
                PrimitiveType::Float => Type::Float,
                PrimitiveType::Bool => Type::Bool,
                PrimitiveType::Null => Type::Null,
                PrimitiveType::Path => Type::Path,
            },
            TypeExpr::Named(name) => {
                if self.ctx.get_class(name).is_some() {
                    Type::Class(name.clone())
                } else if self.ctx.get_enum(name).is_some() {
                    Type::Enum(name.clone())
                } else {
                    // Unknown type, will be caught later
                    Type::Class(name.clone())
                }
            }
            TypeExpr::List(inner) => Type::List(Box::new(self.type_annotation_to_type(inner))),
            TypeExpr::Map(k, v) => Type::Map(
                Box::new(self.type_annotation_to_type(k)),
                Box::new(self.type_annotation_to_type(v)),
            ),
            TypeExpr::Result(ok, err) => Type::Result(
                Box::new(self.type_annotation_to_type(ok)),
                Box::new(self.type_annotation_to_type(err)),
            ),
        }
    }

    /// Convert a type annotation to a FieldType
    fn type_annotation_to_field_type(&self, ann: &TypeAnnotation) -> FieldType {
        self.type_expr_to_field_type(&ann.ty)
    }

    /// Convert a type expression to a FieldType
    fn type_expr_to_field_type(&self, expr: &TypeExpr) -> FieldType {
        match expr {
            TypeExpr::Primitive(p) => match p {
                PrimitiveType::String => FieldType::String,
                PrimitiveType::Int => FieldType::Int,
                PrimitiveType::Float => FieldType::Float,
                PrimitiveType::Bool => FieldType::Bool,
                PrimitiveType::Null => FieldType::String, // Null maps to String for IR
                PrimitiveType::Path => FieldType::Path,
            },
            TypeExpr::Named(name) => {
                if self.ctx.get_enum(name).is_some() {
                    FieldType::Enum(name.clone())
                } else {
                    FieldType::Class(name.clone())
                }
            }
            TypeExpr::List(inner) => {
                FieldType::List(Box::new(self.type_annotation_to_field_type(inner)))
            }
            TypeExpr::Map(k, v) => FieldType::Map(
                Box::new(self.type_annotation_to_field_type(k)),
                Box::new(self.type_annotation_to_field_type(v)),
            ),
            TypeExpr::Result(ok, err) => {
                // Result maps to Union in FieldType
                FieldType::Union(vec![
                    self.type_annotation_to_field_type(ok),
                    self.type_annotation_to_field_type(err),
                ])
            }
        }
    }

    /// Get collected errors
    pub fn errors(&self) -> &[TypeError] {
        &self.errors
    }
}

/// Check if two types are compatible (standalone function to avoid only_used_in_recursion warning)
fn types_compatible(actual: &Type, expected: &Type) -> bool {
    // Any is compatible with everything
    if matches!(actual, Type::Any) || matches!(expected, Type::Any) {
        return true;
    }

    // Unknown is compatible with everything (for inference)
    if matches!(actual, Type::Unknown) || matches!(expected, Type::Unknown) {
        return true;
    }

    // Null is compatible with optional types
    if matches!(actual, Type::Null) && expected.is_optional() {
        return true;
    }

    match (actual, expected) {
        // Same types
        (Type::String, Type::String)
        | (Type::Int, Type::Int)
        | (Type::Float, Type::Float)
        | (Type::Bool, Type::Bool)
        | (Type::Null, Type::Null)
        | (Type::Path, Type::Path) => true,

        // Int is compatible with Float (numeric widening)
        (Type::Int, Type::Float) => true,

        // Named types must match
        (Type::Class(a), Type::Class(b)) => a == b,
        (Type::Enum(a), Type::Enum(b)) => a == b,

        // List types
        (Type::List(a), Type::List(b)) => types_compatible(a, b),

        // Map types
        (Type::Map(ak, av), Type::Map(bk, bv)) => {
            types_compatible(ak, bk) && types_compatible(av, bv)
        }

        // Result types
        (Type::Result(ao, ae), Type::Result(bo, be)) => {
            types_compatible(ao, bo) && types_compatible(ae, be)
        }

        // Function types
        (
            Type::Function {
                params: ap,
                ret: ar,
            },
            Type::Function {
                params: bp,
                ret: br,
            },
        ) => {
            ap.len() == bp.len()
                && ap
                    .iter()
                    .zip(bp.iter())
                    .all(|(a, b)| types_compatible(b, a)) // contravariant params
                && types_compatible(ar, br)
        }

        // Union types
        (_, Type::Union(types)) => types.iter().any(|t| types_compatible(actual, t)),
        (Type::Union(types), _) => types.iter().all(|t| types_compatible(t, expected)),

        _ => false,
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_compatibility() {
        let checker = TypeChecker::new();

        // Basic types
        assert!(checker.types_compatible(&Type::Int, &Type::Int));
        assert!(checker.types_compatible(&Type::String, &Type::String));
        assert!(!checker.types_compatible(&Type::Int, &Type::String));

        // Numeric widening
        assert!(checker.types_compatible(&Type::Int, &Type::Float));
        assert!(!checker.types_compatible(&Type::Float, &Type::Int));

        // Any compatibility
        assert!(checker.types_compatible(&Type::Any, &Type::String));
        assert!(checker.types_compatible(&Type::Int, &Type::Any));

        // Optional types
        assert!(checker.types_compatible(&Type::Null, &Type::String.make_optional()));
        assert!(checker.types_compatible(
            &Type::String,
            &Type::String.make_optional()
        ));
    }

    #[test]
    fn test_list_types() {
        let checker = TypeChecker::new();

        let int_list = Type::List(Box::new(Type::Int));
        let string_list = Type::List(Box::new(Type::String));

        assert!(checker.types_compatible(&int_list, &int_list));
        assert!(!checker.types_compatible(&int_list, &string_list));
    }
}
