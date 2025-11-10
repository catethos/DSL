/// Procedural macros for automatic BAML IR generation
///
/// This crate provides:
/// - `#[derive(BamlSchema)]` macro for automatic class/enum IR generation
/// - `#[baml_function]` macro for defining BAML functions
/// - `baml_client!` macro for configuring LLM clients
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Lit, Meta, ItemFn,
    Expr, Stmt, ReturnType, FnArg, Pat, PatType, parse::Parse, parse::ParseStream,
    Token,
};

/// Parse attribute arguments for baml_function
struct BamlFunctionArgs {
    client: String,
}

impl Parse for BamlFunctionArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut client = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: syn::LitStr = input.parse()?;

            if key == "client" {
                client = Some(value.value());
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(BamlFunctionArgs {
            client: client.expect("#[baml_function] requires a 'client' parameter"),
        })
    }
}

/// Derive macro for automatically implementing the BamlSchema trait
///
/// # Attributes
/// - `#[baml(description = "...")]` - Add description to types and fields
/// - `#[baml(rename = "...")]` - Rename field in schema
///
/// # Example
/// ```ignore
/// #[derive(BamlSchema)]
/// #[baml(description = "A person entity")]
/// struct Person {
///     #[baml(description = "Full name")]
///     name: String,
///     age: i64,
/// }
/// ```
#[proc_macro_derive(BamlSchema, attributes(baml))]
pub fn derive_baml_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        Data::Struct(data_struct) => derive_struct_schema(&input, data_struct),
        Data::Enum(data_enum) => derive_enum_schema(&input, data_enum),
        Data::Union(_) => {
            panic!("BamlSchema does not support unions");
        }
    }
}

fn derive_struct_schema(input: &DeriveInput, data_struct: &syn::DataStruct) -> TokenStream {
    let name = &input.ident;
    let name_str = name.to_string();

    // Extract type-level description
    let type_description = extract_description(&input.attrs);

    // Process fields
    let fields = match &data_struct.fields {
        Fields::Named(fields) => &fields.named,
        _ => panic!("BamlSchema only supports structs with named fields"),
    };

    let field_registrations: Vec<_> = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_ty = &field.ty;
        let field_description = extract_description(&field.attrs);
        let rename = extract_rename(&field.attrs).unwrap_or_else(|| field_name_str.clone());

        // Determine if field is optional and extract inner type
        let (is_optional, field_type_expr) = parse_field_type(field_ty);

        quote! {
            simplify_baml::Field {
                name: #rename.to_string(),
                field_type: #field_type_expr,
                optional: #is_optional,
                description: #field_description,
            }
        }
    }).collect();

    let expanded = quote! {
        impl simplify_baml::BamlSchema for #name {
            fn schema_name() -> &'static str {
                #name_str
            }

            fn register_schemas(registry: &mut simplify_baml::BamlSchemaRegistry) {
                // Register this class
                registry.add_class(simplify_baml::Class {
                    name: #name_str.to_string(),
                    description: #type_description,
                    fields: vec![
                        #(#field_registrations),*
                    ],
                });
            }
        }
    };

    TokenStream::from(expanded)
}

fn derive_enum_schema(input: &DeriveInput, data_enum: &syn::DataEnum) -> TokenStream {
    let name = &input.ident;
    let name_str = name.to_string();

    // Extract type-level description
    let type_description = extract_description(&input.attrs);

    // Extract variant names
    let variants: Vec<String> = data_enum.variants.iter().map(|variant| {
        if !matches!(variant.fields, Fields::Unit) {
            panic!("BamlSchema enums must have unit variants only (no data in variants)");
        }
        variant.ident.to_string()
    }).collect();

    let expanded = quote! {
        impl simplify_baml::BamlSchema for #name {
            fn schema_name() -> &'static str {
                #name_str
            }

            fn register_schemas(registry: &mut simplify_baml::BamlSchemaRegistry) {
                registry.add_enum(simplify_baml::Enum {
                    name: #name_str.to_string(),
                    description: #type_description,
                    values: vec![
                        #(#variants.to_string()),*
                    ],
                });
            }
        }
    };

    TokenStream::from(expanded)
}

/// Extract description from attributes
fn extract_description(attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    for attr in attrs {
        if attr.path().is_ident("baml") {
            if let Ok(meta_list) = attr.meta.require_list() {
                // Parse the entire token stream as a Meta
                if let Ok(Meta::NameValue(nv)) = syn::parse2::<Meta>(meta_list.tokens.clone()) {
                    if nv.path.is_ident("description") {
                        if let syn::Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                let desc = lit_str.value();
                                return quote! { Some(#desc.to_string()) };
                            }
                        }
                    }
                }
            }
        }
    }
    quote! { None }
}

/// Extract rename from attributes
fn extract_rename(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("baml") {
            if let Ok(meta_list) = attr.meta.require_list() {
                if let Ok(Meta::NameValue(nv)) = syn::parse2::<Meta>(meta_list.tokens.clone()) {
                    if nv.path.is_ident("rename") {
                        if let syn::Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                return Some(lit_str.value());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Parse field type and determine if it's optional
/// Returns (is_optional, field_type_expression)
fn parse_field_type(ty: &syn::Type) -> (bool, proc_macro2::TokenStream) {
    // Try to extract the type path
    if let syn::Type::Path(type_path) = ty {
        let path = &type_path.path;

        // Check for Option<T>
        if path.segments.len() == 1 && path.segments[0].ident == "Option" {
            if let syn::PathArguments::AngleBracketed(args) = &path.segments[0].arguments {
                if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                    let (_, inner_expr) = parse_field_type(inner_ty);
                    return (true, inner_expr);
                }
            }
        }

        // Check for Vec<T>
        if path.segments.len() == 1 && path.segments[0].ident == "Vec" {
            if let syn::PathArguments::AngleBracketed(args) = &path.segments[0].arguments {
                if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                    let (_, inner_expr) = parse_field_type(inner_ty);
                    return (false, quote! {
                        simplify_baml::FieldType::List(Box::new(#inner_expr))
                    });
                }
            }
        }

        // Map basic types
        if path.segments.len() == 1 {
            let ident = &path.segments[0].ident;
            let ident_str = ident.to_string();

            let field_type = match ident_str.as_str() {
                "String" | "str" => quote! { simplify_baml::FieldType::String },
                "i64" | "i32" | "i16" | "i8" | "isize" => quote! { simplify_baml::FieldType::Int },
                "u64" | "u32" | "u16" | "u8" | "usize" => quote! { simplify_baml::FieldType::Int },
                "f64" | "f32" => quote! { simplify_baml::FieldType::Float },
                "bool" => quote! { simplify_baml::FieldType::Bool },
                _ => {
                    // Assume it's a custom type (Class or Enum)
                    let type_name = ident_str;
                    quote! { simplify_baml::FieldType::Class(#type_name.to_string()) }
                }
            };

            return (false, field_type);
        }
    }

    // Default fallback
    panic!("Unsupported field type: {:?}", ty);
}

/// Attribute macro for defining BAML functions
///
/// # Usage
/// ```ignore
/// #[baml_function(client = "openai")]
/// fn extract_person(
///     #[baml(description = "Text to extract from")]
///     text: String
/// ) -> Person {
///     r#"Extract person info from: {{ text }}"#
/// }
/// ```
///
/// This generates a function that returns a `Function` definition.
#[proc_macro_attribute]
pub fn baml_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as ItemFn);
    let args = parse_macro_input!(args as BamlFunctionArgs);

    // Extract function name
    let fn_name = &item_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    // Convert to PascalCase for the function name
    let pascal_case_name = to_pascal_case(&fn_name_str);

    // Extract client from attribute args
    let client_name = args.client;

    // Extract input parameters
    let inputs = extract_function_inputs(&item_fn.sig.inputs);

    // Extract return type
    let output = extract_return_type(&item_fn.sig.output);

    // Extract prompt from function body
    let prompt = extract_prompt_from_body(&item_fn.block);

    // Generate a function that returns the Function definition
    let expanded = quote! {
        pub fn #fn_name() -> simplify_baml::Function {
            simplify_baml::Function {
                name: #pascal_case_name.to_string(),
                inputs: vec![#(#inputs),*],
                output: #output,
                prompt_template: #prompt.to_string(),
                client: #client_name.to_string(),
            }
        }
    };

    TokenStream::from(expanded)
}

/// Convert snake_case to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Extract function inputs from signature
fn extract_function_inputs(inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>) -> Vec<proc_macro2::TokenStream> {
    inputs.iter().filter_map(|arg| {
        if let FnArg::Typed(PatType { pat, ty, attrs, .. }) = arg {
            // Extract parameter name
            let param_name = if let Pat::Ident(pat_ident) = &**pat {
                pat_ident.ident.to_string()
            } else {
                panic!("Expected identifier pattern for function parameter");
            };

            // Extract description from attributes
            let description = extract_description(attrs);

            // Parse type
            let (is_optional, field_type_expr) = parse_field_type(ty);

            Some(quote! {
                simplify_baml::Field {
                    name: ::std::string::String::from(#param_name),
                    field_type: #field_type_expr,
                    optional: #is_optional,
                    description: #description,
                }
            })
        } else {
            None
        }
    }).collect()
}

/// Extract return type from function signature
fn extract_return_type(return_type: &ReturnType) -> proc_macro2::TokenStream {
    match return_type {
        ReturnType::Default => panic!("BAML functions must have a return type"),
        ReturnType::Type(_, ty) => {
            let (_, field_type_expr) = parse_field_type(ty);
            field_type_expr
        }
    }
}

/// Extract prompt string literal from function body
fn extract_prompt_from_body(block: &syn::Block) -> String {
    // Expect a single expression statement containing a string literal
    if block.stmts.len() != 1 {
        panic!("BAML function body must contain exactly one string literal (the prompt template)");
    }

    if let Stmt::Expr(Expr::Lit(expr_lit), _) = &block.stmts[0] {
        if let Lit::Str(lit_str) = &expr_lit.lit {
            return lit_str.value();
        }
    }

    panic!("BAML function body must be a string literal containing the prompt template");
}

/// Derive macro for automatically implementing LLM client configuration
///
/// # Attributes
/// - `#[baml(provider = "OpenAI", model = "...")]` - OpenAI client
/// - `#[baml(provider = "Anthropic", model = "...")]` - Anthropic client
/// - `#[baml(provider = "Custom", base_url = "...", model = "...")]` - Custom endpoint
///
/// # Example
/// ```ignore
/// #[derive(BamlClient)]
/// #[baml(provider = "OpenAI", model = "gpt-4o-mini")]
/// struct OpenAIClient;
///
/// // Usage:
/// let client = OpenAIClient::new(api_key);
/// ```
#[proc_macro_derive(BamlClient, attributes(baml))]
pub fn derive_baml_client(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Only support unit structs for now
    if !matches!(input.data, Data::Struct(ref s) if matches!(s.fields, Fields::Unit)) {
        panic!("BamlClient only supports unit structs (e.g., `struct Name;`)");
    }

    let name = &input.ident;

    // Extract client configuration from attributes
    let (provider, model, base_url) = extract_client_config(&input.attrs);

    // Generate the appropriate client constructor based on provider
    let client_constructor = match provider.as_str() {
        "OpenAI" => {
            quote! {
                simplify_baml::LLMClient::openai(api_key, #model.to_string())
            }
        }
        "Anthropic" => {
            quote! {
                simplify_baml::LLMClient::anthropic(api_key, #model.to_string())
            }
        }
        "Custom" => {
            let base_url = base_url.expect("Custom provider requires 'base_url' attribute");
            quote! {
                simplify_baml::LLMClient::custom(api_key, #base_url.to_string(), #model.to_string())
            }
        }
        _ => panic!("Unknown provider '{}'. Supported: OpenAI, Anthropic, Custom", provider),
    };

    let expanded = quote! {
        impl #name {
            pub fn new(api_key: String) -> simplify_baml::LLMClient {
                #client_constructor
            }
        }
    };

    TokenStream::from(expanded)
}

/// Extract client configuration from baml attributes
/// Returns (provider, model, base_url)
fn extract_client_config(attrs: &[syn::Attribute]) -> (String, String, Option<String>) {
    let mut provider = None;
    let mut model = None;
    let mut base_url = None;

    for attr in attrs {
        if attr.path().is_ident("baml") {
            if let Ok(meta_list) = attr.meta.require_list() {
                // Parse comma-separated name-value pairs
                let nested = meta_list.tokens.clone();
                let parser = syn::meta::parser(|meta| {
                    if meta.path.is_ident("provider") {
                        let value: syn::LitStr = meta.value()?.parse()?;
                        provider = Some(value.value());
                        Ok(())
                    } else if meta.path.is_ident("model") {
                        let value: syn::LitStr = meta.value()?.parse()?;
                        model = Some(value.value());
                        Ok(())
                    } else if meta.path.is_ident("base_url") {
                        let value: syn::LitStr = meta.value()?.parse()?;
                        base_url = Some(value.value());
                        Ok(())
                    } else {
                        Err(meta.error("unsupported attribute"))
                    }
                });

                if let Err(e) = syn::parse::Parser::parse2(parser, nested) {
                    panic!("Failed to parse baml attributes: {}", e);
                }
            }
        }
    }

    let provider = provider.expect("#[baml(...)] requires 'provider' attribute");
    let model = model.expect("#[baml(...)] requires 'model' attribute");

    (provider, model, base_url)
}
