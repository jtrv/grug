# Grug

Grug is a command-line tool that provides a workflow for expanding, editing, diffing, and writing edits to files using vim-styled grep lines (such as `grep -RHn`, `ripgrep --vimgrep`, `ugrep -HknI`, etc).

## TODO

- [ ] make --write accept hunks
- [ ] make --preview accept grep lines
- [ ] add tests

## Usage

```
grug [OPTIONS]

  -A, --above <LINES_ABOVE>        Include LINES_ABOVE lines above each line from stdin in the hunk
  -B, --below <LINES_BELOW>        Include LINES_BELOW lines below each line from stdin in the hunk
  -C, --context <CONTEXT_LINES>    Include CONTEXT_LINES above and below each line from stdin in the hunk
  -e, --expand                     Expand the lines from stdin into hunks
  -h, --help                       Print help information
  -p, --preview                    Preview diffs from the hunks passed to stdin
  -w, --write                      Replace lines in files based on input from stdin
```

## Examples

To expand lines from stdin into hunks:

```
echo "src/main.rs:10" | grug --expand
```

To preview diffs from the hunks passed to stdin:

```
echo "src/main.rs:10" | grug --preview
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
