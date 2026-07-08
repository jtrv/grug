use similar::{ChangeTag, TextDiff};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use crate::errors::AppError;
use crate::hunk::Hunk;

/// Read hunks from stdin and print, for each one whose body differs from the
/// current file, a diff of `current file region` vs `edited body`.
pub fn diff_hunks() -> Result<(), AppError> {
    let stdin = io::stdin();
    let input: Vec<String> = BufReader::new(stdin.lock())
        .lines()
        .collect::<Result<_, _>>()?;

    let (hunks, warnings) = Hunk::parse_all(input);
    warnings.iter().for_each(|w| eprintln!("{}", w));

    for hunk in hunks {
        let file_lines = read_file_lines(&hunk.path)?;
        if hunk.start == 0 || hunk.start - 1 + hunk.len > file_lines.len() {
            eprintln!("Hunk at {}:{} does not fit the file", hunk.path, hunk.start);
            continue;
        }

        let region = file_lines[hunk.start - 1..hunk.start - 1 + hunk.len].join("\n");
        let edited = hunk.body.join("\n");
        if region != edited {
            println!("{}", render_header(&hunk));
            print_diff(&region, &edited);
        }
    }
    Ok(())
}

fn render_header(hunk: &Hunk) -> String {
    format!(
        "@@@ {} {},{} {:x} @@@",
        hunk.path, hunk.start, hunk.len, hunk.hash
    )
}

fn read_file_lines(file_path: &str) -> Result<Vec<String>, AppError> {
    let file = File::open(Path::new(file_path))?;
    let lines = BufReader::new(file).lines().collect::<Result<_, _>>()?;
    Ok(lines)
}

fn print_diff(region: &str, edited: &str) {
    for change in TextDiff::from_lines(region, edited).iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Equal => continue,
            ChangeTag::Delete => '-',
            ChangeTag::Insert => '+',
        };
        let value = change.value();
        if value.ends_with('\n') {
            print!("{}{}", sign, value);
        } else {
            println!("{}{}", sign, value);
        }
    }
}
