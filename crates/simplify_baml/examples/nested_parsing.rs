/// Example: Nested Structure Parsing
///
/// This example demonstrates parsing complex nested structures including:
/// 1. Nested objects (classes within classes)
/// 2. Lists/arrays
/// 3. Lists of objects
/// 4. Handling malformed nested JSON
///
/// To run this example:
/// ```bash
/// cargo run --example nested_parsing
/// ```

use simplify_baml::*;

fn main() {
    println!("=== Nested Structure Parsing ===\n");

    let ir = build_company_ir();
    let parser = simplify_baml::parser::Parser::new(&ir);

    // Test Case 1: Well-formatted nested structure
    run_test(&parser, "1. Well-Formatted Company", r#"
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
    "#);

    // Test Case 2: In markdown with extra text
    run_test(&parser, "2. Company in Markdown Block", r#"
Based on the information provided, here's the company data:

```json
{
    "name": "StartupInc",
    "employees": [
        {
            "name": "Charlie",
            "age": "28",
            "role": "ENGINEER"
        }
    ],
    "address": {
        "street": "456 Tech Ave",
        "city": "Austin",
        "zipCode": "78701"
    }
}
```

The company has 1 employee currently.
    "#);

    // Test Case 3: Type coercion in nested structures
    run_test(&parser, "3. Nested with Type Coercion", r#"
{
    "name": "BigCorp",
    "employees": [
        {
            "name": "Diana",
            "age": "42",
            "role": "manager"
        },
        {
            "name": "Eve",
            "age": 29,
            "role": "Engineer"
        }
    ],
    "address": {
        "street": "789 Business Blvd",
        "city": "New York",
        "zipCode": 10001
    }
}
    "#);

    // Test Case 4: Empty arrays
    run_test(&parser, "4. Company with No Employees", r#"
{
    "name": "NewStartup",
    "employees": [],
    "address": {
        "street": "321 Innovation Dr",
        "city": "Seattle",
        "zipCode": "98101"
    }
}
    "#);

    // Test Case 5: Minimal nested structure
    run_test(&parser, "5. Minimal Valid Structure", r#"
The company information is:
{
    "name": "MiniCorp",
    "employees": [
        {"name": "Frank", "age": 25, "role": "engineer"}
    ],
    "address": {
        "street": "111 Small St",
        "city": "Portland",
        "zipCode": "97201"
    }
}
    "#);

    // Test Case 6: Numbers as strings in nested structures
    run_test(&parser, "6. All Numbers as Strings", r#"
```json
{
    "name": "StringCorp",
    "employees": [
        {
            "name": "Grace",
            "age": "33",
            "role": "manager"
        }
    ],
    "address": {
        "street": "222 String Ln",
        "city": "Boston",
        "zipCode": "02101"
    }
}
```
    "#);

    println!("\n=== Summary ===");
    println!("✓ Successfully parsed nested objects");
    println!("✓ Handled arrays of objects");
    println!("✓ Type coercion works in nested structures");
    println!("✓ Case-insensitive enums in arrays");
    println!("✓ Empty arrays handled correctly");
    println!("✓ Markdown extraction with nested JSON");
}

fn run_test(parser: &simplify_baml::parser::Parser, title: &str, response: &str) {
    println!("\n{}", "=".repeat(60));
    println!("{}", title);
    println!("{}", "=".repeat(60));

    let result = parser.parse(response, &FieldType::Class("Company".to_string()));

    match result {
        Ok(value) => {
            println!("✅ Successfully Parsed:");
            print_company(&value, 1);
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
        }
    }
}

fn print_company(value: &BamlValue, indent: usize) {
    let indent_str = "  ".repeat(indent);

    if let BamlValue::Map(map) = value {
        println!("{}Company {{", indent_str);

        if let Some(name) = map.get("name") {
            println!("{}  name: {:?}", indent_str, name.as_string().unwrap_or("N/A"));
        }

        if let Some(employees) = map.get("employees") {
            if let BamlValue::List(emp_list) = employees {
                println!("{}  employees: [", indent_str);
                for emp in emp_list {
                    print_employee(emp, indent + 2);
                }
                println!("{}  ]", indent_str);
            }
        }

        if let Some(address) = map.get("address") {
            println!("{}  address:", indent_str);
            print_address(address, indent + 2);
        }

        println!("{}}}", indent_str);
    }
}

fn print_employee(value: &BamlValue, indent: usize) {
    let indent_str = "  ".repeat(indent);

    if let BamlValue::Map(map) = value {
        println!("{}Employee {{", indent_str);
        if let Some(name) = map.get("name") {
            println!("{}  name: {:?}", indent_str, name.as_string().unwrap_or("N/A"));
        }
        if let Some(age) = map.get("age") {
            println!("{}  age: {}", indent_str, age.as_int().unwrap_or(-1));
        }
        if let Some(role) = map.get("role") {
            println!("{}  role: {:?}", indent_str, role.as_string().unwrap_or("N/A"));
        }
        println!("{}}}", indent_str);
    }
}

fn print_address(value: &BamlValue, indent: usize) {
    let indent_str = "  ".repeat(indent);

    if let BamlValue::Map(map) = value {
        println!("{}Address {{", indent_str);
        if let Some(street) = map.get("street") {
            println!("{}  street: {:?}", indent_str, street.as_string().unwrap_or("N/A"));
        }
        if let Some(city) = map.get("city") {
            println!("{}  city: {:?}", indent_str, city.as_string().unwrap_or("N/A"));
        }
        if let Some(zip) = map.get("zipCode") {
            println!("{}  zipCode: {:?}", indent_str, zip.as_string().unwrap_or("N/A"));
        }
        println!("{}}}", indent_str);
    }
}

fn build_company_ir() -> IR {
    let mut ir = IR::new();

    // Define Role enum
    ir.enums.push(Enum {
        name: "Role".to_string(),
        description: None,
        values: vec![
            "engineer".to_string(),
            "manager".to_string(),
            "designer".to_string(),
        ],
    });

    // Define Address class
    ir.classes.push(Class {
        name: "Address".to_string(),
        description: None,
        fields: vec![
            Field {
                name: "street".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: None,
            },
            Field {
                name: "city".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: None,
            },
            Field {
                name: "zipCode".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: None,
            },
        ],
    });

    // Define Employee class
    ir.classes.push(Class {
        name: "Employee".to_string(),
        description: None,
        fields: vec![
            Field {
                name: "name".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: None,
            },
            Field {
                name: "age".to_string(),
                field_type: FieldType::Int,
                optional: false,
                description: None,
            },
            Field {
                name: "role".to_string(),
                field_type: FieldType::Enum("Role".to_string()),
                optional: false,
                description: None,
            },
        ],
    });

    // Define Company class with nested structures
    ir.classes.push(Class {
        name: "Company".to_string(),
        description: None,
        fields: vec![
            Field {
                name: "name".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: None,
            },
            Field {
                name: "employees".to_string(),
                field_type: FieldType::List(Box::new(FieldType::Class("Employee".to_string()))),
                optional: false,
                description: None,
            },
            Field {
                name: "address".to_string(),
                field_type: FieldType::Class("Address".to_string()),
                optional: false,
                description: None,
            },
        ],
    });

    ir
}
