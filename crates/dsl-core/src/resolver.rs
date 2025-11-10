//! Resolver: Groups function clauses and resolves symbol references
//!
//! This module implements the Resolver pass that:
//! - Groups function definitions by (name, arity) into FunctionGroups
//! - Maintains a symbol table for both file and REPL modes
//! - Provides a single source of truth for function classification
//!
//! Architecture:
//! - Parse → Raw AST (all functions as clauses)
//! - Resolve → Grouped AST with FunctionGroups
//! - Compile → IR

use crate::parser::{
    Expr, FunctionDef, FunctionExecution, Pattern, PatternFunctionClause, Program,
};
use dsl_types::FieldType;
use std::collections::HashMap;

/// A function clause with patterns (unified representation)
#[derive(Debug, Clone)]
pub struct Clause {
    pub patterns: Vec<Pattern>,
    pub guard: Option<Expr>,
    pub body: FunctionBody,
    pub return_type: Option<FieldType>,
}

/// Function body types (renamed from FunctionExecution)
#[derive(Debug, Clone)]
pub enum FunctionBody {
    /// Regular expression-based execution (the default case)
    Expr(Box<Expr>),
    /// LLM-based execution with prompt
    LLM {
        prompt: String,
        model: Option<String>,
        base_url: Option<String>,
        api_key_env: Option<String>,
        temperature: Option<f64>,
    },
    /// HTTP request execution
    HTTP {
        method: String,
        url: String,
        params: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
    },
    /// SQL query execution
    SQL { query: String },
}

/// A group of function clauses with the same name and arity
#[derive(Debug, Clone)]
pub struct FunctionGroup {
    pub name: String,
    pub arity: usize,
    pub clauses: Vec<Clause>,
}

impl FunctionGroup {
    /// Check if this is a trivial function group (single clause with simple variable patterns)
    pub fn is_trivial(&self) -> bool {
        if self.clauses.len() != 1 {
            return false;
        }

        let clause = &self.clauses[0];

        // Check if all patterns are simple variables
        clause
            .patterns
            .iter()
            .all(|p| matches!(p, Pattern::Variable(_)))
    }
}

/// Symbol table for managing function definitions
/// Key: (name, arity)
pub struct SymbolTable {
    functions: HashMap<(String, usize), FunctionGroup>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            functions: HashMap::new(),
        }
    }

    /// Add a clause to the symbol table
    /// Returns true if this created a new function group, false if it added to existing
    pub fn add_clause(&mut self, name: String, clause: Clause) -> bool {
        let arity = clause.patterns.len();
        let key = (name.clone(), arity);

        if let Some(group) = self.functions.get_mut(&key) {
            group.clauses.push(clause);
            false // Added to existing group
        } else {
            self.functions.insert(
                key,
                FunctionGroup {
                    name,
                    arity,
                    clauses: vec![clause],
                },
            );
            true // Created new group
        }
    }

    /// Convert a FunctionDef to a Clause and add it to the symbol table
    pub fn add_function_def(&mut self, func: FunctionDef) -> Result<bool, String> {
        let clause = convert_function_def_to_clause(func)?;
        let name = clause
            .patterns
            .first()
            .map(|_| String::new()) // We'll need the name from the FunctionDef
            .ok_or("Empty clause")?;

        // We need to extract the name before converting - let's refactor this
        Ok(self.add_clause(name, clause))
    }

    /// Get all function groups
    pub fn get_all_groups(&self) -> Vec<FunctionGroup> {
        self.functions.values().cloned().collect()
    }

    /// Get a specific function group by name and arity
    pub fn get_group(&self, name: &str, arity: usize) -> Option<&FunctionGroup> {
        self.functions.get(&(name.to_string(), arity))
    }

    /// Check if a function with this name and arity exists
    pub fn has_function(&self, name: &str, arity: usize) -> bool {
        self.functions.contains_key(&(name.to_string(), arity))
    }

    /// Get all function names (with their arities)
    pub fn function_names(&self) -> Vec<(String, usize)> {
        self.functions.keys().cloned().collect()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a FunctionDef to a Clause
pub fn convert_function_def_to_clause(func: FunctionDef) -> Result<Clause, String> {
    // Convert parameters to simple Variable patterns
    let patterns: Vec<Pattern> = func.params.into_iter().map(Pattern::Variable).collect();

    // Convert FunctionExecution to FunctionBody
    let body = match func.execution {
        FunctionExecution::Expression { body } => FunctionBody::Expr(body),
        FunctionExecution::LLM {
            prompt,
            model,
            base_url,
            api_key_env,
            temperature,
        } => FunctionBody::LLM {
            prompt,
            model,
            base_url,
            api_key_env,
            temperature,
        },
        FunctionExecution::HTTP {
            method,
            url,
            params,
            headers,
            body,
        } => FunctionBody::HTTP {
            method,
            url,
            params,
            headers,
            body,
        },
        FunctionExecution::SQL { query } => FunctionBody::SQL { query },
    };

    Ok(Clause {
        patterns,
        guard: None,
        body,
        return_type: func.return_type,
    })
}

/// Convert a PatternFunctionClause to a Clause
pub fn convert_pattern_clause_to_clause(pfc: PatternFunctionClause) -> Clause {
    Clause {
        patterns: pfc.param_patterns,
        guard: pfc.guard,
        body: FunctionBody::Expr(Box::new(pfc.body)),
        return_type: None,
    }
}

/// Resolve a Program into function groups using a symbol table
pub fn resolve_program(
    program: Program,
    symbol_table: &mut SymbolTable,
) -> Result<Vec<FunctionGroup>, String> {
    // Add all simple functions to the symbol table
    for func in program.functions {
        let name = func.name.clone();
        let clause = convert_function_def_to_clause(func)?;
        symbol_table.add_clause(name, clause);
    }

    // Add all pattern function clauses to the symbol table
    for pattern_func in program.pattern_functions {
        let name = pattern_func.name;
        for pfc in pattern_func.clauses {
            let clause = convert_pattern_clause_to_clause(pfc);
            symbol_table.add_clause(name.clone(), clause);
        }
    }

    Ok(symbol_table.get_all_groups())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_basic() {
        let mut table = SymbolTable::new();

        let clause1 = Clause {
            patterns: vec![Pattern::Variable("x".to_string())],
            guard: None,
            body: FunctionBody::Expr(Box::new(Expr::Variable("x".to_string()))),
            return_type: None,
        };

        let created = table.add_clause("identity".to_string(), clause1);
        assert!(created);
        assert!(table.has_function("identity", 1));
    }

    #[test]
    fn test_symbol_table_overloading() {
        let mut table = SymbolTable::new();

        let clause1 = Clause {
            patterns: vec![Pattern::Literal(Box::new(Expr::Int(0)))],
            guard: None,
            body: FunctionBody::Expr(Box::new(Expr::Int(1))),
            return_type: None,
        };

        let clause2 = Clause {
            patterns: vec![Pattern::Variable("n".to_string())],
            guard: None,
            body: FunctionBody::Expr(Box::new(Expr::Variable("n".to_string()))),
            return_type: None,
        };

        let created1 = table.add_clause("fib".to_string(), clause1);
        let created2 = table.add_clause("fib".to_string(), clause2);

        assert!(created1);
        assert!(!created2); // Second clause added to existing group

        let group = table.get_group("fib", 1).unwrap();
        assert_eq!(group.clauses.len(), 2);
        assert!(!group.is_trivial()); // Not trivial because has pattern matching
    }

    #[test]
    fn test_is_trivial() {
        let trivial_group = FunctionGroup {
            name: "add".to_string(),
            arity: 2,
            clauses: vec![Clause {
                patterns: vec![
                    Pattern::Variable("a".to_string()),
                    Pattern::Variable("b".to_string()),
                ],
                guard: None,
                body: FunctionBody::Expr(Box::new(Expr::Int(0))),
                return_type: None,
            }],
        };

        assert!(trivial_group.is_trivial());

        let non_trivial_group = FunctionGroup {
            name: "fib".to_string(),
            arity: 1,
            clauses: vec![
                Clause {
                    patterns: vec![Pattern::Literal(Box::new(Expr::Int(0)))],
                    guard: None,
                    body: FunctionBody::Expr(Box::new(Expr::Int(1))),
                    return_type: None,
                },
                Clause {
                    patterns: vec![Pattern::Variable("n".to_string())],
                    guard: None,
                    body: FunctionBody::Expr(Box::new(Expr::Int(0))),
                    return_type: None,
                },
            ],
        };

        assert!(!non_trivial_group.is_trivial());
    }
}
