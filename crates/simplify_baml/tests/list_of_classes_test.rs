use simplify_baml::*;

#[test]
fn test_parse_list_of_people() {
    // Create IR with People class
    let mut ir = IR::new();

    ir.classes.push(Class {
        name: "People".to_string(),
        description: None,
        fields: vec![
            Field {
                name: "name".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: None,
            },
        ],
    });

    // The LLM response from the user's error message
    let raw_response = r#"[
  {
    "name": "Regina"
  },
  {
    "name": "Guaguaxuan"
  }
]"#;

    // Try to parse it with List<People> type
    let target_type = FieldType::List(Box::new(FieldType::Class("People".to_string())));

    eprintln!("Raw response:\n{}", raw_response);
    eprintln!("Target type: {:?}", target_type);

    let result = parse_llm_response_with_ir(&ir, raw_response, &target_type);

    match result {
        Ok(baml_value) => {
            println!("✅ Successfully parsed: {:?}", baml_value);
            // Verify it's a list
            if let BamlValue::List(items) = baml_value {
                assert_eq!(items.len(), 2);
                // Verify first item
                if let BamlValue::Map(map) = &items[0] {
                    if let Some(BamlValue::String(name)) = map.get("name") {
                        assert_eq!(name, "Regina");
                    } else {
                        panic!("Expected name field to be string");
                    }
                }
                // Verify second item
                if let BamlValue::Map(map) = &items[1] {
                    if let Some(BamlValue::String(name)) = map.get("name") {
                        assert_eq!(name, "Guaguaxuan");
                    } else {
                        panic!("Expected name field to be string");
                    }
                }
            } else {
                panic!("Expected List, got {:?}", baml_value);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to parse!");
            eprintln!("Error: {:#}", e);
            panic!("Failed to parse: {}", e);
        }
    }
}
