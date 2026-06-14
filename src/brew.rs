use std::process::Command;

/// Run `brew environment --shell=bash` and return its stdout. Returns `None`
/// only when the `brew` executable cannot be spawned at all (e.g. Homebrew is
/// not installed); a non-zero exit still yields whatever was printed, since
/// brew may emit unrelated warnings to stderr while still listing the
/// environment on stdout.
pub fn brew_environment() -> Option<String> {
    let output = Command::new("brew")
        .args(["environment", "--shell=bash"])
        .output()
        .ok()?;
    String::from_utf8(output.stdout).ok()
}

/// Parse `HOMEBREW_*` assignments out of brew's environment output. Accepts
/// `export VAR="value"`, `export VAR=value` and bare `VAR=value`, stripping
/// surrounding quotes. Lines that are not Homebrew assignments are ignored.
pub fn parse_brew_env(output: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        let line = line.strip_prefix("export ").unwrap_or(line);
        if let Some((key, val)) = line.split_once('=') {
            if key.starts_with("HOMEBREW_") {
                let val = val.trim().trim_matches('"').trim_matches('\'');
                pairs.push((key.to_string(), val.to_string()));
            }
        }
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_exported_quoted_values() {
        let out = "export HOMEBREW_CELLAR=\"/opt/homebrew/Cellar\"\n\
                   export HOMEBREW_PREFIX=\"/opt/homebrew\"\n";
        let pairs = parse_brew_env(out);
        assert_eq!(
            pairs,
            vec![
                (
                    "HOMEBREW_CELLAR".to_string(),
                    "/opt/homebrew/Cellar".to_string()
                ),
                ("HOMEBREW_PREFIX".to_string(), "/opt/homebrew".to_string()),
            ]
        );
    }

    #[test]
    fn parses_bare_and_single_quoted_values() {
        let out = "HOMEBREW_NO_ANALYTICS=1\nHOMEBREW_TEMP='/tmp/brew'\n";
        let pairs = parse_brew_env(out);
        assert_eq!(
            pairs[0],
            ("HOMEBREW_NO_ANALYTICS".to_string(), "1".to_string())
        );
        assert_eq!(
            pairs[1],
            ("HOMEBREW_TEMP".to_string(), "/tmp/brew".to_string())
        );
    }

    #[test]
    fn ignores_non_homebrew_and_garbage_lines() {
        let out = "export PATH=\"/usr/bin\"\nsome noise\nexport HOMEBREW_CACHE=\"/c\"\n";
        let pairs = parse_brew_env(out);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "HOMEBREW_CACHE");
    }

    #[test]
    fn empty_output_yields_no_pairs() {
        assert!(parse_brew_env("").is_empty());
    }
}
