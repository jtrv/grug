//! End-to-end: grep line -> expand -> edit hunk body -> write -> assert file.

use std::io::Write;
use std::process::{Command, Stdio};

fn grug(args: &[&str], stdin: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_grug"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn grug");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn expand_edit_write_round_trip() {
    let dir = std::env::temp_dir().join(format!("grug-rt-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("sample.txt");
    std::fs::write(&file, "one\ntwo\nthree\nfour\nfive\n").unwrap();
    let path = file.to_str().unwrap();

    // expand a 2-field grep line (the shape the README documents)
    let expanded = grug(&["--expand"], &format!("{}:3\n", path));
    assert!(expanded.starts_with("@@@ "), "got: {}", expanded);
    assert!(expanded.contains("three"));

    // edit the hunk body: change "three" -> "THREE" and add a line
    let edited = expanded.replace("three", "THREE\nINSERTED");

    // write the edited hunk back
    let summary = grug(&["--write"], &edited);
    assert!(summary.contains("1 hunks applied"), "got: {}", summary);

    let result = std::fs::read_to_string(&file).unwrap();
    assert_eq!(result, "one\ntwo\nTHREE\nINSERTED\nfour\nfive\n");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn raw_line_write_still_works() {
    let dir = std::env::temp_dir().join(format!("grug-raw-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("sample.txt");
    std::fs::write(&file, "a\nb\nc\n").unwrap();
    let path = file.to_str().unwrap();

    let summary = grug(&["--write"], &format!("{}:2:1:B\n", path));
    assert!(summary.contains("1 lines changed"), "got: {}", summary);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "a\nB\nc\n");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn expand_emits_one_trailing_close() {
    let dir = std::env::temp_dir().join(format!("grug-close-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("sample.txt");
    std::fs::write(&file, "l1\nl2\nl3\nl4\nl5\nl6\nl7\nl8\nl9\nl10\n").unwrap();
    let path = file.to_str().unwrap();

    // two distant matches -> two separate hunks
    let expanded = grug(&["--expand"], &format!("{path}:2\n{path}:9\n"));
    let headers = expanded.lines().filter(|l| l.starts_with("@@@ ")).count();
    let closes = expanded.lines().filter(|l| *l == "@@@").count();
    assert_eq!(headers, 2, "two hunks:\n{expanded}");
    assert_eq!(closes, 1, "exactly one trailing close for the stream:\n{expanded}");

    std::fs::remove_dir_all(&dir).ok();
}
