use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Control {
    pub id: String,
    #[serde(rename = "type")]
    pub control_type_field: Option<String>,
    pub href: Option<String>,
    pub name: String,
    #[serde(rename = "controlType")]
    pub control_type: Option<String>,
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlList {
    pub items: Vec<Control>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListControlsParams {
    /// The document ID
    pub doc_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_deserialize_button() {
        let json = r#"{
            "id": "ctrl-btn",
            "type": "control",
            "name": "Submit Button",
            "controlType": "button"
        }"#;

        let ctrl: Control = serde_json::from_str(json).unwrap();
        assert_eq!(ctrl.id, "ctrl-btn");
        assert_eq!(ctrl.name, "Submit Button");
        assert_eq!(ctrl.control_type, Some("button".to_string()));
    }

    #[test]
    fn test_control_deserialize_slider() {
        let json = r#"{
            "id": "ctrl-slider",
            "name": "Progress",
            "controlType": "slider",
            "value": 75
        }"#;

        let ctrl: Control = serde_json::from_str(json).unwrap();
        assert_eq!(ctrl.control_type, Some("slider".to_string()));
        assert_eq!(ctrl.value.unwrap(), 75);
    }

    #[test]
    fn test_control_list_deserialize() {
        let json = r#"{
            "items": [
                {"id": "c1", "name": "Control 1", "controlType": "button"},
                {"id": "c2", "name": "Control 2", "controlType": "slider", "value": 50}
            ]
        }"#;

        let list: ControlList = serde_json::from_str(json).unwrap();
        assert_eq!(list.items.len(), 2);
    }
}
