use dsl_ir::Span;
use std::fmt;

/// Enhanced error type with source location and context information
#[derive(Debug, Clone)]
pub enum InterpreterError {
    /// LLM execution error with full context
    LLMError {
        message: String,
        function_name: Option<String>,
        source_span: Option<Span>,
        prompt: Option<String>,
        response: Option<String>,
    },

    /// HTTP request error with context
    HTTPError {
        message: String,
        function_name: Option<String>,
        source_span: Option<Span>,
        method: Option<String>,
        url: Option<String>,
    },

    /// SQL execution error with context
    SQLError {
        message: String,
        function_name: Option<String>,
        source_span: Option<Span>,
        query: Option<String>,
    },

    /// Type mismatch error
    TypeError {
        message: String,
        expected: String,
        got: String,
        source_span: Option<Span>,
    },

    /// Runtime error with optional span
    RuntimeError {
        message: String,
        source_span: Option<Span>,
    },

    /// Unknown variable
    UnknownVariable {
        name: String,
        source_span: Option<Span>,
    },

    /// Unknown function
    UnknownFunction {
        name: String,
        source_span: Option<Span>,
    },

    /// Unknown intrinsic
    UnknownIntrinsic {
        name: String,
        source_span: Option<Span>,
    },

    /// Invalid arguments
    InvalidArguments {
        message: String,
        source_span: Option<Span>,
    },
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpreterError::LLMError {
                message,
                function_name,
                source_span,
                prompt,
                response,
            } => {
                write!(f, "LLM Error")?;

                if let Some(fname) = function_name {
                    write!(f, " in function '{}'", fname)?;
                }

                if let Some(span) = source_span {
                    write!(f, " at {}:{}:{}", span.file, span.line, span.column)?;
                }

                write!(f, "\n  {}", message)?;

                if let Some(p) = prompt {
                    // Truncate long prompts
                    let truncated = if p.len() > 200 {
                        format!("{}...", &p[..200])
                    } else {
                        p.clone()
                    };
                    write!(f, "\n  Prompt: {}", truncated)?;
                }

                if let Some(r) = response {
                    let truncated = if r.len() > 200 {
                        format!("{}...", &r[..200])
                    } else {
                        r.clone()
                    };
                    write!(f, "\n  Response: {}", truncated)?;
                }

                Ok(())
            }

            InterpreterError::HTTPError {
                message,
                function_name,
                source_span,
                method,
                url,
            } => {
                write!(f, "HTTP Error")?;

                if let Some(fname) = function_name {
                    write!(f, " in function '{}'", fname)?;
                }

                if let Some(span) = source_span {
                    write!(f, " at {}:{}:{}", span.file, span.line, span.column)?;
                }

                write!(f, "\n  {}", message)?;

                if let Some(m) = method {
                    write!(f, "\n  Method: {}", m)?;
                }

                if let Some(u) = url {
                    write!(f, "\n  URL: {}", u)?;
                }

                Ok(())
            }

            InterpreterError::SQLError {
                message,
                function_name,
                source_span,
                query,
            } => {
                write!(f, "SQL Error")?;

                if let Some(fname) = function_name {
                    write!(f, " in function '{}'", fname)?;
                }

                if let Some(span) = source_span {
                    write!(f, " at {}:{}:{}", span.file, span.line, span.column)?;
                }

                write!(f, "\n  {}", message)?;

                if let Some(q) = query {
                    let truncated = if q.len() > 200 {
                        format!("{}...", &q[..200])
                    } else {
                        q.clone()
                    };
                    write!(f, "\n  Query: {}", truncated)?;
                }

                Ok(())
            }

            InterpreterError::TypeError {
                message,
                expected,
                got,
                source_span,
            } => {
                write!(f, "Type Error")?;

                if let Some(span) = source_span {
                    write!(f, " at {}:{}:{}", span.file, span.line, span.column)?;
                }

                write!(f, "\n  {}", message)?;
                write!(f, "\n  Expected: {}", expected)?;
                write!(f, "\n  Got: {}", got)?;

                Ok(())
            }

            InterpreterError::RuntimeError {
                message,
                source_span,
            } => {
                write!(f, "Runtime Error")?;

                if let Some(span) = source_span {
                    write!(f, " at {}:{}:{}", span.file, span.line, span.column)?;
                }

                write!(f, "\n  {}", message)?;

                Ok(())
            }

            InterpreterError::UnknownVariable { name, source_span } => {
                write!(f, "Unknown Variable")?;

                if let Some(span) = source_span {
                    write!(f, " at {}:{}:{}", span.file, span.line, span.column)?;
                }

                write!(f, "\n  Variable '{}' not found", name)?;

                Ok(())
            }

            InterpreterError::UnknownFunction { name, source_span } => {
                write!(f, "Unknown Function")?;

                if let Some(span) = source_span {
                    write!(f, " at {}:{}:{}", span.file, span.line, span.column)?;
                }

                write!(f, "\n  Function '{}' not found", name)?;

                Ok(())
            }

            InterpreterError::UnknownIntrinsic { name, source_span } => {
                write!(f, "Unknown Intrinsic")?;

                if let Some(span) = source_span {
                    write!(f, " at {}:{}:{}", span.file, span.line, span.column)?;
                }

                write!(f, "\n  Intrinsic function '{}' not found", name)?;

                Ok(())
            }

            InterpreterError::InvalidArguments {
                message,
                source_span,
            } => {
                write!(f, "Invalid Arguments")?;

                if let Some(span) = source_span {
                    write!(f, " at {}:{}:{}", span.file, span.line, span.column)?;
                }

                write!(f, "\n  {}", message)?;

                Ok(())
            }
        }
    }
}

impl std::error::Error for InterpreterError {}

/// Convert anyhow::Error to InterpreterError for better error messages
impl From<anyhow::Error> for InterpreterError {
    fn from(error: anyhow::Error) -> Self {
        InterpreterError::RuntimeError {
            message: error.to_string(),
            source_span: None,
        }
    }
}

/// Convert String errors to InterpreterError
impl From<String> for InterpreterError {
    fn from(message: String) -> Self {
        InterpreterError::RuntimeError {
            message,
            source_span: None,
        }
    }
}

/// Convert &str errors to InterpreterError
impl From<&str> for InterpreterError {
    fn from(message: &str) -> Self {
        InterpreterError::RuntimeError {
            message: message.to_string(),
            source_span: None,
        }
    }
}
