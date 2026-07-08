use xxhash_rust::xxh32::xxh32;

/// A block of file lines framed by `@@@ path start,len hash @@@` and a bare
/// `@@@`. `start` is 1-based; `len` and `hash` describe the original region.
/// The hash is a staleness guard, not a checksum of `body` (which may be
/// edited between expand and write).
pub struct Hunk {
    pub path: String,
    pub start: usize,
    pub len: usize,
    pub hash: u32,
    pub body: Vec<String>,
}

impl Hunk {
    /// Build a hunk from a file region (expand side); the hash covers `body`.
    pub fn from_region(path: String, start: usize, body: Vec<String>) -> Hunk {
        let hash = hash_lines(&body);
        Hunk {
            path,
            start,
            len: body.len(),
            hash,
            body,
        }
    }

    /// Render header + body + close marker as it appears on the wire.
    pub fn render(&self) -> String {
        let mut out = format!(
            "@@@ {} {},{} {:x} @@@",
            self.path, self.start, self.len, self.hash
        );
        for line in &self.body {
            out.push('\n');
            out.push_str(line);
        }
        out.push('\n');
        out.push_str(CLOSE);
        out
    }

    /// True if the file's current region still hashes to the header hash;
    /// false if it drifted since expand or no longer fits the file.
    pub fn verify(&self, file_lines: &[String]) -> bool {
        if self.start == 0 || self.start - 1 + self.len > file_lines.len() {
            return false;
        }
        let region = &file_lines[self.start - 1..self.start - 1 + self.len];
        hash_lines(region) == self.hash
    }

    /// Parse a stream into hunks. A header opens a hunk; following lines are
    /// its body until a bare `@@@`, the next header, or EOF. Content between a
    /// close and the next header (e.g. an editor's trailing newline) is
    /// ignored; a missing close is tolerated. Malformed headers become warnings.
    pub fn parse_all<I: IntoIterator<Item = String>>(lines: I) -> (Vec<Hunk>, Vec<String>) {
        let mut hunks: Vec<Hunk> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();
        let mut current: Option<Hunk> = None;

        for line in lines {
            if line.trim() == CLOSE {
                if let Some(h) = current.take() {
                    hunks.push(h);
                }
                continue;
            }
            match parse_header(&line) {
                Some(Ok(h)) => {
                    if let Some(prev) = current.take() {
                        hunks.push(prev);
                    }
                    current = Some(h);
                }
                Some(Err(w)) => {
                    if let Some(prev) = current.take() {
                        hunks.push(prev);
                    }
                    warnings.push(w);
                }
                None => {
                    if let Some(h) = current.as_mut() {
                        h.body.push(line);
                    }
                }
            }
        }
        if let Some(h) = current.take() {
            hunks.push(h);
        }
        (hunks, warnings)
    }
}

/// Bare marker that closes a hunk body.
const CLOSE: &str = "@@@";

fn hash_lines(lines: &[String]) -> u32 {
    xxh32(lines.join("\n").as_bytes(), 0)
}

/// `None` if the line isn't a header; `Some(Err)` if it looks like one but
/// doesn't parse; `Some(Ok)` with an empty body otherwise.
fn parse_header(line: &str) -> Option<Result<Hunk, String>> {
    if !line.starts_with("@@@") {
        return None;
    }
    let malformed = || Some(Err(format!("Malformed hunk header: {}", line)));

    let inner = match line
        .strip_prefix("@@@ ")
        .and_then(|s| s.strip_suffix(" @@@"))
    {
        Some(i) => i,
        None => return malformed(),
    };

    // path may contain spaces; split the two trailing tokens from the right.
    let mut it = inner.rsplitn(3, ' ');
    let (hash_s, range_s, path) = match (it.next(), it.next(), it.next()) {
        (Some(h), Some(r), Some(p)) => (h, r, p),
        _ => return malformed(),
    };
    let (start_s, len_s) = match range_s.split_once(',') {
        Some(x) => x,
        None => return malformed(),
    };

    match (
        u32::from_str_radix(hash_s, 16),
        start_s.parse::<usize>(),
        len_s.parse::<usize>(),
    ) {
        (Ok(hash), Ok(start), Ok(len)) => Some(Ok(Hunk {
            path: path.to_string(),
            start,
            len,
            hash,
            body: Vec::new(),
        })),
        _ => malformed(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn render_parse_are_inverses() {
        let body = lines(&["fn a() {", "    b();", "}"]);
        let h = Hunk::from_region("src/x.rs".into(), 10, body.clone());
        let rendered = h.render();
        let (parsed, warnings) = Hunk::parse_all(rendered.lines().map(String::from));
        assert!(warnings.is_empty());
        assert_eq!(parsed.len(), 1);
        let p = &parsed[0];
        assert_eq!(p.path, "src/x.rs");
        assert_eq!(p.start, 10);
        assert_eq!(p.len, 3);
        assert_eq!(p.hash, h.hash);
        assert_eq!(p.body, body);
    }

    #[test]
    fn verify_true_on_unchanged_region() {
        let file = lines(&["0", "1", "2", "3", "4"]);
        let h = Hunk::from_region("f".into(), 2, file[1..4].to_vec());
        assert!(h.verify(&file));
    }

    #[test]
    fn verify_false_on_mutated_region() {
        let file = lines(&["0", "1", "2", "3", "4"]);
        let h = Hunk::from_region("f".into(), 2, file[1..4].to_vec());
        let mutated = lines(&["0", "1", "CHANGED", "3", "4"]);
        assert!(!h.verify(&mutated));
    }

    #[test]
    fn verify_false_when_region_overruns_file() {
        let file = lines(&["0", "1"]);
        let h = Hunk::from_region("f".into(), 1, lines(&["0", "1", "2"]));
        assert!(!h.verify(&file));
    }

    #[test]
    fn path_with_spaces_round_trips() {
        let h = Hunk::from_region("my dir/x.rs".into(), 1, lines(&["a"]));
        let (parsed, w) = Hunk::parse_all(h.render().lines().map(String::from));
        assert!(w.is_empty());
        assert_eq!(parsed[0].path, "my dir/x.rs");
    }

    #[test]
    fn malformed_header_warns() {
        let (hunks, warnings) = Hunk::parse_all(lines(&["@@@ garbage", "body"]));
        assert!(hunks.is_empty());
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn edited_body_kept_hash_from_header() {
        let h = Hunk::from_region("f".into(), 1, lines(&["orig"]));
        let mut rendered: Vec<String> = h.render().lines().map(String::from).collect();
        // user edits the body line (index 1: after the header, before the close)
        rendered[1] = "edited".to_string();
        let (parsed, _) = Hunk::parse_all(rendered);
        assert_eq!(parsed[0].body, lines(&["edited"]));
        assert_eq!(parsed[0].hash, h.hash); // hash is still the original's
    }

    #[test]
    fn trailing_lines_after_close_are_ignored() {
        let h = Hunk::from_region("f".into(), 1, lines(&["a", "b", "c"]));
        // an editor appends a blank line after the close marker
        let mut wire: Vec<String> = h.render().lines().map(String::from).collect();
        wire.push(String::new());
        let (parsed, _) = Hunk::parse_all(wire);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].body, lines(&["a", "b", "c"])); // no phantom blank line
    }

    #[test]
    fn close_bounds_the_body_between_hunks() {
        let wire = lines(&["@@@ f 1,1 0 @@@", "one", "@@@", "", "@@@ f 5,1 0 @@@", "two", "@@@"]);
        let (parsed, _) = Hunk::parse_all(wire);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].body, lines(&["one"]));
        assert_eq!(parsed[1].body, lines(&["two"]));
    }
}
