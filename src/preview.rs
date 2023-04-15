use fasthash::{xx::Hash32 as XxHash32, FastHash};
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use crate::errors::AppError;

pub fn diff_hunks() -> Result<(), AppError> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());

    let re = Regex::new(r"@@@ (.*?) (\d+),(\d+) ([0-9a-f]+) @@@").unwrap();

    let mut _current_file_path = String::new();
    let mut current_file_lines: Option<Vec<String>> = None;
    let mut current_hunk_lines: Vec<String> = Vec::new();
    let mut inside_hunk = false;

    for line in reader.lines() {
        let line = line?;

        if inside_hunk {
            if line.starts_with("@@@") {
                verify_and_print_hunk(
                    &mut current_file_lines,
                    &current_hunk_lines,
                )?;
                current_hunk_lines.clear();
                inside_hunk = false;
            } else {
                current_hunk_lines.push(line.clone());
            }
        }

        if !inside_hunk {
            if let Some(caps) = re.captures(&line) {
                _current_file_path = caps[1].to_string();
                current_file_lines = Some(read_file_lines(&_current_file_path)?);
                inside_hunk = true;
            }
        }
    }

    if !current_hunk_lines.is_empty() {
        verify_and_print_hunk(
            &mut current_file_lines,
            &current_hunk_lines,
        )?;
    }

    Ok(())
}

fn read_file_lines(file_path: &str) -> Result<Vec<String>, AppError> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
    Ok(lines)
}

fn verify_and_print_hunk(
    file_lines: &mut Option<Vec<String>>,
    hunk_lines: &[String],
) -> Result<(), AppError> {
    if let Some(lines) = file_lines {
        let hunk_text = hunk_lines.join("\n");
        let hunk_hash = XxHash32::hash(hunk_text.as_bytes());

        let file_text = lines.join("\n");
        let file_hash = XxHash32::hash(file_text.as_bytes());

        if hunk_hash != file_hash {
            let diff = difference::Changeset::new(&file_text, &hunk_text, "\n");
            println!("{}", hunk_lines[0]); // Print the hunk header
            print_diff(&diff);
        }
    }

    Ok(())
}

fn print_diff(changeset: &difference::Changeset) {
    for diff in &changeset.diffs {
        match diff {
            difference::Difference::Same(_) => {
                // Do nothing, as we only want to print the changed lines
            }
            difference::Difference::Add(ref x) => {
                print!("+"); // Use '+' to represent added lines
                println!("{}", x);
            }
            difference::Difference::Rem(ref x) => {
                print!("-"); // Use '-' to represent removed lines
                println!("{}", x);
            }
        }
    }
}
