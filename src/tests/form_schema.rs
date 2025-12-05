use crate::capabilities::form::{Field, FieldType, FormSchema};

#[test]
fn form_schema_with_all_field_types() {
    let schema = FormSchema {
        title: "Complete Form".to_string(),
        description: Some("A form with all field types".to_string()),
        fields: vec![
            Field {
                id: "text_field".to_string(),
                label: "Text Input".to_string(),
                field_type: FieldType::Text,
                placeholder: Some("Enter text".to_string()),
                regex: None,
                help: Some("A text field".to_string()),
                value: None,
            },
            Field {
                id: "password_field".to_string(),
                label: "Password".to_string(),
                field_type: FieldType::Password,
                placeholder: Some("Enter password".to_string()),
                regex: Some(r"^.{8,}$".to_string()),
                help: Some("At least 8 characters".to_string()),
                value: None,
            },
            Field {
                id: "select_field".to_string(),
                label: "Select Option".to_string(),
                field_type: FieldType::Select(vec![
                    "Option A".to_string(),
                    "Option B".to_string(),
                    "Option C".to_string(),
                ]),
                placeholder: None,
                regex: None,
                help: Some("Choose one".to_string()),
                value: None,
            },
            Field {
                id: "info_field".to_string(),
                label: "Information".to_string(),
                field_type: FieldType::MarkdownInfo,
                placeholder: None,
                regex: None,
                help: None,
                value: None,
            },
        ],
    };

    assert_eq!(schema.fields.len(), 4);
    assert_eq!(schema.title, "Complete Form");
}

#[test]
fn field_type_text_serialization() {
    let field_type = FieldType::Text;
    let json = serde_json::to_string(&field_type).expect("Should serialize");
    let deserialized: FieldType = serde_json::from_str(&json).expect("Should deserialize");

    match deserialized {
        FieldType::Text => {}
        _ => panic!("Incorrect deserialization"),
    }
}

#[test]
fn field_type_password_serialization() {
    let field_type = FieldType::Password;
    let json = serde_json::to_string(&field_type).expect("Should serialize");
    let deserialized: FieldType = serde_json::from_str(&json).expect("Should deserialize");

    match deserialized {
        FieldType::Password => {}
        _ => panic!("Incorrect deserialization"),
    }
}

#[test]
fn field_type_select_with_options() {
    let options = vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()];
    let field_type = FieldType::Select(options.clone());

    match field_type {
        FieldType::Select(opts) => {
            assert_eq!(opts.len(), 3);
            assert_eq!(opts[0], "Red");
            assert_eq!(opts[1], "Green");
            assert_eq!(opts[2], "Blue");
        }
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn field_type_select_empty_options() {
    let field_type = FieldType::Select(vec![]);

    match field_type {
        FieldType::Select(opts) => assert_eq!(opts.len(), 0),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn field_type_markdown_info() {
    let field_type = FieldType::MarkdownInfo;
    let json = serde_json::to_string(&field_type).expect("Should serialize");
    let deserialized: FieldType = serde_json::from_str(&json).expect("Should deserialize");

    match deserialized {
        FieldType::MarkdownInfo => {}
        _ => panic!("Incorrect deserialization"),
    }
}

#[test]
fn field_with_regex_validation() {
    let field = Field {
        id: "email".to_string(),
        label: "Email Address".to_string(),
        field_type: FieldType::Text,
        placeholder: Some("user@example.com".to_string()),
        regex: Some(r"^[^\s@]+@[^\s@]+\.[^\s@]+$".to_string()),
        help: Some("Enter a valid email".to_string()),
        value: None,
    };

    assert_eq!(field.id, "email");
    assert!(field.regex.is_some());

    let regex_pattern = field.regex.unwrap();
    assert!(regex_pattern.contains("@"));
}

#[test]
fn field_serialization_roundtrip() {
    let field = Field {
        id: "username".to_string(),
        label: "Username".to_string(),
        field_type: FieldType::Text,
        placeholder: Some("Enter username".to_string()),
        regex: Some(r"^\w{3,20}$".to_string()),
        help: Some("3-20 characters".to_string()),
        value: None,
    };

    let json = serde_json::to_string(&field).expect("Should serialize");
    let deserialized: Field = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.id, "username");
    assert_eq!(deserialized.label, "Username");
    assert_eq!(deserialized.placeholder, Some("Enter username".to_string()));
    assert_eq!(deserialized.regex, Some(r"^\w{3,20}$".to_string()));
    assert_eq!(deserialized.help, Some("3-20 characters".to_string()));
}

#[test]
fn form_schema_serialization_roundtrip() {
    let schema = FormSchema {
        title: "Registration Form".to_string(),
        description: Some("Please fill out all fields".to_string()),
        fields: vec![
            Field {
                id: "name".to_string(),
                label: "Full Name".to_string(),
                field_type: FieldType::Text,
                placeholder: Some("John Doe".to_string()),
                regex: None,
                help: None,
                value: None,
            },
            Field {
                id: "password".to_string(),
                label: "Password".to_string(),
                field_type: FieldType::Password,
                placeholder: None,
                regex: Some(r"^.{8,}$".to_string()),
                help: Some("Minimum 8 characters".to_string()),
                value: None,
            },
        ],
    };

    let json = serde_json::to_string(&schema).expect("Should serialize");
    let deserialized: FormSchema = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.title, "Registration Form");
    assert_eq!(
        deserialized.description,
        Some("Please fill out all fields".to_string())
    );
    assert_eq!(deserialized.fields.len(), 2);
    assert_eq!(deserialized.fields[0].id, "name");
    assert_eq!(deserialized.fields[1].id, "password");
}

#[test]
fn form_schema_minimal() {
    let schema = FormSchema {
        title: "Minimal".to_string(),
        description: None,
        fields: vec![],
    };

    assert_eq!(schema.title, "Minimal");
    assert!(schema.description.is_none());
    assert!(schema.fields.is_empty());
}

#[test]
fn field_clone() {
    let field = Field {
        id: "test".to_string(),
        label: "Test".to_string(),
        field_type: FieldType::Text,
        placeholder: Some("placeholder".to_string()),
        regex: Some("regex".to_string()),
        help: Some("help".to_string()),
        value: None,
    };

    let cloned = field.clone();
    assert_eq!(field.id, cloned.id);
    assert_eq!(field.label, cloned.label);
    assert_eq!(field.placeholder, cloned.placeholder);
}

#[test]
fn form_schema_clone() {
    let schema = FormSchema {
        title: "Test".to_string(),
        description: Some("Description".to_string()),
        fields: vec![],
    };

    let cloned = schema.clone();
    assert_eq!(schema.title, cloned.title);
    assert_eq!(schema.description, cloned.description);
    assert_eq!(schema.fields.len(), cloned.fields.len());
}

#[test]
fn field_type_clone() {
    let types = vec![
        FieldType::Text,
        FieldType::Password,
        FieldType::Select(vec!["a".to_string()]),
        FieldType::MarkdownInfo,
    ];

    for ft in types {
        let cloned = ft.clone();
        // Just verify clone works without panicking
        let _ = format!("{:?}", cloned);
    }
}

#[test]
fn field_debug_output() {
    let field = Field {
        id: "debug_test".to_string(),
        label: "Debug Test".to_string(),
        field_type: FieldType::Text,
        placeholder: None,
        regex: None,
        help: None,
        value: None,
    };

    let debug_str = format!("{:?}", field);
    assert!(debug_str.contains("debug_test"));
    assert!(debug_str.contains("Debug Test"));
}

#[test]
fn form_schema_debug_output() {
    let schema = FormSchema {
        title: "Debug Form".to_string(),
        description: Some("For debugging".to_string()),
        fields: vec![],
    };

    let debug_str = format!("{:?}", schema);
    assert!(debug_str.contains("Debug Form"));
    assert!(debug_str.contains("For debugging"));
}

#[test]
fn field_type_select_large_option_list() {
    let options: Vec<String> = (0..100).map(|i| format!("Option {}", i)).collect();
    let field_type = FieldType::Select(options.clone());

    match field_type {
        FieldType::Select(opts) => {
            assert_eq!(opts.len(), 100);
            assert_eq!(opts[0], "Option 0");
            assert_eq!(opts[99], "Option 99");
        }
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn field_with_complex_regex() {
    let field = Field {
        id: "complex".to_string(),
        label: "Complex Validation".to_string(),
        field_type: FieldType::Text,
        placeholder: None,
        regex: Some(
            r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$".to_string(),
        ),
        help: Some(
            "Password must contain uppercase, lowercase, number, and special character".to_string(),
        ),
        value: None,
    };

    assert!(field.regex.is_some());
    let regex = field.regex.unwrap();
    assert!(regex.len() > 20);
}
