use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum FieldType {
    Text,
    Password,
    Select(Vec<String>),
    MarkdownInfo
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Field {
    pub id: String,
    pub label: String,
    pub field_type: Option<FieldType>,
    pub placeholder: Option<String>,
    pub regex: Option<String>,
    pub help: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct FormSchema {
    pub title: String,
    pub description: Option<String>,
    pub fields: Vec<Field>
}
