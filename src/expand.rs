use clap::ArgMatches;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use crate::errors::AppError;
use crate::grepline::GrepLine;
use crate::hunk::Hunk;

pub fn expand_to_hunks(matches: &ArgMatches) -> Result<(), AppError> {
    let arg = |name| matches.get_one::<String>(name).map(String::as_str);

    let context_lines: usize = arg("context")
        .unwrap_or("1")
        .parse()
        .map_err(|_| AppError::InvalidNumber("Invalid number of context lines".to_string()))?;

    let lines_above: usize = arg("above")
        .map(str::parse)
        .transpose()
        .map_err(|_| AppError::InvalidNumber("Invalid number of lines above".to_string()))?
        .unwrap_or(context_lines);

    let lines_below: usize = arg("below")
        .map(str::parse)
        .transpose()
        .map_err(|_| AppError::InvalidNumber("Invalid number of lines below".to_string()))?
        .unwrap_or(context_lines);

    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());

    // Group requested line numbers by file (warn-and-skip on malformed input).
    let mut file_lines: HashMap<String, Vec<usize>> = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        match GrepLine::parse(&line) {
            Ok(Some(g)) => file_lines.entry(g.path).or_default().push(g.line),
            Ok(None) => {}
            Err(e) => eprintln!("{}", e),
        }
    }

    let mut printed = false;
    for (file_path, lines) in file_lines {
        let path = Path::new(&file_path);
        let file = File::open(path)?;
        let contents: Vec<String> = BufReader::new(file).lines().collect::<Result<_, _>>()?;

        for hunk in build_hunks(&file_path, &contents, &lines, lines_above, lines_below) {
            println!("{}", hunk.render());
            printed = true;
        }
    }
    // A single terminator bounds the last hunk against an editor's trailing newline.
    if printed {
        println!("{}", crate::hunk::CLOSE);
    }
    Ok(())
}

/// Expand requested line numbers into merged hunks. Each line contributes the
/// region `[line - above, line + below]` (clamped to the file); overlapping or
/// adjacent regions merge into one hunk. Lines past EOF are skipped.
fn build_hunks(
    file_path: &str,
    contents: &[String],
    lines: &[usize],
    above: usize,
    below: usize,
) -> Vec<Hunk> {
    let mut ranges: Vec<(usize, usize)> = lines
        .iter()
        .filter(|&&n| n >= 1 && n <= contents.len())
        .map(|&n| {
            let start = if n > above { n - above } else { 1 };
            let end = std::cmp::min(n + below, contents.len());
            (start, end)
        })
        .collect();
    ranges.sort_unstable();

    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (s, e) in ranges {
        match merged.last_mut() {
            Some(last) if s <= last.1 + 1 => last.1 = last.1.max(e),
            _ => merged.push((s, e)),
        }
    }

    merged
        .into_iter()
        .map(|(s, e)| Hunk::from_region(file_path.to_string(), s, contents[s - 1..e].to_vec()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("line{}", i)).collect()
    }

    #[test]
    fn single_line_gets_context() {
        let h = build_hunks("f", &file(10), &[5], 1, 1);
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].start, 4);
        assert_eq!(h[0].body, vec!["line4", "line5", "line6"]);
    }

    #[test]
    fn adjacent_lines_merge() {
        let h = build_hunks("f", &file(10), &[3, 4], 1, 1);
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].body, vec!["line2", "line3", "line4", "line5"]);
    }

    #[test]
    fn distant_lines_stay_separate() {
        let h = build_hunks("f", &file(20), &[3, 15], 1, 1);
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn clamps_at_file_edges() {
        let h = build_hunks("f", &file(3), &[1], 5, 5);
        assert_eq!(h[0].start, 1);
        assert_eq!(h[0].body, vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn line_past_eof_skipped() {
        assert!(build_hunks("f", &file(3), &[99], 1, 1).is_empty());
    }
}
