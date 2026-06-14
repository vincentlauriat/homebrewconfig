use serde::Serialize;
use serde_json::Value;

use crate::app::{App, SettingKind};

/// Machine-readable view of a single setting.
#[derive(Serialize)]
struct SettingView {
    name: &'static str,
    env_var: &'static str,
    category: &'static str,
    kind: &'static str,
    value: Value,
    modified: bool,
    #[serde(skip_serializing_if = "str::is_empty")]
    default: &'static str,
}

/// Serialize the full current state as a pretty JSON array, one object per
/// setting, for piping into `jq` and other tooling.
pub fn state_json(app: &App) -> Result<String, String> {
    let views: Vec<SettingView> = app
        .settings
        .iter()
        .map(|s| {
            let (kind, value) = match &s.kind {
                SettingKind::Bool { .. } => ("bool", Value::Bool(s.bool_val)),
                SettingKind::Num => ("number", s.num_val.map(Value::from).unwrap_or(Value::Null)),
                SettingKind::Str => ("string", Value::String(s.str_val.clone())),
            };
            SettingView {
                name: s.name,
                env_var: s.env_var,
                category: s.category,
                kind,
                value,
                modified: s.modified,
                default: s.default_hint,
            }
        })
        .collect();

    serde_json::to_string_pretty(&views).map_err(|e| format!("failed to serialize state: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_json_is_valid_and_covers_all_settings() {
        let app = App::new();
        let json = state_json(&app).unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        let arr = parsed.as_array().unwrap();
        assert_eq!(arr.len(), app.settings.len());
    }

    #[test]
    fn state_json_types_values_by_kind() {
        let mut app = App::new();
        app.set_var("HOMEBREW_CLEANUP_MAX_AGE_DAYS", Some("30".to_string()))
            .unwrap();
        let json = state_json(&app).unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        let entry = parsed
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["env_var"] == "HOMEBREW_CLEANUP_MAX_AGE_DAYS")
            .unwrap();
        assert_eq!(entry["value"], Value::from(30u32));
        assert_eq!(entry["kind"], "number");
        assert_eq!(entry["modified"], Value::Bool(true));
    }

    #[test]
    fn state_json_bool_is_boolean() {
        let app = App::new();
        let json = state_json(&app).unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        let entry = parsed
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["env_var"] == "HOMEBREW_NO_ANALYTICS")
            .unwrap();
        assert!(entry["value"].is_boolean());
    }
}
