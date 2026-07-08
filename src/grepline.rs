use crate::errors::AppError;

/// A parsed vimgrep-style line: `path : line : [col] : content`.
///
/// Field count varies by tool — `file:line` (2), `grep -Hn` (3),
/// `rg --vimgrep` / `ugrep -HknI` (4). `expand` needs `path`+`line`;
/// raw-mode `write` needs `content`.
pub struct GrepLine {
    pub path: String,
    pub line: usize,
    /// Parsed for completeness; grug's commands don't consume it yet.
    #[allow(dead_code)]
    pub col: Option<usize>,
    pub content: Option<String>,
}

impl GrepLine {
    /// Parse one line. `Ok(None)` for blank lines; `Err` for malformed input
    /// (the caller decides whether to warn-and-skip).
    pub fn parse(raw: &str) -> Result<Option<GrepLine>, AppError> {
        if raw.trim().is_empty() {
            return Ok(None);
        }

        let parts: Vec<&str> = raw.splitn(4, ':').collect();
        if parts.len() < 2 {
            return Err(AppError::InvalidLineFormat(raw.to_string()));
        }

        let path = parts[0].to_string();
        let line: usize = parts[1]
            .parse()
            .map_err(|_| AppError::InvalidLineNumber(parts[1].to_string()))?;

        // ponytail: col heuristic — 4 parts AND part[2] all-digits => column.
        // A 3-field line whose content begins `digits:` is misread as having a
        // column. Rare; upgrade path is an explicit --vimgrep/--no-column flag.
        let (col, content) = match parts.as_slice() {
            [_, _] => (None, None),
            [_, _, c] => (None, Some(c.to_string())),
            [_, _, maybe_col, rest] => {
                if !maybe_col.is_empty() && maybe_col.chars().all(|c| c.is_ascii_digit()) {
                    (maybe_col.parse().ok(), Some(rest.to_string()))
                } else {
                    // No column: rejoin the tail splitn over-separated.
                    (None, Some(format!("{}:{}", maybe_col, rest)))
                }
            }
            _ => unreachable!("splitn(4) yields 2..=4 parts"),
        };

        Ok(Some(GrepLine {
            path,
            line,
            col,
            content,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_two_field() {
        let g = GrepLine::parse("src/main.rs:10").unwrap().unwrap();
        assert_eq!(g.path, "src/main.rs");
        assert_eq!(g.line, 10);
        assert_eq!(g.col, None);
        assert_eq!(g.content, None);
    }

    #[test]
    fn parses_three_field_grep_hn() {
        let g = GrepLine::parse("a.rs:3:let x = 1;").unwrap().unwrap();
        assert_eq!(g.line, 3);
        assert_eq!(g.col, None);
        assert_eq!(g.content.as_deref(), Some("let x = 1;"));
    }

    #[test]
    fn parses_four_field_vimgrep() {
        let g = GrepLine::parse("a.rs:3:5:let x = 1;").unwrap().unwrap();
        assert_eq!(g.col, Some(5));
        assert_eq!(g.content.as_deref(), Some("let x = 1;"));
    }

    #[test]
    fn content_may_contain_colons() {
        let g = GrepLine::parse("a.rs:3:5:http://x").unwrap().unwrap();
        assert_eq!(g.col, Some(5));
        assert_eq!(g.content.as_deref(), Some("http://x"));
    }

    #[test]
    fn three_field_content_with_colons_keeps_them() {
        let g = GrepLine::parse("a.rs:3:foo:bar").unwrap().unwrap();
        assert_eq!(g.col, None);
        assert_eq!(g.content.as_deref(), Some("foo:bar"));
    }

    #[test]
    fn blank_line_is_none() {
        assert!(GrepLine::parse("   ").unwrap().is_none());
    }

    #[test]
    fn missing_line_number_errors() {
        assert!(GrepLine::parse("just-a-path").is_err());
        assert!(GrepLine::parse("a.rs:notanumber:x").is_err());
    }
}
