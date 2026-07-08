use clap::ArgMatches;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use crate::grepline::GrepLine;
use crate::hunk::Hunk;

pub fn write_changes(_matches: &ArgMatches) -> io::Result<()> {
    let stdin = io::stdin();
    let input: Vec<String> = BufReader::new(stdin.lock())
        .lines()
        .collect::<Result<_, _>>()?;

    // Auto-detect: a `@@@` header anywhere means the buffer is edited hunks.
    if input.iter().any(|l| l.starts_with("@@@")) {
        write_hunks(input)
    } else {
        write_raw(input)
    }
}

// ---- hunk-apply path --------------------------------------------------------

fn write_hunks(input: Vec<String>) -> io::Result<()> {
    let (hunks, warnings) = Hunk::parse_all(input);
    warnings.iter().for_each(|w| eprintln!("{}", w));

    let mut by_file: HashMap<String, Vec<Hunk>> = HashMap::new();
    for h in hunks {
        by_file.entry(h.path.clone()).or_default().push(h);
    }

    let counts: Vec<_> = by_file
        .into_par_iter()
        .filter_map(|(path, hunks)| {
            apply_hunks_to_file(&path, hunks)
                .map_err(|e| eprintln!("Error applying hunks to {}: {}", path, e))
                .ok()
        })
        .collect();

    let applied: usize = counts.iter().map(|(a, _)| a).sum();
    let skipped: usize = counts.iter().map(|(_, s)| s).sum();
    println!("{} hunks applied, {} skipped", applied, skipped);
    Ok(())
}

fn apply_hunks_to_file(file_path: &str, hunks: Vec<Hunk>) -> io::Result<(usize, usize)> {
    let path = Path::new(file_path);
    let orig: Vec<String> = BufReader::new(File::open(path)?)
        .lines()
        .collect::<Result<_, _>>()?;

    let (out, applied, skipped) = apply_hunks(&orig, hunks, file_path);

    let mut file = File::create(path)?;
    for line in out {
        writeln!(file, "{}", line)?;
    }
    Ok((applied, skipped))
}

/// Splice edited hunk bodies into the file. Hunks apply bottom-up (highest
/// `start` first) against the original coordinates, so earlier splices — at
/// higher indices — never shift the coordinates of later ones. Stale hunks
/// (failing `verify`) and hunks overlapping an already-applied region are
/// skipped and warned.
fn apply_hunks(
    orig: &[String],
    mut hunks: Vec<Hunk>,
    file_path: &str,
) -> (Vec<String>, usize, usize) {
    hunks.sort_by_key(|h| std::cmp::Reverse(h.start)); // bottom-up

    let mut out = orig.to_vec();
    let mut applied = 0;
    let mut skipped = 0;
    let mut lowest_touched = usize::MAX; // lowest original index already spliced

    for h in hunks {
        let end = h.start - 1 + h.len; // exclusive, original coordinates
        if !h.verify(orig) {
            eprintln!(
                "Skipping stale hunk at {}:{} (file changed since expand)",
                file_path, h.start
            );
            skipped += 1;
            continue;
        }
        if end > lowest_touched {
            eprintln!("Skipping overlapping hunk at {}:{}", file_path, h.start);
            skipped += 1;
            continue;
        }
        out.splice((h.start - 1)..end, h.body);
        lowest_touched = h.start - 1;
        applied += 1;
    }
    (out, applied, skipped)
}

// ---- raw grep-line path -----------------------------------------------------

struct Change(usize, String);

fn write_raw(input: Vec<String>) -> io::Result<()> {
    let mut file_changes: HashMap<String, Vec<Change>> = HashMap::new();
    for line in &input {
        match GrepLine::parse(line) {
            Ok(Some(g)) => {
                let replacement = g.content.unwrap_or_default();
                file_changes
                    .entry(g.path)
                    .or_default()
                    .push(Change(g.line, replacement));
            }
            Ok(None) => {}
            Err(e) => eprintln!("{}", e),
        }
    }

    let counts: Vec<_> = file_changes
        .into_par_iter()
        .filter_map(|(file_path, changes)| {
            replace_lines(&file_path, changes)
                .map_err(|e| eprintln!("Error replacing lines in {}: {}", file_path, e))
                .ok()
        })
        .collect();

    let changed: usize = counts.iter().map(|(c, _)| c).sum();
    let ignored: usize = counts.iter().map(|(_, i)| i).sum();
    println!("{} lines changed, {} lines ignored", changed, ignored);
    Ok(())
}

fn replace_lines(file_path: &str, changes: Vec<Change>) -> io::Result<(usize, usize)> {
    let path = Path::new(file_path);
    let orig: Vec<String> = BufReader::new(File::open(path)?)
        .lines()
        .collect::<Result<_, _>>()?;

    let (lines, changed, ignored) = apply_changes(&orig, changes, file_path);

    let mut file = File::create(path)?;
    for line in lines {
        writeln!(file, "{}", line)?;
    }
    Ok((changed, ignored))
}

fn apply_changes(
    orig: &[String],
    changes: Vec<Change>,
    file_path: &str,
) -> (Vec<String>, usize, usize) {
    let mut lines = orig.to_vec();
    let mut changed = 0;
    let mut ignored = 0;

    for Change(line_number, replacement) in changes {
        if line_number == 0 || line_number > lines.len() {
            eprintln!(
                "Line number {} is out of range for file {}",
                line_number, file_path
            );
            ignored += 1;
            continue;
        }
        if lines[line_number - 1] != replacement {
            lines[line_number - 1] = replacement;
            changed += 1;
        } else {
            ignored += 1;
        }
    }
    (lines, changed, ignored)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hunk::Hunk;

    fn lines(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn hunk_replace_can_grow() {
        let orig = lines(&["a", "b", "c"]);
        let mut h = Hunk::from_region("f".into(), 2, lines(&["b"]));
        h.body = lines(&["b1", "b2"]); // edited to add a line
        let (out, applied, skipped) = apply_hunks(&orig, vec![h], "f");
        assert_eq!(applied, 1);
        assert_eq!(skipped, 0);
        assert_eq!(out, lines(&["a", "b1", "b2", "c"]));
    }

    #[test]
    fn hunk_replace_can_shrink() {
        let orig = lines(&["a", "b", "c", "d"]);
        let mut h = Hunk::from_region("f".into(), 2, lines(&["b", "c"]));
        h.body = lines(&["merged"]);
        let (out, _, _) = apply_hunks(&orig, vec![h], "f");
        assert_eq!(out, lines(&["a", "merged", "d"]));
    }

    #[test]
    fn two_hunks_apply_bottom_up() {
        let orig = lines(&["a", "b", "c", "d", "e"]);
        let mut top = Hunk::from_region("f".into(), 1, lines(&["a"]));
        top.body = lines(&["a1", "a2"]); // grows — must not shift the lower hunk
        let mut bot = Hunk::from_region("f".into(), 4, lines(&["d"]));
        bot.body = lines(&["D"]);
        let (out, applied, _) = apply_hunks(&orig, vec![top, bot], "f");
        assert_eq!(applied, 2);
        assert_eq!(out, lines(&["a1", "a2", "b", "c", "D", "e"]));
    }

    #[test]
    fn stale_hunk_skipped() {
        let orig = lines(&["a", "b", "c"]);
        let mut h = Hunk::from_region("f".into(), 2, lines(&["DIFFERENT"]));
        h.body = lines(&["x"]);
        // header hash is for "DIFFERENT", file region is "b" -> stale
        let (out, applied, skipped) = apply_hunks(&orig, vec![h], "f");
        assert_eq!(applied, 0);
        assert_eq!(skipped, 1);
        assert_eq!(out, orig);
    }

    #[test]
    fn overlapping_hunk_skipped() {
        let orig = lines(&["a", "b", "c", "d"]);
        let outer = Hunk::from_region("f".into(), 1, lines(&["a", "b", "c"]));
        let inner = Hunk::from_region("f".into(), 2, lines(&["b"]));
        // bottom-up: inner (start 2) applies first, outer (start 1, end 4) overlaps it
        let (_, applied, skipped) = apply_hunks(&orig, vec![outer, inner], "f");
        assert_eq!(applied, 1);
        assert_eq!(skipped, 1);
    }

    #[test]
    fn raw_change_replaces_line() {
        let orig = lines(&["a", "b", "c"]);
        let (out, changed, ignored) = apply_changes(&orig, vec![Change(2, "B".into())], "f");
        assert_eq!(changed, 1);
        assert_eq!(ignored, 0);
        assert_eq!(out, lines(&["a", "B", "c"]));
    }

    #[test]
    fn raw_out_of_range_ignored() {
        let orig = lines(&["a"]);
        let (out, changed, ignored) = apply_changes(&orig, vec![Change(9, "x".into())], "f");
        assert_eq!(changed, 0);
        assert_eq!(ignored, 1);
        assert_eq!(out, orig);
    }
}
