#[derive(Debug, Clone, PartialEq)]
pub enum RustItem {
    Use(String),
    Struct(RustStruct),
    Enum(RustEnum),
    Impl(RustImpl),
    Function(RustFunction),
    Mod(String, Vec<RustItem>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustStruct {
    pub name: String,
    pub derives: Vec<String>,
    pub fields: Vec<RustField>,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustField {
    pub name: String,
    pub ty: RustType,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustEnum {
    pub name: String,
    pub derives: Vec<String>,
    pub variants: Vec<RustVariant>,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustVariant {
    pub name: String,
    pub fields: Option<Vec<RustField>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustImpl {
    pub type_name: String,
    pub functions: Vec<RustFunction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustFunction {
    pub name: String,
    pub params: Vec<RustParam>,
    pub return_type: RustType,
    pub is_async: bool,
    pub is_public: bool,
    pub body: Vec<RustStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustParam {
    pub name: String,
    pub ty: RustType,
    pub is_mut: bool,
    pub is_ref: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustType {
    Unit,
    Named(String),
    Generic(String, Vec<RustType>),
    Reference(bool, Box<RustType>), // (is_mut, inner)
    Tuple(Vec<RustType>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustStmt {
    Let {
        name: String,
        ty: Option<RustType>,
        value: RustExpr,
        is_mut: bool,
    },
    Expr(RustExpr),
    Return(Option<RustExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustExpr {
    Literal(RustLiteral),
    Variable(String),
    Call {
        func: Box<RustExpr>,
        args: Vec<RustExpr>,
    },
    MethodCall {
        receiver: Box<RustExpr>,
        method: String,
        args: Vec<RustExpr>,
    },
    FieldAccess {
        base: Box<RustExpr>,
        field: String,
    },
    IndexAccess {
        base: Box<RustExpr>,
        index: Box<RustExpr>,
    },
    BinaryOp {
        left: Box<RustExpr>,
        op: String,
        right: Box<RustExpr>,
    },
    UnaryOp {
        op: String,
        expr: Box<RustExpr>,
    },
    Block(Vec<RustStmt>, Option<Box<RustExpr>>),
    If {
        condition: Box<RustExpr>,
        then_block: Vec<RustStmt>,
        else_block: Option<Vec<RustStmt>>,
    },
    Match {
        expr: Box<RustExpr>,
        arms: Vec<RustMatchArm>,
    },
    Loop {
        body: Vec<RustStmt>,
    },
    While {
        condition: Box<RustExpr>,
        body: Vec<RustStmt>,
    },
    For {
        var: String,
        iter: Box<RustExpr>,
        body: Vec<RustStmt>,
    },
    Break(Option<Box<RustExpr>>),
    Continue,
    Try(Box<RustExpr>),
    Await(Box<RustExpr>),
    Macro {
        name: String,
        args: Vec<RustExpr>,
    },
    Vec(Vec<RustExpr>),
    Tuple(Vec<RustExpr>),
    Struct {
        name: String,
        fields: Vec<(String, RustExpr)>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RustMatchArm {
    pub pattern: RustPattern,
    pub guard: Option<RustExpr>,
    pub body: RustExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustPattern {
    Wildcard,
    Literal(RustLiteral),
    Binding(String),
    Struct {
        name: String,
        fields: Vec<(String, RustPattern)>,
    },
    Tuple(Vec<RustPattern>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RustLiteral {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Unit,
}

impl RustItem {
    pub fn pretty_print(&self, indent: usize) -> String {
        match self {
            RustItem::Use(path) => format!("{}use {};", "    ".repeat(indent), path),
            RustItem::Struct(s) => s.pretty_print(indent),
            RustItem::Enum(e) => e.pretty_print(indent),
            RustItem::Impl(i) => i.pretty_print(indent),
            RustItem::Function(f) => f.pretty_print(indent, false),
            RustItem::Mod(name, items) => {
                let ind = "    ".repeat(indent);
                let mut result = format!("{}mod {} {{\n", ind, name);
                for item in items {
                    result.push_str(&item.pretty_print(indent + 1));
                    result.push('\n');
                }
                result.push_str(&format!("{}}}", ind));
                result
            }
        }
    }
}

impl RustStruct {
    pub fn pretty_print(&self, indent: usize) -> String {
        let ind = "    ".repeat(indent);
        let pub_str = if self.is_public { "pub " } else { "" };

        let mut result = String::new();

        if !self.derives.is_empty() {
            result.push_str(&format!("{}#[derive({})]\n", ind, self.derives.join(", ")));
        }

        result.push_str(&format!("{}{}struct {} {{\n", ind, pub_str, self.name));

        for field in &self.fields {
            let field_pub = if field.is_public { "pub " } else { "" };
            result.push_str(&format!(
                "{}    {}{}: {},\n",
                ind,
                field_pub,
                field.name,
                field.ty.to_string()
            ));
        }

        result.push_str(&format!("{}}}", ind));
        result
    }
}

impl RustEnum {
    pub fn pretty_print(&self, indent: usize) -> String {
        let ind = "    ".repeat(indent);
        let pub_str = if self.is_public { "pub " } else { "" };

        let mut result = String::new();

        if !self.derives.is_empty() {
            result.push_str(&format!("{}#[derive({})]\n", ind, self.derives.join(", ")));
        }

        result.push_str(&format!("{}{}enum {} {{\n", ind, pub_str, self.name));

        for variant in &self.variants {
            result.push_str(&format!("{}    {}", ind, variant.name));
            if let Some(fields) = &variant.fields {
                result.push_str(" {\n");
                for field in fields {
                    let field_pub = if field.is_public { "pub " } else { "" };
                    result.push_str(&format!(
                        "{}        {}{}: {},\n",
                        ind,
                        field_pub,
                        field.name,
                        field.ty.to_string()
                    ));
                }
                result.push_str(&format!("{}    }}", ind));
            }
            result.push_str(",\n");
        }

        result.push_str(&format!("{}}}", ind));
        result
    }
}

impl RustImpl {
    pub fn pretty_print(&self, indent: usize) -> String {
        let ind = "    ".repeat(indent);
        let mut result = format!("{}impl {} {{\n", ind, self.type_name);

        for func in &self.functions {
            result.push_str(&func.pretty_print(indent + 1, true));
            result.push('\n');
        }

        result.push_str(&format!("{}}}", ind));
        result
    }
}

impl RustFunction {
    pub fn pretty_print(&self, indent: usize, in_impl: bool) -> String {
        let ind = "    ".repeat(indent);
        let pub_str = if !in_impl && self.is_public { "pub " } else { "" };
        let async_str = if self.is_async { "async " } else { "" };

        let params = self
            .params
            .iter()
            .map(|p| {
                let mut_str = if p.is_mut { "mut " } else { "" };
                let ref_str = if p.is_ref { "&" } else { "" };
                format!("{}{}{}: {}", ref_str, mut_str, p.name, p.ty.to_string())
            })
            .collect::<Vec<_>>()
            .join(", ");

        let return_str = match &self.return_type {
            RustType::Unit => String::new(),
            ty => format!(" -> {}", ty.to_string()),
        };

        let mut result = format!(
            "{}{}{}fn {}({}){} {{\n",
            ind, pub_str, async_str, self.name, params, return_str
        );

        for stmt in &self.body {
            result.push_str(&stmt.pretty_print(indent + 1));
            result.push('\n');
        }

        result.push_str(&format!("{}}}", ind));
        result
    }
}

impl RustStmt {
    pub fn pretty_print(&self, indent: usize) -> String {
        let ind = "    ".repeat(indent);
        match self {
            RustStmt::Let { name, ty, value, is_mut } => {
                let mut_str = if *is_mut { "mut " } else { "" };
                let ty_str = ty.as_ref().map(|t| format!(": {}", t.to_string())).unwrap_or_default();
                format!("{}let {}{}{} = {};", ind, mut_str, name, ty_str, value.to_string())
            }
            RustStmt::Expr(expr) => {
                format!("{}{};", ind, expr.to_string())
            }
            RustStmt::Return(expr) => {
                if let Some(e) = expr {
                    format!("{}return {};", ind, e.to_string())
                } else {
                    format!("{}return;", ind)
                }
            }
        }
    }
}

impl RustExpr {
    pub fn to_string(&self) -> String {
        match self {
            RustExpr::Literal(lit) => lit.to_string(),
            RustExpr::Variable(name) => name.clone(),
            RustExpr::Call { func, args } => {
                let args_str = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
                format!("{}({})", func.to_string(), args_str)
            }
            RustExpr::MethodCall { receiver, method, args } => {
                let args_str = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
                format!("{}.{}({})", receiver.to_string(), method, args_str)
            }
            RustExpr::FieldAccess { base, field } => {
                format!("{}.{}", base.to_string(), field)
            }
            RustExpr::IndexAccess { base, index } => {
                format!("{}[{}]", base.to_string(), index.to_string())
            }
            RustExpr::BinaryOp { left, op, right } => {
                format!("{} {} {}", left.to_string(), op, right.to_string())
            }
            RustExpr::UnaryOp { op, expr } => {
                format!("{}{}", op, expr.to_string())
            }
            RustExpr::Block(stmts, expr) => {
                let mut result = "{\n".to_string();
                for stmt in stmts {
                    result.push_str(&stmt.pretty_print(1));
                    result.push('\n');
                }
                if let Some(e) = expr {
                    result.push_str(&format!("    {}\n", e.to_string()));
                }
                result.push('}');
                result
            }
            RustExpr::If { condition, then_block, else_block } => {
                let mut result = format!("if {} {{\n", condition.to_string());
                for stmt in then_block {
                    result.push_str(&stmt.pretty_print(1));
                    result.push('\n');
                }
                result.push('}');
                if let Some(else_stmts) = else_block {
                    result.push_str(" else {\n");
                    for stmt in else_stmts {
                        result.push_str(&stmt.pretty_print(1));
                        result.push('\n');
                    }
                    result.push('}');
                }
                result
            }
            RustExpr::Match { expr, arms } => {
                let mut result = format!("match {} {{\n", expr.to_string());
                for arm in arms {
                    result.push_str(&format!("    {} => {},\n", arm.pattern.to_string(), arm.body.to_string()));
                }
                result.push('}');
                result
            }
            RustExpr::Loop { body } => {
                let mut result = "loop {\n".to_string();
                for stmt in body {
                    result.push_str(&stmt.pretty_print(1));
                    result.push('\n');
                }
                result.push('}');
                result
            }
            RustExpr::While { condition, body } => {
                let mut result = format!("while {} {{\n", condition.to_string());
                for stmt in body {
                    result.push_str(&stmt.pretty_print(1));
                    result.push('\n');
                }
                result.push('}');
                result
            }
            RustExpr::For { var, iter, body } => {
                let mut result = format!("for {} in {} {{\n", var, iter.to_string());
                for stmt in body {
                    result.push_str(&stmt.pretty_print(1));
                    result.push('\n');
                }
                result.push('}');
                result
            }
            RustExpr::Break(expr) => {
                if let Some(e) = expr {
                    format!("break {}", e.to_string())
                } else {
                    "break".to_string()
                }
            }
            RustExpr::Continue => "continue".to_string(),
            RustExpr::Try(expr) => format!("{}?", expr.to_string()),
            RustExpr::Await(expr) => format!("{}.await", expr.to_string()),
            RustExpr::Macro { name, args } => {
                let args_str = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
                format!("{}!({})", name, args_str)
            }
            RustExpr::Vec(items) => {
                let items_str = items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
                format!("vec![{}]", items_str)
            }
            RustExpr::Tuple(items) => {
                let items_str = items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
                format!("({})", items_str)
            }
            RustExpr::Struct { name, fields } => {
                let fields_str = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {{ {} }}", name, fields_str)
            }
        }
    }
}

impl RustType {
    pub fn to_string(&self) -> String {
        match self {
            RustType::Unit => "()".to_string(),
            RustType::Named(name) => name.clone(),
            RustType::Generic(name, args) => {
                let args_str = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
                format!("{}<{}>", name, args_str)
            }
            RustType::Reference(is_mut, inner) => {
                let mut_str = if *is_mut { "mut " } else { "" };
                format!("&{}{}", mut_str, inner.to_string())
            }
            RustType::Tuple(types) => {
                let types_str = types.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ");
                format!("({})", types_str)
            }
        }
    }
}

impl RustPattern {
    pub fn to_string(&self) -> String {
        match self {
            RustPattern::Wildcard => "_".to_string(),
            RustPattern::Literal(lit) => lit.to_string(),
            RustPattern::Binding(name) => name.clone(),
            RustPattern::Struct { name, fields } => {
                let fields_str = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {{ {} }}", name, fields_str)
            }
            RustPattern::Tuple(patterns) => {
                let patterns_str = patterns.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ");
                format!("({})", patterns_str)
            }
        }
    }
}

impl RustLiteral {
    pub fn to_string(&self) -> String {
        match self {
            RustLiteral::String(s) => format!("\"{}\"", escape_string(s)),
            RustLiteral::Int(i) => i.to_string(),
            RustLiteral::Float(f) => f.to_string(),
            RustLiteral::Bool(b) => b.to_string(),
            RustLiteral::Unit => "()".to_string(),
        }
    }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
