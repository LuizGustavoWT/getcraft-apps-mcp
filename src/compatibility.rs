use serde_json::{Value, json};

pub fn for_app(app: &str) -> Value {
    match serde_json::from_str::<Value>(include_str!("../upstream-versions.json")) {
        Ok(snapshot) => {
            let mut audit = snapshot["apps"][app].clone();
            if let Some(fields) = audit.as_object_mut() {
                fields.insert("checked_on".to_string(), snapshot["checked_on"].clone());
            }
            audit
        }
        Err(error) => json!({"error": format!("invalid bundled upstream snapshot: {error}")}),
    }
}
