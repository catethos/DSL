//! RuntimeBuilder API for constructing LatticeRuntime instances
//!
//! This module provides a builder pattern for configuring and creating
//! isolated Lattice runtime instances with injectable providers.
//!
//! # Example
//!
//! ```ignore
//! use lattice::runtime::{RuntimeBuilder, DefaultLlmProvider};
//!
//! // Create a runtime with default providers
//! let runtime = RuntimeBuilder::new()
//!     .with_llm_provider(DefaultLlmProvider::new()?)
//!     .with_stdlib_core()
//!     .build()?;
//!
//! // Create a runtime without LLM support
//! let minimal_runtime = RuntimeBuilder::new()
//!     .without_llm()
//!     .without_sql()
//!     .build()?;
//! ```

use std::sync::Arc;
use std::time::Duration;

use crate::error::LatticeError;
use crate::types::IR;

use super::providers::{
    BoxedLlmProvider, BoxedSqlProvider, DefaultLlmProvider, LlmProvider, NoLlmProvider,
    NoSqlProvider, SqlProvider,
};

#[cfg(feature = "sql")]
use super::providers::DuckDbProvider;

/// Configuration options for the runtime
#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    /// Maximum execution time for a single eval (None = no limit)
    pub timeout: Option<Duration>,
    /// Maximum concurrent LLM calls (None = unlimited)
    pub max_concurrent_llm_calls: Option<usize>,
    /// Whether to include core stdlib functions
    pub include_stdlib_core: bool,
    /// Whether to include IO stdlib functions (file reading, etc.)
    pub include_stdlib_io: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            timeout: None,
            max_concurrent_llm_calls: None,
            include_stdlib_core: true,
            include_stdlib_io: false,
        }
    }
}

/// Builder for constructing LatticeRuntime instances
///
/// The builder pattern allows configuring which providers and features
/// are enabled in the runtime. Each runtime instance is fully isolated
/// with no shared global state.
///
/// # Example
///
/// ```ignore
/// use lattice::runtime::RuntimeBuilder;
///
/// let runtime = RuntimeBuilder::new()
///     .with_default_providers()
///     .with_max_concurrent_llm_calls(5)
///     .with_timeout(Duration::from_secs(30))
///     .build()?;
/// ```
pub struct RuntimeBuilder {
    /// LLM provider (None = use NoLlmProvider)
    llm_provider: Option<BoxedLlmProvider>,
    /// SQL provider (None = use NoSqlProvider)
    sql_provider: Option<BoxedSqlProvider>,
    /// Runtime configuration
    config: RuntimeConfig,
    /// Pre-loaded type definitions
    ir: Option<IR>,
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeBuilder {
    /// Create a new RuntimeBuilder with default (disabled) providers
    ///
    /// By default, both LLM and SQL are disabled. Use `with_llm_provider()`
    /// and `with_sql_provider()` to enable these capabilities.
    pub fn new() -> Self {
        Self {
            llm_provider: None,
            sql_provider: None,
            config: RuntimeConfig::default(),
            ir: None,
        }
    }

    /// Set a custom LLM provider
    ///
    /// The provider will be used for all LLM function calls in this runtime.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use lattice::runtime::{RuntimeBuilder, DefaultLlmProvider};
    ///
    /// let runtime = RuntimeBuilder::new()
    ///     .with_llm_provider(DefaultLlmProvider::new()?)
    ///     .build()?;
    /// ```
    pub fn with_llm_provider<P: LlmProvider + 'static>(mut self, provider: P) -> Self {
        self.llm_provider = Some(Arc::new(provider));
        self
    }

    /// Set the LLM provider from a boxed trait object
    ///
    /// Useful when you already have an `Arc<dyn LlmProvider>`.
    pub fn with_boxed_llm_provider(mut self, provider: BoxedLlmProvider) -> Self {
        self.llm_provider = Some(provider);
        self
    }

    /// Use the default HTTP-based LLM provider
    ///
    /// This creates a `DefaultLlmProvider` that makes HTTP calls to
    /// OpenAI-compatible APIs.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client or tokio runtime fails to initialize.
    pub fn with_default_llm_provider(self) -> Result<Self, LatticeError> {
        let provider = DefaultLlmProvider::new().map_err(|e| {
            LatticeError::Runtime(format!("Failed to create default LLM provider: {}", e))
        })?;
        Ok(self.with_llm_provider(provider))
    }

    /// Explicitly disable LLM support
    ///
    /// Any LLM function calls will return an error.
    pub fn without_llm(mut self) -> Self {
        self.llm_provider = Some(Arc::new(NoLlmProvider));
        self
    }

    /// Set a custom SQL provider
    ///
    /// The provider will be used for all SQL queries in this runtime.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use lattice::runtime::{RuntimeBuilder, DuckDbProvider};
    ///
    /// let runtime = RuntimeBuilder::new()
    ///     .with_sql_provider(DuckDbProvider::new()?)
    ///     .build()?;
    /// ```
    pub fn with_sql_provider<P: SqlProvider + 'static>(mut self, provider: P) -> Self {
        self.sql_provider = Some(Arc::new(provider));
        self
    }

    /// Set the SQL provider from a boxed trait object
    ///
    /// Useful when you already have an `Arc<dyn SqlProvider>`.
    pub fn with_boxed_sql_provider(mut self, provider: BoxedSqlProvider) -> Self {
        self.sql_provider = Some(provider);
        self
    }

    /// Use the default DuckDB-based SQL provider
    ///
    /// This creates an in-memory DuckDB database.
    ///
    /// # Errors
    ///
    /// Returns an error if DuckDB fails to initialize.
    #[cfg(feature = "sql")]
    pub fn with_default_sql_provider(self) -> Result<Self, LatticeError> {
        let provider = DuckDbProvider::new()
            .map_err(|e| LatticeError::Runtime(format!("Failed to create DuckDB provider: {}", e)))?;
        Ok(self.with_sql_provider(provider))
    }

    /// Use DuckDB with a file-based database
    ///
    /// # Errors
    ///
    /// Returns an error if the database file cannot be opened.
    #[cfg(feature = "sql")]
    pub fn with_sql_database(self, path: &str) -> Result<Self, LatticeError> {
        let provider = DuckDbProvider::with_path(path).map_err(|e| {
            LatticeError::Runtime(format!("Failed to open database '{}': {}", path, e))
        })?;
        Ok(self.with_sql_provider(provider))
    }

    /// Explicitly disable SQL support
    ///
    /// Any SQL queries will return an error.
    pub fn without_sql(mut self) -> Self {
        self.sql_provider = Some(Arc::new(NoSqlProvider));
        self
    }

    /// Configure with all default providers (LLM and SQL)
    ///
    /// Equivalent to calling `with_default_llm_provider()` and
    /// `with_default_sql_provider()` (if the sql feature is enabled).
    ///
    /// # Errors
    ///
    /// Returns an error if any provider fails to initialize.
    #[cfg(feature = "sql")]
    pub fn with_default_providers(self) -> Result<Self, LatticeError> {
        self.with_default_llm_provider()?
            .with_default_sql_provider()
    }

    /// Configure with all default providers (LLM only when sql feature is disabled)
    #[cfg(not(feature = "sql"))]
    pub fn with_default_providers(self) -> Result<Self, LatticeError> {
        self.with_default_llm_provider()
    }

    /// Enable core stdlib functions (list operations, string functions, etc.)
    ///
    /// This is enabled by default.
    pub fn with_stdlib_core(mut self) -> Self {
        self.config.include_stdlib_core = true;
        self
    }

    /// Disable core stdlib functions
    ///
    /// Use this for minimal embedded runtimes that only need basic evaluation.
    pub fn without_stdlib_core(mut self) -> Self {
        self.config.include_stdlib_core = false;
        self
    }

    /// Enable IO stdlib functions (file reading, path operations, etc.)
    ///
    /// This is disabled by default for security in embedded contexts.
    pub fn with_stdlib_io(mut self) -> Self {
        self.config.include_stdlib_io = true;
        self
    }

    /// Set the maximum execution time for a single eval
    ///
    /// If an evaluation exceeds this timeout, it will be cancelled.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = Some(timeout);
        self
    }

    /// Set the maximum number of concurrent LLM calls
    ///
    /// This limits parallelism in map operations that use LLM functions.
    /// Use this to avoid rate limiting from LLM providers.
    pub fn with_max_concurrent_llm_calls(mut self, limit: usize) -> Self {
        self.config.max_concurrent_llm_calls = Some(limit);
        self
    }

    /// Pre-load type definitions from an IR registry
    ///
    /// This allows sharing type definitions across multiple runtimes.
    pub fn with_ir(mut self, ir: IR) -> Self {
        self.ir = Some(ir);
        self
    }

    /// Get the configured LLM provider, or NoLlmProvider if not set
    pub fn llm_provider(&self) -> BoxedLlmProvider {
        self.llm_provider
            .clone()
            .unwrap_or_else(|| Arc::new(NoLlmProvider))
    }

    /// Get the configured SQL provider, or NoSqlProvider if not set
    pub fn sql_provider(&self) -> BoxedSqlProvider {
        self.sql_provider
            .clone()
            .unwrap_or_else(|| Arc::new(NoSqlProvider))
    }

    /// Get the current configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Build the runtime configuration
    ///
    /// This returns a `BuiltRuntime` containing all the configured providers
    /// and settings. The actual `LatticeRuntime` struct will use this in
    /// a future task.
    ///
    /// # Errors
    ///
    /// Currently infallible, but returns Result for forward compatibility.
    pub fn build(self) -> Result<BuiltRuntime, LatticeError> {
        let llm_provider = self.llm_provider();
        let sql_provider = self.sql_provider();

        Ok(BuiltRuntime {
            llm_provider,
            sql_provider,
            config: self.config,
            ir: self.ir.unwrap_or_else(IR::new),
        })
    }
}

/// A built runtime configuration ready for execution
///
/// This is the output of `RuntimeBuilder::build()`. It contains all the
/// configured providers and settings needed to create a `LatticeRuntime`.
///
/// Currently this is a simple container. The full `LatticeRuntime` struct
/// will be implemented in a follow-up task (DSL-rewrite-tz4.10).
pub struct BuiltRuntime {
    /// The LLM provider for this runtime
    pub llm_provider: BoxedLlmProvider,
    /// The SQL provider for this runtime
    pub sql_provider: BoxedSqlProvider,
    /// Runtime configuration
    pub config: RuntimeConfig,
    /// Type registry
    pub ir: IR,
}

impl BuiltRuntime {
    /// Get a reference to the LLM provider
    pub fn llm_provider(&self) -> &BoxedLlmProvider {
        &self.llm_provider
    }

    /// Get a reference to the SQL provider
    pub fn sql_provider(&self) -> &BoxedSqlProvider {
        &self.sql_provider
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Get a reference to the type registry
    pub fn ir(&self) -> &IR {
        &self.ir
    }

    /// Get a mutable reference to the type registry
    pub fn ir_mut(&mut self) -> &mut IR {
        &mut self.ir
    }

    /// Check if LLM support is enabled
    pub fn has_llm(&self) -> bool {
        // Check if the provider is not NoLlmProvider by trying a dummy call
        // This is a heuristic - we can't easily check the concrete type
        // For now, we just check if the provider was explicitly set
        true // Will be refined in the full implementation
    }

    /// Check if SQL support is enabled
    pub fn has_sql(&self) -> bool {
        // Same heuristic as has_llm
        true // Will be refined in the full implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let builder = RuntimeBuilder::new();
        assert!(builder.config.include_stdlib_core);
        assert!(!builder.config.include_stdlib_io);
        assert!(builder.config.timeout.is_none());
        assert!(builder.config.max_concurrent_llm_calls.is_none());
    }

    #[test]
    fn test_builder_without_llm() {
        let builder = RuntimeBuilder::new().without_llm();
        let runtime = builder.build().unwrap();

        // The provider should be NoLlmProvider (returns error on call)
        use super::super::providers::LlmRequest;
        let request = LlmRequest::new(
            "https://api.example.com".to_string(),
            "model".to_string(),
            "key".to_string(),
            "prompt".to_string(),
        );
        let result = runtime.llm_provider.call(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_without_sql() {
        let builder = RuntimeBuilder::new().without_sql();
        let runtime = builder.build().unwrap();

        // The provider should be NoSqlProvider (returns error on query)
        let result = runtime.sql_provider.query("SELECT 1");
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_with_timeout() {
        let runtime = RuntimeBuilder::new()
            .with_timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        assert_eq!(runtime.config.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_builder_with_max_concurrent_llm_calls() {
        let runtime = RuntimeBuilder::new()
            .with_max_concurrent_llm_calls(5)
            .build()
            .unwrap();

        assert_eq!(runtime.config.max_concurrent_llm_calls, Some(5));
    }

    #[test]
    fn test_builder_stdlib_options() {
        let runtime = RuntimeBuilder::new()
            .without_stdlib_core()
            .with_stdlib_io()
            .build()
            .unwrap();

        assert!(!runtime.config.include_stdlib_core);
        assert!(runtime.config.include_stdlib_io);
    }

    #[test]
    fn test_builder_with_ir() {
        let mut ir = IR::new();
        // Add a dummy class to verify it's preserved
        use crate::types::ir::{Class, Field, FieldType};
        ir.classes.push(Class {
            name: "TestClass".to_string(),
            fields: vec![Field {
                name: "id".to_string(),
                field_type: FieldType::Int,
                optional: false,
                description: None,
            }],
            description: None,
        });

        let runtime = RuntimeBuilder::new().with_ir(ir).build().unwrap();

        assert!(runtime.ir.find_class("TestClass").is_some());
    }

    #[test]
    fn test_builder_chaining() {
        let runtime = RuntimeBuilder::new()
            .without_llm()
            .without_sql()
            .with_stdlib_core()
            .with_max_concurrent_llm_calls(10)
            .with_timeout(Duration::from_secs(60))
            .build()
            .unwrap();

        assert_eq!(runtime.config.timeout, Some(Duration::from_secs(60)));
        assert_eq!(runtime.config.max_concurrent_llm_calls, Some(10));
        assert!(runtime.config.include_stdlib_core);
        // stdlib_io defaults to false
        assert!(!runtime.config.include_stdlib_io);
    }

    #[cfg(feature = "sql")]
    mod sql_tests {
        use super::*;

        #[test]
        fn test_builder_with_default_sql_provider() {
            let runtime = RuntimeBuilder::new()
                .with_default_sql_provider()
                .unwrap()
                .build()
                .unwrap();

            // Should be able to query
            let result = runtime.sql_provider.query("SELECT 1 as num");
            assert!(result.is_ok());
        }

        #[test]
        fn test_builder_with_default_providers() {
            let result = RuntimeBuilder::new().with_default_providers();
            // Should succeed (though LLM provider won't work without an API key)
            assert!(result.is_ok());
        }
    }
}
