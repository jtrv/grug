use crate::errors::AppError;

/// A vimgrep-style line: `path:line[:col]:content`. The field count varies by
/// tool (`file:line`, `grep -Hn`, `rg --vimgrep`), so the column is detected
/// and dropped, leaving what `expand` (path, line) and raw-mode `write`
/// (content) need.
pub struct GrepLine {
    pub path: String,
    pub line: usize,
    pub content: Option<String>,
}

impl GrepLine {
    /// `Ok(None)` for a blank line, `Err` for a malformed one.
    pub fn parse(raw: &str) -> Result<Option<GrepLine>, AppError> {
        if raw.trim().is_empty() {
            return Ok(None);
        }

        let parts: Vec<&str> = raw.splitn(4, ':').collect();
        if parts.len() < 2 {
            return Err(AppError::InvalidLineFormat(raw.to_string()));
        }

        let path = parts[0].to_string();
        let line = parts[1]
            .parse()
            .map_err(|_| AppError::InvalidLineNumber(parts[1].to_string()))?;

        // Field 3 is a column only when it's all digits; a 3-field line whose
        // content happens to start `digits:` is misread, which is rare enough.
        let content = match parts.as_slice() {
            [_, _] => None,
            [_, _, c] => Some(c.to_string()),
            [_, _, col, rest] if !col.is_empty() && col.bytes().all(|b| b.is_ascii_digit()) => {
                Some(rest.to_string())
            }
            [_, _, a, b] => Some(format!("{a}:{b}")),
            _ => unreachable!(),
        };

        Ok(Some(GrepLine {
            path,
            line,
            content,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content(raw: &str) -> Option<String> {
        GrepLine::parse(raw).unwrap().unwrap().content
    }

    #[test]
    fn parses_two_field() {
        let g = GrepLine::parse("src/main.rs:10").unwrap().unwrap();
        assert_eq!(g.path, "src/main.rs");
        assert_eq!(g.line, 10);
        assert_eq!(g.content, None);
    }

    #[test]
    fn three_field_is_content() {
        assert_eq!(content("a.rs:3:let x = 1;").as_deref(), Some("let x = 1;"));
    }

    #[test]
    fn four_field_drops_the_column() {
        assert_eq!(
            content("a.rs:3:5:let x = 1;").as_deref(),
            Some("let x = 1;")
        );
    }

    #[test]
    fn content_may_contain_colons() {
        assert_eq!(content("a.rs:3:5:http://x").as_deref(), Some("http://x"));
        assert_eq!(content("a.rs:3:foo:bar").as_deref(), Some("foo:bar"));
    }

    #[test]
    fn blank_line_is_none() {
        assert!(GrepLine::parse("   ").unwrap().is_none());
    }

    #[test]
    fn malformed_lines_error() {
        assert!(GrepLine::parse("just-a-path").is_err());
        assert!(GrepLine::parse("a.rs:notanumber:x").is_err());
    }
}
