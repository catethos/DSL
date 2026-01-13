//! Type schema and function signature conversion for Neon bindings

use neon::prelude::*;

use lattice::runtime::{FieldSchema, FunctionSignature, ParameterSchema, TypeSchema};

/// Convert TypeSchema to JavaScript object
pub fn type_schema_to_js<'a>(
    cx: &mut impl Context<'a>,
    schema: &TypeSchema,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();

    match schema {
        TypeSchema::Null => {
            let kind = cx.string("null");
            obj.set(cx, "kind", kind)?;
        }
        TypeSchema::Int => {
            let kind = cx.string("int");
            obj.set(cx, "kind", kind)?;
        }
        TypeSchema::Float => {
            let kind = cx.string("float");
            obj.set(cx, "kind", kind)?;
        }
        TypeSchema::String => {
            let kind = cx.string("string");
            obj.set(cx, "kind", kind)?;
        }
        TypeSchema::Bool => {
            let kind = cx.string("bool");
            obj.set(cx, "kind", kind)?;
        }
        TypeSchema::Path => {
            let kind = cx.string("path");
            obj.set(cx, "kind", kind)?;
        }
        TypeSchema::Any => {
            let kind = cx.string("any");
            obj.set(cx, "kind", kind)?;
        }
        TypeSchema::List(inner) => {
            let kind = cx.string("list");
            obj.set(cx, "kind", kind)?;
            let inner_schema = type_schema_to_js(cx, inner)?;
            obj.set(cx, "inner", inner_schema)?;
        }
        TypeSchema::Map { key, value } => {
            let kind = cx.string("map");
            obj.set(cx, "kind", kind)?;
            let key_schema = type_schema_to_js(cx, key)?;
            obj.set(cx, "key", key_schema)?;
            let value_schema = type_schema_to_js(cx, value)?;
            obj.set(cx, "value", value_schema)?;
        }
        TypeSchema::Optional(inner) => {
            let kind = cx.string("optional");
            obj.set(cx, "kind", kind)?;
            let inner_schema = type_schema_to_js(cx, inner)?;
            obj.set(cx, "inner", inner_schema)?;
        }
        TypeSchema::Struct(s) => {
            let kind = cx.string("struct");
            obj.set(cx, "kind", kind)?;
            let name = cx.string(&s.name);
            obj.set(cx, "name", name)?;

            let fields_arr = cx.empty_array();
            for (i, field) in s.fields.iter().enumerate() {
                let field_obj = field_schema_to_js(cx, field)?;
                fields_arr.set(cx, i as u32, field_obj)?;
            }
            obj.set(cx, "fields", fields_arr)?;

            if let Some(desc) = &s.description {
                let description = cx.string(desc);
                obj.set(cx, "description", description)?;
            }
        }
        TypeSchema::Enum(e) => {
            let kind = cx.string("enum");
            obj.set(cx, "kind", kind)?;
            let name = cx.string(&e.name);
            obj.set(cx, "name", name)?;

            let variants_arr = cx.empty_array();
            for (i, variant) in e.variants.iter().enumerate() {
                let variant_str = cx.string(variant);
                variants_arr.set(cx, i as u32, variant_str)?;
            }
            obj.set(cx, "variants", variants_arr)?;
        }
        TypeSchema::Named(name) => {
            let kind = cx.string("named");
            obj.set(cx, "kind", kind)?;
            let name_str = cx.string(name);
            obj.set(cx, "name", name_str)?;
        }
    }

    Ok(obj)
}

fn field_schema_to_js<'a>(
    cx: &mut impl Context<'a>,
    field: &FieldSchema,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();

    let name = cx.string(&field.name);
    obj.set(cx, "name", name)?;

    let type_schema = type_schema_to_js(cx, &field.type_schema)?;
    obj.set(cx, "type_schema", type_schema)?;

    let optional = cx.boolean(field.optional);
    obj.set(cx, "optional", optional)?;

    if let Some(desc) = &field.description {
        let description = cx.string(desc);
        obj.set(cx, "description", description)?;
    }

    Ok(obj)
}

/// Convert FunctionSignature to JavaScript object
pub fn function_signature_to_js<'a>(
    cx: &mut impl Context<'a>,
    sig: &FunctionSignature,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();

    let name = cx.string(&sig.name);
    obj.set(cx, "name", name)?;

    let is_llm = cx.boolean(sig.is_llm);
    obj.set(cx, "is_llm", is_llm)?;

    let is_async = cx.boolean(sig.is_async);
    obj.set(cx, "is_async", is_async)?;

    let return_type = type_schema_to_js(cx, &sig.return_type)?;
    obj.set(cx, "return_type", return_type)?;

    let params_arr = cx.empty_array();
    for (i, param) in sig.params.iter().enumerate() {
        let param_obj = param_schema_to_js(cx, param)?;
        params_arr.set(cx, i as u32, param_obj)?;
    }
    obj.set(cx, "params", params_arr)?;

    Ok(obj)
}

fn param_schema_to_js<'a>(
    cx: &mut impl Context<'a>,
    param: &ParameterSchema,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();

    let name = cx.string(&param.name);
    obj.set(cx, "name", name)?;

    let type_schema = type_schema_to_js(cx, &param.type_schema)?;
    obj.set(cx, "type_schema", type_schema)?;

    Ok(obj)
}
