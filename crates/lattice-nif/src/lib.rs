//! Rustler NIF bindings for embedding Lattice in Elixir
//!
//! This crate provides NIF (Native Implemented Functions) that allow
//! Elixir applications to use the Lattice runtime.
//!
//! # Usage in Elixir
//!
//! ```elixir
//! defmodule Lattice.Native do
//!   use Rustler, otp_app: :my_app, crate: "lattice_nif"
//!
//!   def new_runtime(), do: :erlang.nif_error(:nif_not_loaded)
//!   def eval(_runtime, _source), do: :erlang.nif_error(:nif_not_loaded)
//!   def call(_runtime, _name, _args), do: :erlang.nif_error(:nif_not_loaded)
//!   def get_types(_runtime), do: :erlang.nif_error(:nif_not_loaded)
//! end
//! ```
//!
//! # Example
//!
//! ```elixir
//! {:ok, rt} = Lattice.Native.new_runtime()
//! {:ok, 6} = Lattice.Native.eval(rt, "1 + 2 + 3")
//! ```

use rustler::{Encoder, Env, NifResult, ResourceArc, Term};
use std::sync::Mutex;

use lattice::runtime::{
    FunctionSignature, LatticeRuntime, LatticeValue, ParameterSchema, RuntimeBuilder, TypeSchema,
};

mod atoms {
    rustler::atoms! {
        ok,
        error,
        null,
        // Type tags for Elixir
        lattice_int,
        lattice_float,
        lattice_string,
        lattice_bool,
        lattice_list,
        lattice_map,
        lattice_path,
        lattice_null,
        // TypeSchema tags
        null_type,
        int,
        float,
        string,
        bool,
        path,
        any,
        list,
        map,
        optional,
        struct_type,
        enum_type,
        named,
        // Other atoms
        name,
        fields,
        variants,
        type_schema,
        params,
        return_type,
        is_llm,
        is_async,
        optional_field,
        description,
    }
}

/// Resource wrapper for LatticeRuntime
///
/// We wrap in a Mutex because Rustler resources must be Send + Sync,
/// and LatticeRuntime's eval/call methods require &mut self.
pub struct RuntimeResource(Mutex<LatticeRuntime>);

/// Load function called when the NIF is loaded
#[allow(non_local_definitions)]
fn load(env: Env, _info: Term) -> bool {
    let _ = rustler::resource!(RuntimeResource, env);
    true
}

// ============================================================
// NIF Functions
// ============================================================

/// Create a new Lattice runtime instance.
///
/// Returns `{:ok, runtime}` on success or `{:error, reason}` on failure.
#[rustler::nif]
fn new_runtime<'a>(env: Env<'a>) -> NifResult<Term<'a>> {
    let built = RuntimeBuilder::new()
        .without_llm()
        .without_sql()
        .build()
        .map_err(|e| rustler::Error::Term(Box::new(format!("{}", e))))?;

    let runtime = LatticeRuntime::from_built(built);
    let resource = ResourceArc::new(RuntimeResource(Mutex::new(runtime)));

    Ok((atoms::ok(), resource).encode(env))
}

/// Create a new Lattice runtime with LLM support.
///
/// Returns `{:ok, runtime}` on success or `{:error, reason}` on failure.
#[rustler::nif]
fn new_runtime_with_llm<'a>(env: Env<'a>) -> NifResult<Term<'a>> {
    let built = RuntimeBuilder::new()
        .with_default_llm_provider()
        .map_err(|e| rustler::Error::Term(Box::new(format!("{}", e))))?
        .without_sql()
        .build()
        .map_err(|e| rustler::Error::Term(Box::new(format!("{}", e))))?;

    let runtime = LatticeRuntime::from_built(built);
    let resource = ResourceArc::new(RuntimeResource(Mutex::new(runtime)));

    Ok((atoms::ok(), resource).encode(env))
}

/// Create a new Lattice runtime with SQL support (DuckDB).
///
/// Returns `{:ok, runtime}` on success or `{:error, reason}` on failure.
#[cfg(feature = "sql")]
#[rustler::nif]
fn new_runtime_with_sql<'a>(env: Env<'a>) -> NifResult<Term<'a>> {
    let built = RuntimeBuilder::new()
        .without_llm()
        .with_default_sql_provider()
        .map_err(|e| rustler::Error::Term(Box::new(format!("{}", e))))?
        .build()
        .map_err(|e| rustler::Error::Term(Box::new(format!("{}", e))))?;

    let runtime = LatticeRuntime::from_built(built);
    let resource = ResourceArc::new(RuntimeResource(Mutex::new(runtime)));

    Ok((atoms::ok(), resource).encode(env))
}

/// Create a new Lattice runtime with both LLM and SQL support.
///
/// Returns `{:ok, runtime}` on success or `{:error, reason}` on failure.
#[cfg(feature = "sql")]
#[rustler::nif]
fn new_runtime_with_all<'a>(env: Env<'a>) -> NifResult<Term<'a>> {
    let built = RuntimeBuilder::new()
        .with_default_llm_provider()
        .map_err(|e| rustler::Error::Term(Box::new(format!("{}", e))))?
        .with_default_sql_provider()
        .map_err(|e| rustler::Error::Term(Box::new(format!("{}", e))))?
        .build()
        .map_err(|e| rustler::Error::Term(Box::new(format!("{}", e))))?;

    let runtime = LatticeRuntime::from_built(built);
    let resource = ResourceArc::new(RuntimeResource(Mutex::new(runtime)));

    Ok((atoms::ok(), resource).encode(env))
}

/// Evaluate Lattice source code.
///
/// Returns `{:ok, value}` on success or `{:error, reason}` on failure.
#[rustler::nif]
fn eval<'a>(
    env: Env<'a>,
    runtime: ResourceArc<RuntimeResource>,
    source: String,
) -> NifResult<Term<'a>> {
    let mut rt = runtime
        .0
        .lock()
        .map_err(|_| rustler::Error::Term(Box::new("Lock poisoned")))?;

    match rt.eval(&source) {
        Ok(value) => Ok((atoms::ok(), lattice_value_to_term(env, &value)).encode(env)),
        Err(e) => Ok((atoms::error(), format!("{}", e)).encode(env)),
    }
}

/// Evaluate a Lattice file (.lat or .md).
///
/// Reads the file, resolves imports, compiles, and executes.
/// Returns `{:ok, value}` on success or `{:error, reason}` on failure.
#[rustler::nif]
fn eval_file<'a>(
    env: Env<'a>,
    runtime: ResourceArc<RuntimeResource>,
    path: String,
) -> NifResult<Term<'a>> {
    let mut rt = runtime
        .0
        .lock()
        .map_err(|_| rustler::Error::Term(Box::new("Lock poisoned")))?;

    match rt.eval_file(std::path::Path::new(&path)) {
        Ok(value) => Ok((atoms::ok(), lattice_value_to_term(env, &value)).encode(env)),
        Err(e) => Ok((atoms::error(), format!("{}", e)).encode(env)),
    }
}

/// Evaluate Lattice source code with import resolution relative to a base path.
///
/// Useful when the source code comes from a different location than where imports should resolve.
/// Returns `{:ok, value}` on success or `{:error, reason}` on failure.
#[rustler::nif]
fn eval_with_base_path<'a>(
    env: Env<'a>,
    runtime: ResourceArc<RuntimeResource>,
    source: String,
    base_path: String,
) -> NifResult<Term<'a>> {
    let mut rt = runtime
        .0
        .lock()
        .map_err(|_| rustler::Error::Term(Box::new("Lock poisoned")))?;

    match rt.eval_with_base_path(&source, std::path::Path::new(&base_path)) {
        Ok(value) => Ok((atoms::ok(), lattice_value_to_term(env, &value)).encode(env)),
        Err(e) => Ok((atoms::error(), format!("{}", e)).encode(env)),
    }
}

/// Evaluate Lattice source code with pre-bound variables.
///
/// Bindings is a list of `{name, value}` tuples.
/// Returns `{:ok, value}` on success or `{:error, reason}` on failure.
#[rustler::nif]
fn eval_with_bindings<'a>(
    env: Env<'a>,
    runtime: ResourceArc<RuntimeResource>,
    source: String,
    bindings: Vec<(String, Term<'a>)>,
) -> NifResult<Term<'a>> {
    let mut rt = runtime
        .0
        .lock()
        .map_err(|_| rustler::Error::Term(Box::new("Lock poisoned")))?;

    // Convert Elixir terms to LatticeValues
    let lattice_bindings: Vec<(String, LatticeValue)> = bindings
        .into_iter()
        .map(|(name, term)| {
            let value = term_to_lattice_value(term)?;
            Ok((name, value))
        })
        .collect::<NifResult<Vec<_>>>()?;

    match rt.eval_with_bindings(&source, lattice_bindings) {
        Ok(value) => Ok((atoms::ok(), lattice_value_to_term(env, &value)).encode(env)),
        Err(e) => Ok((atoms::error(), format!("{}", e)).encode(env)),
    }
}

/// Call a Lattice function by name with arguments.
///
/// Returns `{:ok, value}` on success or `{:error, reason}` on failure.
#[rustler::nif]
fn call_function<'a>(
    env: Env<'a>,
    runtime: ResourceArc<RuntimeResource>,
    name: String,
    args: Vec<Term<'a>>,
) -> NifResult<Term<'a>> {
    let mut rt = runtime
        .0
        .lock()
        .map_err(|_| rustler::Error::Term(Box::new("Lock poisoned")))?;

    // Convert Elixir terms to LatticeValues
    let lattice_args: Vec<LatticeValue> = args
        .into_iter()
        .map(term_to_lattice_value)
        .collect::<NifResult<Vec<_>>>()?;

    match rt.call(&name, lattice_args) {
        Ok(value) => Ok((atoms::ok(), lattice_value_to_term(env, &value)).encode(env)),
        Err(e) => Ok((atoms::error(), format!("{}", e)).encode(env)),
    }
}

/// Get all registered type schemas.
///
/// Returns a list of type schema maps.
#[rustler::nif]
fn get_types<'a>(env: Env<'a>, runtime: ResourceArc<RuntimeResource>) -> NifResult<Term<'a>> {
    let rt = runtime
        .0
        .lock()
        .map_err(|_| rustler::Error::Term(Box::new("Lock poisoned")))?;

    let schemas = rt.get_types();
    let terms: Vec<Term<'a>> = schemas
        .iter()
        .map(|schema| type_schema_to_term(env, schema))
        .collect();

    Ok(terms.encode(env))
}

/// Get all function signatures.
///
/// Returns a list of function signature maps.
#[rustler::nif]
fn get_function_signatures<'a>(
    env: Env<'a>,
    runtime: ResourceArc<RuntimeResource>,
) -> NifResult<Term<'a>> {
    let rt = runtime
        .0
        .lock()
        .map_err(|_| rustler::Error::Term(Box::new("Lock poisoned")))?;

    let signatures = rt.get_function_signatures();
    let terms: Vec<Term<'a>> = signatures
        .iter()
        .map(|sig| function_signature_to_term(env, sig))
        .collect();

    Ok(terms.encode(env))
}

/// Check if a function exists by name.
#[rustler::nif]
fn has_function(runtime: ResourceArc<RuntimeResource>, name: String) -> NifResult<bool> {
    let rt = runtime
        .0
        .lock()
        .map_err(|_| rustler::Error::Term(Box::new("Lock poisoned")))?;

    Ok(rt.has_function(&name))
}

/// Get a global variable value.
///
/// Returns `{:ok, value}` if found, `{:error, :not_found}` if not.
#[rustler::nif]
fn get_global<'a>(
    env: Env<'a>,
    runtime: ResourceArc<RuntimeResource>,
    name: String,
) -> NifResult<Term<'a>> {
    let rt = runtime
        .0
        .lock()
        .map_err(|_| rustler::Error::Term(Box::new("Lock poisoned")))?;

    match rt.get_global(&name) {
        Some(value) => Ok((atoms::ok(), lattice_value_to_term(env, &value)).encode(env)),
        None => Ok((atoms::error(), "not_found").encode(env)),
    }
}

/// Set a global variable.
#[rustler::nif]
fn set_global<'a>(
    env: Env<'a>,
    runtime: ResourceArc<RuntimeResource>,
    name: String,
    value: Term<'a>,
) -> NifResult<Term<'a>> {
    let mut rt = runtime
        .0
        .lock()
        .map_err(|_| rustler::Error::Term(Box::new("Lock poisoned")))?;

    let lattice_value = term_to_lattice_value(value)?;
    rt.set_global(&name, lattice_value);

    Ok(atoms::ok().encode(env))
}

/// Reset the runtime, clearing all state.
#[rustler::nif]
fn reset<'a>(env: Env<'a>, runtime: ResourceArc<RuntimeResource>) -> NifResult<Term<'a>> {
    let mut rt = runtime
        .0
        .lock()
        .map_err(|_| rustler::Error::Term(Box::new("Lock poisoned")))?;

    rt.reset();
    Ok(atoms::ok().encode(env))
}

// ============================================================
// Value Conversion Functions
// ============================================================

/// Convert a LatticeValue to an Elixir term.
fn lattice_value_to_term<'a>(env: Env<'a>, value: &LatticeValue) -> Term<'a> {
    match value {
        LatticeValue::Null => atoms::null().encode(env),
        LatticeValue::Bool(b) => b.encode(env),
        LatticeValue::Int(i) => i.encode(env),
        LatticeValue::Float(f) => f.encode(env),
        LatticeValue::String(s) => s.encode(env),
        LatticeValue::Path(p) => {
            // Return as {:path, "string"} tuple to distinguish from regular strings
            (atoms::lattice_path(), p.as_str()).encode(env)
        }
        LatticeValue::List(items) => {
            let terms: Vec<Term<'a>> = items
                .iter()
                .map(|item| lattice_value_to_term(env, item))
                .collect();
            terms.encode(env)
        }
        LatticeValue::Map(pairs) => {
            // Convert to Elixir map
            let map_pairs: Vec<(Term<'a>, Term<'a>)> = pairs
                .iter()
                .map(|(k, v)| (k.encode(env), lattice_value_to_term(env, v)))
                .collect();
            Term::map_from_pairs(env, &map_pairs).expect("Failed to create map")
        }
    }
}

/// Convert an Elixir term to a LatticeValue.
fn term_to_lattice_value(term: Term) -> NifResult<LatticeValue> {
    // Check for null atom
    if let Ok(atom) = term.decode::<rustler::Atom>() {
        if atom == atoms::null() {
            return Ok(LatticeValue::Null);
        }
        if atom == rustler::types::atom::true_() {
            return Ok(LatticeValue::Bool(true));
        }
        if atom == rustler::types::atom::false_() {
            return Ok(LatticeValue::Bool(false));
        }
        // Unknown atom - treat as string
        return Ok(LatticeValue::String(format!("{:?}", atom)));
    }

    // Check for integer
    if let Ok(i) = term.decode::<i64>() {
        return Ok(LatticeValue::Int(i));
    }

    // Check for float
    if let Ok(f) = term.decode::<f64>() {
        return Ok(LatticeValue::Float(f));
    }

    // Check for string/binary
    if let Ok(s) = term.decode::<String>() {
        return Ok(LatticeValue::String(s));
    }

    // Check for list
    if let Ok(list) = term.decode::<Vec<Term>>() {
        let items: Vec<LatticeValue> = list
            .into_iter()
            .map(term_to_lattice_value)
            .collect::<NifResult<Vec<_>>>()?;
        return Ok(LatticeValue::List(items));
    }

    // Check for map
    if let Ok(iter) = term.decode::<rustler::MapIterator>() {
        let pairs: Vec<(String, LatticeValue)> = iter
            .map(|(k, v)| {
                let key: String = k.decode().map_err(|_| {
                    rustler::Error::Term(Box::new("Map keys must be strings"))
                })?;
                let value = term_to_lattice_value(v)?;
                Ok((key, value))
            })
            .collect::<NifResult<Vec<_>>>()?;
        return Ok(LatticeValue::Map(pairs));
    }

    // Check for path tuple {:path, "string"}
    if let Ok((tag, path_str)) = term.decode::<(rustler::Atom, String)>() {
        if tag == atoms::lattice_path() {
            return Ok(LatticeValue::Path(path_str));
        }
    }

    Err(rustler::Error::Term(Box::new(
        "Cannot convert term to LatticeValue",
    )))
}

/// Convert a TypeSchema to an Elixir term (map).
fn type_schema_to_term<'a>(env: Env<'a>, schema: &TypeSchema) -> Term<'a> {
    match schema {
        TypeSchema::Null => rustler::types::map::map_new(env)
            .map_put(
                atoms::type_schema().encode(env),
                atoms::null_type().encode(env),
            )
            .expect("map put failed"),
        TypeSchema::Int => rustler::types::map::map_new(env)
            .map_put(atoms::type_schema().encode(env), atoms::int().encode(env))
            .expect("map put failed"),
        TypeSchema::Float => rustler::types::map::map_new(env)
            .map_put(atoms::type_schema().encode(env), atoms::float().encode(env))
            .expect("map put failed"),
        TypeSchema::String => rustler::types::map::map_new(env)
            .map_put(
                atoms::type_schema().encode(env),
                atoms::string().encode(env),
            )
            .expect("map put failed"),
        TypeSchema::Bool => rustler::types::map::map_new(env)
            .map_put(atoms::type_schema().encode(env), atoms::bool().encode(env))
            .expect("map put failed"),
        TypeSchema::Path => rustler::types::map::map_new(env)
            .map_put(atoms::type_schema().encode(env), atoms::path().encode(env))
            .expect("map put failed"),
        TypeSchema::Any => rustler::types::map::map_new(env)
            .map_put(atoms::type_schema().encode(env), atoms::any().encode(env))
            .expect("map put failed"),
        TypeSchema::List(inner) => {
            let inner_term = type_schema_to_term(env, inner);
            rustler::types::map::map_new(env)
                .map_put(atoms::type_schema().encode(env), atoms::list().encode(env))
                .expect("map put failed")
                .map_put("inner".encode(env), inner_term)
                .expect("map put failed")
        }
        TypeSchema::Map { key, value } => {
            let key_term = type_schema_to_term(env, key);
            let value_term = type_schema_to_term(env, value);
            rustler::types::map::map_new(env)
                .map_put(atoms::type_schema().encode(env), atoms::map().encode(env))
                .expect("map put failed")
                .map_put("key".encode(env), key_term)
                .expect("map put failed")
                .map_put("value".encode(env), value_term)
                .expect("map put failed")
        }
        TypeSchema::Optional(inner) => {
            let inner_term = type_schema_to_term(env, inner);
            rustler::types::map::map_new(env)
                .map_put(
                    atoms::type_schema().encode(env),
                    atoms::optional().encode(env),
                )
                .expect("map put failed")
                .map_put("inner".encode(env), inner_term)
                .expect("map put failed")
        }
        TypeSchema::Struct(struct_schema) => {
            let fields: Vec<Term<'a>> = struct_schema
                .fields
                .iter()
                .map(|field| {
                    let field_type = type_schema_to_term(env, &field.type_schema);
                    rustler::types::map::map_new(env)
                        .map_put(atoms::name().encode(env), field.name.encode(env))
                        .expect("map put failed")
                        .map_put(atoms::type_schema().encode(env), field_type)
                        .expect("map put failed")
                        .map_put(
                            atoms::optional_field().encode(env),
                            field.optional.encode(env),
                        )
                        .expect("map put failed")
                })
                .collect();

            rustler::types::map::map_new(env)
                .map_put(
                    atoms::type_schema().encode(env),
                    atoms::struct_type().encode(env),
                )
                .expect("map put failed")
                .map_put(atoms::name().encode(env), struct_schema.name.encode(env))
                .expect("map put failed")
                .map_put(atoms::fields().encode(env), fields.encode(env))
                .expect("map put failed")
        }
        TypeSchema::Enum(enum_schema) => {
            let variants: Vec<&str> = enum_schema.variants.iter().map(|s| s.as_str()).collect();
            rustler::types::map::map_new(env)
                .map_put(
                    atoms::type_schema().encode(env),
                    atoms::enum_type().encode(env),
                )
                .expect("map put failed")
                .map_put(atoms::name().encode(env), enum_schema.name.encode(env))
                .expect("map put failed")
                .map_put(atoms::variants().encode(env), variants.encode(env))
                .expect("map put failed")
        }
        TypeSchema::Named(type_name) => rustler::types::map::map_new(env)
            .map_put(atoms::type_schema().encode(env), atoms::named().encode(env))
            .expect("map put failed")
            .map_put(atoms::name().encode(env), type_name.encode(env))
            .expect("map put failed"),
    }
}

/// Convert a FunctionSignature to an Elixir term (map).
fn function_signature_to_term<'a>(env: Env<'a>, sig: &FunctionSignature) -> Term<'a> {
    let params: Vec<Term<'a>> = sig
        .params
        .iter()
        .map(|param| param_schema_to_term(env, param))
        .collect();

    let return_type_term = type_schema_to_term(env, &sig.return_type);

    rustler::types::map::map_new(env)
        .map_put(atoms::name().encode(env), sig.name.encode(env))
        .expect("map put failed")
        .map_put(atoms::params().encode(env), params.encode(env))
        .expect("map put failed")
        .map_put(atoms::return_type().encode(env), return_type_term)
        .expect("map put failed")
        .map_put(atoms::is_llm().encode(env), sig.is_llm.encode(env))
        .expect("map put failed")
        .map_put(atoms::is_async().encode(env), sig.is_async.encode(env))
        .expect("map put failed")
}

/// Convert a ParameterSchema to an Elixir term (map).
fn param_schema_to_term<'a>(env: Env<'a>, param: &ParameterSchema) -> Term<'a> {
    let type_term = type_schema_to_term(env, &param.type_schema);

    rustler::types::map::map_new(env)
        .map_put(atoms::name().encode(env), param.name.encode(env))
        .expect("map put failed")
        .map_put(atoms::type_schema().encode(env), type_term)
        .expect("map put failed")
}

// ============================================================
// NIF Registration
// ============================================================

rustler::init!("Elixir.Lattice.Native", load = load);
