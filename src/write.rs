use clap::ArgMatches;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub fn write_changes(_matches: &ArgMatches) -> io::Result<()> {
    let file_changes = process_input()?;

    let actual_changes_and_ignores: Vec<_> = file_changes
        .into_par_iter()
        .filter_map(|(file_path, changes)| {
            replace_lines(&file_path, changes)
                .map_err(|e| eprintln!("Error replacing lines in {}: {}", file_path, e))
                .ok()
        })
        .collect();

    let actual_changed_count: usize = actual_changes_and_ignores.iter().map(|(c, _)| c).sum();
    let actual_ignored_count: usize = actual_changes_and_ignores.iter().map(|(_, i)| i).sum();

    println!(
        "{} lines changed, {} lines ignored",
        actual_changed_count, actual_ignored_count
    );
    Ok(())
}

fn process_input() -> io::Result<HashMap<String, Vec<Change>>> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());

    let mut file_changes: HashMap<String, Vec<Change>> = HashMap::new();

    for line in reader.lines() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(4, ':').collect();

        if parts.len() != 4 {
            eprintln!("Invalid line format: {}", line);
            continue;
        }

        let file_path = parts[0].to_string();
        let line_number: usize = match parts[1].parse() {
            Ok(num) => num,
            Err(_) => {
                eprintln!("Invalid line number: {}", parts[1]);
                continue;
            }
        };

        let replacement = String::from(parts[3]);

        file_changes
            .entry(file_path)
            .or_default()
            .push(Change(line_number, replacement));
    }

    Ok(file_changes)
}

struct Change(usize, String);

fn replace_lines(file_path: &str, changes: Vec<Change>) -> io::Result<(usize, usize)> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    let mut changed_count = 0;
    let mut ignored_count = 0;

    for Change(line_number, replacement) in changes {
        if line_number == 0 || line_number > lines.len() {
            eprintln!(
                "Line number {} is out of range for file {}",
                line_number, file_path
            );
            ignored_count += 1;
            continue;
        }

        if lines[line_number - 1] != replacement {
            lines[line_number - 1] = replacement;
            changed_count += 1;
        } else {
            ignored_count += 1;
        }
    }

    let mut file = File::create(path)?;
    for line in lines {
        writeln!(file, "{}", line)?;
    }

    Ok((changed_count, ignored_count))
}
