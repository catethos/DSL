/// Example: Nested Structures with Macros
///
/// This example demonstrates how the macro system handles complex nested structures.
/// Compare this with `nested_parsing.rs` which manually builds ~90 lines of IR!
///
/// To run this example:
/// ```bash
/// cargo run --example nested_macro
/// ```

use simplify_baml::*;
use simplify_baml_macros::BamlSchema;

// Define complex nested types using macros - so much cleaner!

#[derive(BamlSchema)]
enum Role {
    #[allow(dead_code)]
    engineer,
    #[allow(dead_code)]
    manager,
    #[allow(dead_code)]
    designer,
}

#[derive(BamlSchema)]
struct Address {
    #[allow(dead_code)]
    street: String,
    #[allow(dead_code)]
    city: String,
    #[allow(dead_code)]
    #[baml(rename = "zipCode")]
    zip_code: String,
}

#[derive(BamlSchema)]
struct Employee {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    age: i64,
    #[allow(dead_code)]
    role: Role,
}

#[derive(BamlSchema)]
struct Company {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    employees: Vec<Employee>,
    #[allow(dead_code)]
    address: Address,
}

fn main() {
    println!("=== Nested Structures with Macros ===\n");

    // Build IR with automatic nesting resolution
    let ir = BamlSchemaRegistry::new()
        .register::<Role>()
        .register::<Address>()
        .register::<Employee>()
        .register::<Company>()
        .build();

    println!("✅ Generated IR with {} classes and {} enums:",
        ir.classes.len(),
        ir.enums.len()
    );
    println!();

    // Show what was generated
    for enum_def in &ir.enums {
        println!("📋 Enum: {}", enum_def.name);
        println!("   Variants: {}", enum_def.values.join(", "));
        println!();
    }

    for class in &ir.classes {
        println!("📦 Class: {}", class.name);
        for field in &class.fields {
            let optional = if field.optional { " (optional)" } else { "" };
            println!("   - {}: {}{}",
                field.name,
                field.field_type.to_string(),
                optional
            );
        }
        println!();
    }

    // Test parsing with the generated IR
    let parser = simplify_baml::parser::Parser::new(&ir);

    let test_json = r#"
    {
        "name": "TechCorp",
        "employees": [
            {
                "name": "Alice",
                "age": 30,
                "role": "engineer"
            },
            {
                "name": "Bob",
                "age": 35,
                "role": "manager"
            }
        ],
        "address": {
            "street": "123 Main St",
            "city": "San Francisco",
            "zipCode": "94102"
        }
    }
    "#;

    match parser.parse(test_json, &FieldType::Class("Company".to_string())) {
        Ok(result) => {
            println!("✅ Successfully parsed nested JSON!");
            println!("{:#?}", result);
        }
        Err(e) => {
            println!("❌ Parse error: {}", e);
        }
    }

    println!("\n=== Code Comparison ===");
    println!("Manual IR building (nested_parsing.rs): ~90 lines");
    println!("With macros (this file): ~40 lines of definitions");
    println!("Reduction: ~55% less code!");
    println!("\n✨ The macro handles:");
    println!("  • Nested structs (Company contains Address)");
    println!("  • Lists of structs (Vec<Employee>)");
    println!("  • Enum fields (Role in Employee)");
    println!("  • Field renaming (zip_code → zipCode)");
}
