use std::fs;
use std::path::Path;

use crate::app::{App, Setting, SettingKind};

const BLOCK_START: &str = "# homebrewconfig BEGIN";
const BLOCK_END: &str = "# homebrewconfig END";

/// Whether `path` already contains a homebrewconfig-managed block.
/// Missing or unreadable files count as "no block".
pub fn file_contains_block(path: &Path) -> bool {
    fs::read_to_string(path)
        .map(|c| c.contains(BLOCK_START))
        .unwrap_or(false)
}

/// The export block that `apply_config` would write, for display in the
/// confirmation popup before any disk write happens.
pub fn preview_block(app: &App) -> String {
    generate_block(&app.settings)
}

pub fn apply_config(app: &App) -> Result<(), String> {
    let path = &app.shell_profile;

    let existing = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("Failed to read {}: {}", path.display(), e)),
    };

    let block = generate_block(&app.settings);
    let updated = replace_block(&existing, &block);

    // No-op: nothing to write and no existing block to preserve.
    if updated == existing {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
        }
    }

    // Back up the current profile before overwriting so a bad apply is reversible.
    if !existing.is_empty() {
        let backup = path.with_extension("bak");
        fs::write(&backup, &existing)
            .map_err(|e| format!("Failed to write backup {}: {}", backup.display(), e))?;
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

fn generate_block(settings: &[Setting]) -> String {
    let mut lines = vec![BLOCK_START.to_string()];
    let mut current_category: Option<&str> = None;

    for setting in settings {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn setting(kind: SettingKind, category: &'static str) -> Setting {
        Setting {
            name: "Test",
            env_var: "HOMEBREW_TEST",
            description: "test setting",
            kind,
            bool_val: false,
            str_val: String::new(),
            num_val: None,
            category,
            modified: false,
        }
    }

    // ---- escape_value ----

    #[test]
    fn escape_value_leaves_plain_text() {
        assert_eq!(escape_value("hello"), "hello");
    }

    #[test]
    fn escape_value_escapes_quotes_and_backslashes() {
        assert_eq!(escape_value(r#"a"b"#), r#"a\"b"#);
        assert_eq!(escape_value(r"a\b"), r"a\\b");
        // Backslashes are escaped before quotes, so a literal \" becomes \\\".
        assert_eq!(escape_value(r#"\""#), r#"\\\""#);
    }

    // ---- generate_block ----

    #[test]
    fn generate_block_omits_defaults() {
        // A non-inverted bool left off and an empty string both produce no export.
        let settings = vec![
            setting(SettingKind::Bool { inverted: false }, "Display"),
            setting(SettingKind::Str, "Network"),
        ];
        let block = generate_block(&settings);
        assert_eq!(block, format!("{}\n{}", BLOCK_START, BLOCK_END));
    }

    #[test]
    fn generate_block_inverted_bool_exports_when_feature_off() {
        // inverted: feature ON (bool_val true) is the Homebrew default -> no export.
        let mut on = setting(SettingKind::Bool { inverted: true }, "Privacy");
        on.bool_val = true;
        assert_eq!(
            generate_block(std::slice::from_ref(&on)),
            format!("{}\n{}", BLOCK_START, BLOCK_END)
        );

        // Feature OFF -> the HOMEBREW_NO_* variable must be exported.
        let mut off = setting(SettingKind::Bool { inverted: true }, "Privacy");
        off.bool_val = false;
        assert!(generate_block(std::slice::from_ref(&off)).contains("export HOMEBREW_TEST=1"));
    }

    #[test]
    fn generate_block_non_inverted_bool_exports_when_on() {
        let mut s = setting(SettingKind::Bool { inverted: false }, "Display");
        s.bool_val = true;
        assert!(generate_block(std::slice::from_ref(&s)).contains("export HOMEBREW_TEST=1"));
    }

    #[test]
    fn generate_block_quotes_and_escapes_strings() {
        let mut s = setting(SettingKind::Str, "Network");
        s.str_val = r#"a"b"#.to_string();
        assert!(generate_block(std::slice::from_ref(&s)).contains(r#"export HOMEBREW_TEST="a\"b""#));
    }

    #[test]
    fn generate_block_emits_numbers_unquoted() {
        let mut s = setting(SettingKind::Num, "Updates");
        s.num_val = Some(30);
        assert!(generate_block(std::slice::from_ref(&s)).contains("export HOMEBREW_TEST=30"));
    }

    #[test]
    fn generate_block_writes_category_header_once() {
        let mut a = setting(SettingKind::Bool { inverted: false }, "Display");
        a.bool_val = true;
        let mut b = setting(SettingKind::Bool { inverted: false }, "Display");
        b.bool_val = true;
        b.name = "Test2";
        b.env_var = "HOMEBREW_TEST2";
        let block = generate_block(&[a, b]);
        assert_eq!(block.matches("# Display").count(), 1);
    }

    // ---- replace_block ----

    #[test]
    fn replace_block_appends_to_empty() {
        let block = format!("{}\n{}", BLOCK_START, BLOCK_END);
        let result = replace_block("", &block);
        assert_eq!(result, format!("{}\n", block));
    }

    #[test]
    fn replace_block_appends_after_existing_content() {
        let block = format!("{}\n{}", BLOCK_START, BLOCK_END);
        let result = replace_block("export FOO=1\n", &block);
        assert_eq!(result, format!("export FOO=1\n\n{}\n", block));
    }

    #[test]
    fn replace_block_replaces_existing_block_in_place() {
        let old = format!(
            "before\n{}\nexport OLD=1\n{}\nafter\n",
            BLOCK_START, BLOCK_END
        );
        let new_block = format!("{}\nexport NEW=1\n{}", BLOCK_START, BLOCK_END);
        let result = replace_block(&old, &new_block);
        assert!(result.contains("before"));
        assert!(result.contains("after"));
        assert!(result.contains("export NEW=1"));
        assert!(!result.contains("export OLD=1"));
        // Re-applying the same block must be idempotent.
        assert_eq!(replace_block(&result, &new_block), result);
    }

    #[test]
    fn replace_block_preserves_surrounding_lines() {
        let old = format!("{}\nexport OLD=1\n{}\nkeep me\n", BLOCK_START, BLOCK_END);
        let new_block = format!("{}\n{}", BLOCK_START, BLOCK_END);
        let result = replace_block(&old, &new_block);
        assert!(result.contains("keep me"));
        assert!(!result.contains("export OLD=1"));
    }
}
