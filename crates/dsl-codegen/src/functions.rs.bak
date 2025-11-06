use crate::rust_ast::*;
use anyhow::Result;
use dsl_ir::{IRExecution, IRFunction};

pub fn generate_function(func: &IRFunction) -> Result<RustFunction> {
    match &func.execution {
        IRExecution::LLM {
            prompt,
            model,
            base_url,
            api_key_env,
            temperature,
        } => generate_llm_function(func, prompt, model, base_url, api_key_env, temperature),

        IRExecution::HTTP {
            method,
            url,
            params,
            headers,
            body,
        } => generate_http_function(func, method, url, params, headers, body),

        IRExecution::SQL { query } => generate_sql_function(func, query),

        IRExecution::HTTPWithLLM {
            http_method,
            http_url,
            http_params,
            http_headers,
            llm_prompt,
            llm_model,
            llm_base_url,
            llm_api_key_env,
            llm_temperature,
        } => generate_http_llm_function(
            func,
            http_method,
            http_url,
            http_params,
            http_headers,
            llm_prompt,
            llm_model,
            llm_base_url,
            llm_api_key_env,
            llm_temperature,
        ),
    }
}

fn generate_llm_function(
    func: &IRFunction,
    prompt: &str,
    model: &Option<String>,
    _base_url: &Option<String>,
    _api_key_env: &Option<String>,
    _temperature: &Option<f64>,
) -> Result<RustFunction> {
    let mut body = Vec::new();

    body.push(RustStmt::Expr(RustExpr::Macro {
        name: "println".to_string(),
        args: vec![RustExpr::Literal(RustLiteral::String(format!(
            "Calling LLM function: {}",
            func.name
        )))],
    }));

    body.push(RustStmt::Let {
        name: "_prompt".to_string(),
        ty: None,
        value: RustExpr::Literal(RustLiteral::String(prompt.to_string())),
        is_mut: false,
    });

    if let Some(model_str) = model {
        body.push(RustStmt::Let {
            name: "_model".to_string(),
            ty: None,
            value: RustExpr::Literal(RustLiteral::String(model_str.clone())),
            is_mut: false,
        });
    }

    body.push(RustStmt::Expr(RustExpr::Macro {
        name: "unimplemented".to_string(),
        args: vec![RustExpr::Literal(RustLiteral::String(
            "LLM function execution".to_string(),
        ))],
    }));

    Ok(RustFunction {
        name: func.name.clone(),
        params: func
            .params
            .iter()
            .map(|p| RustParam {
                name: p.clone(),
                ty: RustType::Named("Value".to_string()),
                is_mut: false,
                is_ref: false,
            })
            .collect(),
        return_type: RustType::Generic(
            "Result".to_string(),
            vec![RustType::Named("Value".to_string())],
        ),
        is_async: true,
        is_public: true,
        body,
    })
}

fn generate_http_function(
    func: &IRFunction,
    method: &str,
    url: &str,
    _params: &Option<std::collections::HashMap<String, String>>,
    _headers: &Option<std::collections::HashMap<String, String>>,
    _body: &Option<String>,
) -> Result<RustFunction> {
    let mut stmts = Vec::new();

    stmts.push(RustStmt::Let {
        name: "_client".to_string(),
        ty: None,
        value: RustExpr::Call {
            func: Box::new(RustExpr::Variable("reqwest::Client::new".to_string())),
            args: vec![],
        },
        is_mut: false,
    });

    stmts.push(RustStmt::Let {
        name: "_method".to_string(),
        ty: None,
        value: RustExpr::Literal(RustLiteral::String(method.to_string())),
        is_mut: false,
    });

    stmts.push(RustStmt::Let {
        name: "_url".to_string(),
        ty: None,
        value: RustExpr::Literal(RustLiteral::String(url.to_string())),
        is_mut: false,
    });

    stmts.push(RustStmt::Expr(RustExpr::Macro {
        name: "unimplemented".to_string(),
        args: vec![RustExpr::Literal(RustLiteral::String(
            "HTTP function execution".to_string(),
        ))],
    }));

    Ok(RustFunction {
        name: func.name.clone(),
        params: func
            .params
            .iter()
            .map(|p| RustParam {
                name: p.clone(),
                ty: RustType::Named("Value".to_string()),
                is_mut: false,
                is_ref: false,
            })
            .collect(),
        return_type: RustType::Generic(
            "Result".to_string(),
            vec![RustType::Named("Value".to_string())],
        ),
        is_async: true,
        is_public: true,
        body: stmts,
    })
}

fn generate_sql_function(func: &IRFunction, query: &str) -> Result<RustFunction> {
    let mut body = Vec::new();

    body.push(RustStmt::Let {
        name: "_query".to_string(),
        ty: None,
        value: RustExpr::Literal(RustLiteral::String(query.to_string())),
        is_mut: false,
    });

    body.push(RustStmt::Expr(RustExpr::Macro {
        name: "unimplemented".to_string(),
        args: vec![RustExpr::Literal(RustLiteral::String(
            "SQL function execution".to_string(),
        ))],
    }));

    Ok(RustFunction {
        name: func.name.clone(),
        params: func
            .params
            .iter()
            .map(|p| RustParam {
                name: p.clone(),
                ty: RustType::Named("Value".to_string()),
                is_mut: false,
                is_ref: false,
            })
            .collect(),
        return_type: RustType::Generic(
            "Result".to_string(),
            vec![RustType::Named("Value".to_string())],
        ),
        is_async: true,
        is_public: true,
        body,
    })
}

fn generate_http_llm_function(
    func: &IRFunction,
    _http_method: &str,
    _http_url: &str,
    _http_params: &Option<std::collections::HashMap<String, String>>,
    _http_headers: &Option<std::collections::HashMap<String, String>>,
    _llm_prompt: &str,
    _llm_model: &Option<String>,
    _llm_base_url: &Option<String>,
    _llm_api_key_env: &Option<String>,
    _llm_temperature: &Option<f64>,
) -> Result<RustFunction> {
    let mut body = Vec::new();

    body.push(RustStmt::Expr(RustExpr::Macro {
        name: "println".to_string(),
        args: vec![RustExpr::Literal(RustLiteral::String(format!(
            "Calling HTTP+LLM function: {}",
            func.name
        )))],
    }));

    body.push(RustStmt::Expr(RustExpr::Macro {
        name: "unimplemented".to_string(),
        args: vec![RustExpr::Literal(RustLiteral::String(
            "HTTP+LLM function execution".to_string(),
        ))],
    }));

    Ok(RustFunction {
        name: func.name.clone(),
        params: func
            .params
            .iter()
            .map(|p| RustParam {
                name: p.clone(),
                ty: RustType::Named("Value".to_string()),
                is_mut: false,
                is_ref: false,
            })
            .collect(),
        return_type: RustType::Generic(
            "Result".to_string(),
            vec![RustType::Named("Value".to_string())],
        ),
        is_async: true,
        is_public: true,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dsl_ir::FieldType;
    use std::collections::HashMap;

    #[test]
    fn test_generate_llm_function() {
        let func = IRFunction {
            name: "summarize".to_string(),
            params: vec!["text".to_string()],
            return_type: Some(FieldType::String),
            properties: HashMap::new(),
            execution: IRExecution::LLM {
                prompt: "Summarize: {{text}}".to_string(),
                model: Some("gpt-4".to_string()),
                base_url: None,
                api_key_env: None,
                temperature: Some(0.7),
            },
        };

        let rust_func = generate_function(&func).unwrap();
        assert_eq!(rust_func.name, "summarize");
        assert_eq!(rust_func.params.len(), 1);
        assert!(rust_func.is_async);
        assert!(rust_func.is_public);
    }

    #[test]
    fn test_generate_sql_function() {
        let func = IRFunction {
            name: "get_users".to_string(),
            params: vec![],
            return_type: Some(FieldType::List(Box::new(FieldType::String))),
            properties: HashMap::new(),
            execution: IRExecution::SQL {
                query: "SELECT * FROM users".to_string(),
            },
        };

        let rust_func = generate_function(&func).unwrap();
        assert_eq!(rust_func.name, "get_users");
        assert!(rust_func.is_async);
    }
}
