use super::{DslParser, Rule};
use pest::iterators::Pair;
use pest::Parser;
use dsl_types::{Class, Enum, Field, FieldType};

/// Parse a type definition
pub fn parse_type_definition(input: &str) -> Result<Class, String> {
    let pairs =
        DslParser::parse(Rule::type_decl, input).map_err(|e| format!("Parse error: {}", e))?;

    let pair = pairs
        .into_iter()
        .next()
        .ok_or_else(|| "No type definition found".to_string())?;

    build_type_definition(pair)
}

pub(super) fn build_type_definition(pair: Pair<Rule>) -> Result<Class, String> {
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or("Missing type name")?
        .as_str()
        .to_string();

    let mut fields = Vec::new();

    for field_pair in inner {
        if field_pair.as_rule() == Rule::field {
            fields.push(build_field(field_pair)?);
        }
    }

    Ok(Class {
        name,
        description: None,
        fields,
    })
}

fn build_field(pair: Pair<Rule>) -> Result<Field, String> {
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or("Missing field name")?
        .as_str()
        .to_string();

    // Check for optional marker
    let mut optional = false;
    let mut next = inner.next().ok_or("Missing field type")?;

    if next.as_rule() == Rule::optional {
        optional = true;
        next = inner
            .next()
            .ok_or("Missing field type after optional marker")?;
    }

    let field_type = build_field_type(next)?;

    // Check for description (optional)
    let description = inner.next().map(|p| {
        let desc_inner = p.into_inner().next().unwrap();
        desc_inner.as_str().to_string()
    });

    Ok(Field {
        name,
        field_type,
        optional,
        description,
    })
}

pub(super) fn build_field_type(pair: Pair<Rule>) -> Result<FieldType, String> {
    let inner = pair.into_inner().next().ok_or("Empty field type")?;

    match inner.as_rule() {
        Rule::primitive_type => match inner.as_str().to_lowercase().as_str() {
            "string" => Ok(FieldType::String),
            "int" => Ok(FieldType::Int),
            "float" => Ok(FieldType::Float),
            "bool" => Ok(FieldType::Bool),
            _ => Err(format!("Unknown primitive type: {}", inner.as_str())),
        },
        Rule::list_type => {
            let inner_type = inner.into_inner().next().ok_or("Missing list inner type")?;
            let element_type = build_field_type(inner_type)?;
            Ok(FieldType::List(Box::new(element_type)))
        }
        Rule::custom_type => {
            let type_name = inner.as_str().to_string();
            Ok(FieldType::Class(type_name))
        }
        _ => Err(format!("Unexpected field type rule: {:?}", inner.as_rule())),
    }
}

/// Parse an enum definition
pub fn parse_enum_definition(input: &str) -> Result<Enum, String> {
    let pairs =
        DslParser::parse(Rule::enum_decl, input).map_err(|e| format!("Parse error: {}", e))?;

    let pair = pairs
        .into_iter()
        .next()
        .ok_or_else(|| "No enum definition found".to_string())?;

    build_enum_definition(pair)
}

pub(super) fn build_enum_definition(pair: Pair<Rule>) -> Result<Enum, String> {
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or("Missing enum name")?
        .as_str()
        .to_string();

    let mut values = Vec::new();

    for variant_pair in inner {
        if variant_pair.as_rule() == Rule::enum_variant {
            let variant_name = variant_pair
                .into_inner()
                .next()
                .ok_or("Missing variant name")?
                .as_str()
                .to_string();
            values.push(variant_name);
        }
    }

    if values.is_empty() {
        return Err("Enum must have at least one value".to_string());
    }

    Ok(Enum {
        name,
        description: None,
        values,
    })
}
