use std::fs;

use crate::app::{App, SettingKind};

const BLOCK_START: &str = "# homebrewconfig BEGIN";
const BLOCK_END: &str = "# homebrewconfig END";

pub fn apply_config(app: &App) -> Result<(), String> {
    let path = &app.shell_profile;

    let existing = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("Failed to read {}: {}", path.display(), e)),
    };

    let block = generate_block(app);
    let updated = replace_block(&existing, &block);

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
        }
    }

    fs::write(path, updated).map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    Ok(())
}

fn replace_block(existing: &str, block: &str) -> String {
    let start = existing.find(BLOCK_START);
    let end = existing.find(BLOCK_END);

    if let (Some(start), Some(end)) = (start, end) {
        if end > start {
            let block_end = end + BLOCK_END.len();
            let mut tail = &existing[block_end..];
            if let Some(stripped) = tail.strip_prefix('\n') {
                tail = stripped;
            }
            let mut head = &existing[..start];
            if let Some(stripped) = head.strip_suffix('\n') {
                head = stripped;
            }
            let mut result = String::new();
            if !head.is_empty() {
                result.push_str(head);
                result.push('\n');
            }
            result.push_str(block);
            if !tail.is_empty() {
                result.push('\n');
                result.push_str(tail);
            } else {
                result.push('\n');
            }
            return result;
        }
    }

    let mut result = String::new();
    if !existing.is_empty() {
        result.push_str(existing.trim_end_matches('\n'));
        result.push_str("\n\n");
    }
    result.push_str(block);
    result.push('\n');
    result
}

fn generate_block(app: &App) -> String {
    let mut lines = vec![BLOCK_START.to_string()];
    let mut current_category: Option<&str> = None;

    for setting in &app.settings {
        let export = match &setting.kind {
            SettingKind::Bool { inverted } => {
                let should_export = if *inverted {
                    !setting.bool_val
                } else {
                    setting.bool_val
                };
                if should_export {
                    Some(format!("export {}=1", setting.env_var))
                } else {
                    None
                }
            }
            SettingKind::Str => {
                if setting.str_val.is_empty() {
                    None
                } else {
                    Some(format!(
                        "export {}=\"{}\"",
                        setting.env_var,
                        escape_value(&setting.str_val)
                    ))
                }
            }
            SettingKind::Num => setting
                .num_val
                .map(|n| format!("export {}={}", setting.env_var, n)),
        };

        if let Some(line) = export {
            if current_category != Some(setting.category) {
                lines.push(format!("# {}", setting.category));
                current_category = Some(setting.category);
            }
            lines.push(line);
        }
    }

    lines.push(BLOCK_END.to_string());
    lines.join("\n")
}

fn escape_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
