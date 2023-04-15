use clap::ArgMatches;
use fasthash::{xx::Hash32 as XxHash32, FastHash};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use crate::errors::AppError;

pub fn expand_to_hunks(matches: &ArgMatches) -> Result<(), AppError> {
    let context_lines: usize = matches
        .value_of("context")
        .unwrap_or("1")
        .parse()
        .map_err(|_| AppError::InvalidNumber("Invalid number of lines above".to_string()))?;

    let lines_above: usize = matches
        .value_of("above")
        .unwrap_or(&context_lines.to_string())
        .parse()
        .map_err(|_| AppError::InvalidNumber("Invalid number of lines above".to_string()))?;

    let lines_below: usize = matches
        .value_of("below")
        .unwrap_or(&context_lines.to_string())
        .parse()
        .map_err(|_| AppError::InvalidNumber("Invalid number of lines below".to_string()))?;

    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());

    // Init HashMap to store files and their corresponding lines
    let mut file_lines: HashMap<String, Vec<usize>> = HashMap::new();

    // Process each line from stdin into file_lines
    for line in reader.lines() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(4, ':').collect();

        // Validate format
        if parts.len() != 4 {
            return Err(AppError::InvalidLineFormat(line));
        }

        // Extract file path and line number from parts
        let file_path = parts[0].to_string();
        let line_number: usize = parts[1]
            .parse()
            .map_err(|_| AppError::InvalidLineNumber(parts[1].to_string()))?;

        file_lines.entry(file_path).or_default().push(line_number);
    }

    // Process each files' lines
    for (file_path, lines) in file_lines {
        let path = Path::new(&file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let file_lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
        let hunks = create_hunks(lines_above, lines_below, &lines, &file_lines, file_path)?;

        for hunk in hunks {
            println!("{}", hunk);
        }
    }
    Ok(())
}

fn create_hunks(
    lines_above: usize,
    lines_below: usize,
    lines: &[usize],
    file_lines: &[String],
    file_path: String,
) -> Result<Vec<String>, AppError> {
    let mut hunks: Vec<String> = Vec::new();
    let mut current_hunk: Vec<String> = Vec::new();
    let mut current_hunk_start = 0;

    for &line_number in lines {
        let start_line = if line_number > lines_above {
            line_number - lines_above
        } else {
            1
        };
        let end_line = std::cmp::min(line_number + lines_below, file_lines.len());

        // Set the hunk start line and add lines to the hunk
        if current_hunk.is_empty() {
            current_hunk_start = start_line;
            current_hunk.extend(file_lines[start_line - 1..=end_line - 1].iter().cloned());
        } else if start_line - current_hunk_start > current_hunk.len() {
            // Create hunk text and compute its hash
            let hunk_text = current_hunk.join("\n");
            let hash = XxHash32::hash(hunk_text.as_bytes());
            // Create a hunk header and add it to the hunks vector
            let hunk_header = format!(
                "@@@ {} {},{} {:x} @@@",
                file_path,
                current_hunk_start,
                current_hunk.len(),
                hash
            );
            hunks.push(hunk_header);
            hunks.push(hunk_text);

            // Reset the current_hunk and start a new one
            current_hunk_start = start_line;
            current_hunk = file_lines[start_line - 1..=end_line - 1].to_vec();
        } else {
            // Extend the current hunk with additional lines
            current_hunk.extend(
                file_lines[current_hunk.len() + current_hunk_start - 1..=end_line - 1]
                    .iter()
                    .cloned(),
            );
        }
    }

    // Check if there's an unprocessed hunk
    if !current_hunk.is_empty() {
        let hunk_text = current_hunk.join("\n");
        let hash = XxHash32::hash(hunk_text.as_bytes());
        let hunk_header = format!(
            "@@@ {} {},{} {:x} @@@",
            file_path,
            current_hunk_start,
            current_hunk.len(),
            hash
        );
        hunks.push(hunk_header);
        hunks.push(hunk_text);
    }

    Ok(hunks)
}
