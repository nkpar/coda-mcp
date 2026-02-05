use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Formula {
    pub id: String,
    #[serde(rename = "type")]
    pub formula_type: Option<String>,
    pub href: Option<String>,
    pub name: String,
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaList {
    pub items: Vec<Formula>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListFormulasParams {
    /// The document ID
    pub doc_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFormulaParams {
    /// The document ID
    pub doc_id: String,
    /// The formula ID or name
    pub formula_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formula_deserialize_string_value() {
        let json = r#"{
            "id": "f-abc",
            "type": "formula",
            "name": "TotalCount",
            "value": "42"
        }"#;

        let formula: Formula = serde_json::from_str(json).unwrap();
        assert_eq!(formula.id, "f-abc");
        assert_eq!(formula.name, "TotalCount");
        assert_eq!(formula.value.unwrap(), "42");
    }

    #[test]
    fn test_formula_deserialize_number_value() {
        let json = r#"{
            "id": "f-num",
            "name": "Sum",
            "value": 123.45
        }"#;

        let formula: Formula = serde_json::from_str(json).unwrap();
        assert_eq!(formula.value.unwrap(), 123.45);
    }

    #[test]
    fn test_formula_deserialize_object_value() {
        let json = r#"{
            "id": "f-obj",
            "name": "Complex",
            "value": {"count": 10, "items": ["a", "b"]}
        }"#;

        let formula: Formula = serde_json::from_str(json).unwrap();
        let val = formula.value.unwrap();
        assert!(val.is_object());
    }

    #[test]
    fn test_formula_list_deserialize() {
        let json = r#"{
            "items": [
                {"id": "f1", "name": "Formula1", "value": 1},
                {"id": "f2", "name": "Formula2", "value": 2}
            ]
        }"#;

        let list: FormulaList = serde_json::from_str(json).unwrap();
        assert_eq!(list.items.len(), 2);
    }
}
