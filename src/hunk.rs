use xxhash_rust::xxh32::xxh32;

/// A contiguous block of file lines with a header:
/// `@@@ path start,len hash @@@` followed by the body.
///
/// `start` is the 1-based line of the first body line, `len` is the length of
/// the **original** region, and `hash` is a staleness guard over that original
/// region — never over the (possibly edited) body carried in `body`.
pub struct Hunk {
    pub path: String,
    pub start: usize,
    pub len: usize,
    pub hash: u32,
    pub body: Vec<String>,
}

impl Hunk {
    /// Build a hunk from a slice of the file (the `expand` side). At expand
    /// time body == region, so the hash is taken over the body.
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

    /// Render header + body as it appears on the wire.
    pub fn render(&self) -> String {
        let mut out = format!(
            "@@@ {} {},{} {:x} @@@",
            self.path, self.start, self.len, self.hash
        );
        for line in &self.body {
            out.push('\n');
            out.push_str(line);
        }
        out
    }

    /// Recompute the hash over the current file region this hunk targets and
    /// compare to the header hash. `false` => the file changed since expand
    /// (stale) or the region no longer fits the file.
    pub fn verify(&self, file_lines: &[String]) -> bool {
        if self.start == 0 || self.start - 1 + self.len > file_lines.len() {
            return false;
        }
        let region = &file_lines[self.start - 1..self.start - 1 + self.len];
        hash_lines(region) == self.hash
    }

    /// Parse a stream of lines into hunks. A `@@@` header starts a new hunk;
    /// the lines that follow (until the next header) are its body. Anything
    /// before the first header is ignored. Malformed headers become warnings.
    pub fn parse_all<I: IntoIterator<Item = String>>(lines: I) -> (Vec<Hunk>, Vec<String>) {
        let mut hunks: Vec<Hunk> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();
        let mut current: Option<Hunk> = None;

        for line in lines {
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
        // user edits the body line
        *rendered.last_mut().unwrap() = "edited".to_string();
        let (parsed, _) = Hunk::parse_all(rendered);
        assert_eq!(parsed[0].body, lines(&["edited"]));
        assert_eq!(parsed[0].hash, h.hash); // hash is still the original's
    }
}
