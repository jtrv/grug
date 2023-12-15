# Grug

Grug is a command-line tool that provides a workflow for expanding, editing, diffing, and writing edits to files using vim-styled grep lines (such as `grep -RHn`, `ripgrep --vimgrep`, `ugrep -HknI`, etc).

## TODO

- [ ] adapt `--write` to apply hunk changes (e.g. edited output from `--expand`)
- [ ] create `--preview` to view a diff of lines/hunks supplied and current file contents
- [ ] refactor so it doesn't seem like I leaned on chatgpt as much as I did
- [ ] add tests

## Usage

```
grug [OPTIONS]

  -A, --above <LINES_ABOVE>        Include LINES_ABOVE lines above each line from stdin in the hunk
  -B, --below <LINES_BELOW>        Include LINES_BELOW lines below each line from stdin in the hunk
  -C, --context <CONTEXT_LINES>    Include CONTEXT_LINES above and below each line from stdin in the hunk
  -e, --expand                     Expand the lines from stdin into hunks
  -h, --help                       Print help information
  -w, --write                      Replace lines in files based on input from stdin
```

## Examples

To expand lines from stdin into hunks:

```
echo "src/main.rs:10" | grug --expand
```

To replace lines in files based on input from stdin:

```
echo "src/main.rs:10:new content" | grug --write
```

## Installation

```
cargo install --git https://github.com/jtrv/grug
```

## License

This project is licensed under the MIT License.
