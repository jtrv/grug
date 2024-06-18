# Grug

Grug is a command-line tool that provides a workflow for expanding, editing, diffing, and writing edits to files using vim-styled grep lines (such as `grep -RHn`, `ripgrep --vimgrep`, `ugrep -HknI`, etc).

## TODO

- [ ] adapt `--write` to apply hunk changes (e.g. edited output from `--expand`)
- [ ] create `--preview` to view a diff of current file contents and the lines/hunks piped in
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

To replace lines in files based on input from stdin:

```
echo "src/main.rs:10:new content" | grug --write
```

To expand lines from stdin into hunks:

```
echo "src/main.rs:10" | grug --expand
```

## Installation

```
cargo install --git https://github.com/jtrv/grug
```

## Kakoune

In order to use this with kakoune you can add the following code to your kakrc

```
define-command grep-write -docstring "
  grep-write: pipes the current grep-buffer to grug -w and prints the results
" %{
  declare-option -hidden str grug_buf
  evaluate-commands -draft %{
    evaluate-commands %sh{
      echo "set-option buffer grug_buf '$(mktemp /tmp/grug_buf.XXX)'"
    }
    write -sync -force %opt{grug_buf}
    evaluate-commands %sh{
      cat "$kak_opt_grug_buf" | grug -w |
        xargs -I{} echo "echo -debug 'grug: {}'; echo -markup {Information} 'grug: {}';"
    }
  }
}
```

## License

This project is licensed under the MIT License.
