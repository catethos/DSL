/// Example: Using Descriptions in IR
///
/// This example demonstrates how to add descriptions to:
/// 1. Classes
/// 2. Fields
/// 3. Enums
///
/// Descriptions are rendered in the schema sent to the LLM, providing context
/// that helps the model understand what to extract or generate.
///
/// To run this example:
/// ```bash
/// cargo run --example with_descriptions
/// ```

use simplify_baml::*;
use simplify_baml::schema::SchemaFormatter;

fn main() {
    println!("=== Descriptions in Schema Generation ===\n");

    // Build IR with rich descriptions
    let ir = build_documented_ir();

    // Create a schema formatter
    let mut formatter = SchemaFormatter::new(&ir);

    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Schema with Descriptions                                    │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    // Render schema for Resume class
    let schema = formatter.render(&FieldType::Class("Resume".to_string()));
    println!("{}", schema);

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Benefits of Descriptions                                    │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!("\n✅ Help LLMs understand the expected data");
    println!("✅ Provide context for extraction tasks");
    println!("✅ Clarify ambiguous field names");
    println!("✅ Specify formats or constraints");
    println!("✅ Serve as documentation for your schema");

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│ Example: How to Define Descriptions in Code                │");
    println!("└─────────────────────────────────────────────────────────────┘\n");

    print_code_example();
}

fn build_documented_ir() -> IR {
    let mut ir = IR::new();

    // Define EmploymentType enum with description
    ir.enums.push(Enum {
        name: "EmploymentType".to_string(),
        description: Some("Type of employment arrangement".to_string()),
        values: vec![
            "full-time".to_string(),
            "part-time".to_string(),
            "contract".to_string(),
            "internship".to_string(),
        ],
    });

    // Define SkillLevel enum with description
    ir.enums.push(Enum {
        name: "SkillLevel".to_string(),
        description: Some("Proficiency level for a skill".to_string()),
        values: vec![
            "beginner".to_string(),
            "intermediate".to_string(),
            "advanced".to_string(),
            "expert".to_string(),
        ],
    });

    // Define Skill class with descriptions
    ir.classes.push(Class {
        name: "Skill".to_string(),
        description: Some("A technical or professional skill".to_string()),
        fields: vec![
            Field {
                name: "name".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Name of the skill (e.g., 'Python', 'Project Management')".to_string()),
            },
            Field {
                name: "level".to_string(),
                field_type: FieldType::Enum("SkillLevel".to_string()),
                optional: false,
                description: Some("Self-assessed proficiency level".to_string()),
            },
            Field {
                name: "yearsOfExperience".to_string(),
                field_type: FieldType::Int,
                optional: true,
                description: Some("Number of years using this skill".to_string()),
            },
        ],
    });

    // Define Education class with descriptions
    ir.classes.push(Class {
        name: "Education".to_string(),
        description: Some("Educational background entry".to_string()),
        fields: vec![
            Field {
                name: "institution".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Name of the school, college, or university".to_string()),
            },
            Field {
                name: "degree".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Degree or certification obtained".to_string()),
            },
            Field {
                name: "fieldOfStudy".to_string(),
                field_type: FieldType::String,
                optional: true,
                description: Some("Major or area of study".to_string()),
            },
            Field {
                name: "graduationYear".to_string(),
                field_type: FieldType::Int,
                optional: true,
                description: Some("Year of graduation (YYYY format)".to_string()),
            },
        ],
    });

    // Define WorkExperience class with descriptions
    ir.classes.push(Class {
        name: "WorkExperience".to_string(),
        description: Some("Professional work experience entry".to_string()),
        fields: vec![
            Field {
                name: "company".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Name of the employer or company".to_string()),
            },
            Field {
                name: "title".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Job title or position held".to_string()),
            },
            Field {
                name: "employmentType".to_string(),
                field_type: FieldType::Enum("EmploymentType".to_string()),
                optional: true,
                description: Some("Type of employment".to_string()),
            },
            Field {
                name: "startYear".to_string(),
                field_type: FieldType::Int,
                optional: false,
                description: Some("Year started (YYYY format)".to_string()),
            },
            Field {
                name: "endYear".to_string(),
                field_type: FieldType::Int,
                optional: true,
                description: Some("Year ended (YYYY format), omit if current position".to_string()),
            },
            Field {
                name: "description".to_string(),
                field_type: FieldType::String,
                optional: true,
                description: Some("Brief description of responsibilities and achievements".to_string()),
            },
        ],
    });

    // Define Resume class with descriptions
    ir.classes.push(Class {
        name: "Resume".to_string(),
        description: Some("Complete resume/CV information".to_string()),
        fields: vec![
            Field {
                name: "fullName".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Full name of the candidate".to_string()),
            },
            Field {
                name: "email".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Primary email address".to_string()),
            },
            Field {
                name: "phone".to_string(),
                field_type: FieldType::String,
                optional: true,
                description: Some("Contact phone number".to_string()),
            },
            Field {
                name: "summary".to_string(),
                field_type: FieldType::String,
                optional: true,
                description: Some("Professional summary or objective statement".to_string()),
            },
            Field {
                name: "skills".to_string(),
                field_type: FieldType::List(Box::new(FieldType::Class("Skill".to_string()))),
                optional: false,
                description: Some("List of professional skills".to_string()),
            },
            Field {
                name: "education".to_string(),
                field_type: FieldType::List(Box::new(FieldType::Class("Education".to_string()))),
                optional: false,
                description: Some("Educational background".to_string()),
            },
            Field {
                name: "workExperience".to_string(),
                field_type: FieldType::List(Box::new(FieldType::Class("WorkExperience".to_string()))),
                optional: false,
                description: Some("Professional work history".to_string()),
            },
        ],
    });

    ir
}

fn print_code_example() {
    println!(r#"
// Example: Creating a class with descriptions

ir.classes.push(Class {{
    name: "Person".to_string(),
    description: Some("A person entity with contact information".to_string()),
    fields: vec![
        Field {{
            name: "email".to_string(),
            field_type: FieldType::String,
            optional: false,
            description: Some("Primary email address (must be valid format)".to_string()),
        }},
        Field {{
            name: "age".to_string(),
            field_type: FieldType::Int,
            optional: false,
            description: Some("Age in years (must be positive)".to_string()),
        }},
    ],
}});

// Example: Creating an enum with descriptions

ir.enums.push(Enum {{
    name: "Status".to_string(),
    description: Some("Current status of the entity".to_string()),
    values: vec![
        "active".to_string(),
        "inactive".to_string(),
    ],
}});
"#);
}
