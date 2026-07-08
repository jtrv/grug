# Grug

Grug is a command-line tool for expanding, editing, diffing, and writing edits to files using vim-styled grep lines (such as `grep -RHn`, `ripgrep --vimgrep`, `ugrep -HknI`, etc).
Grug is heavily inspired by the functionality and workflows of [kakoune-multi-file](https://github.com/natasky/kakoune-multi-file), and [kakoune-find](https://github.com/occivink/kakoune-find).

## Usage

```
grug [OPTIONS]

  -A, --above <LINES_ABOVE>        Include LINES_ABOVE lines above each line from stdin in the hunk
  -B, --below <LINES_BELOW>        Include LINES_BELOW lines below each line from stdin in the hunk
  -C, --context <CONTEXT_LINES>    Include CONTEXT_LINES above and below each line from stdin in the hunk
  -e, --expand                     Expand the lines from stdin into hunks
  -p, --preview                    Preview diffs of file contents against the hunks piped in
  -h, --help                       Print help information
  -w, --write                      Apply stdin to files: edited hunks (@@@) or raw grep lines
```

Grug accepts vimgrep lines in `path:line`, `path:line:content`, or
`path:line:col:content` form, so `grep -Hn`, `ripgrep --vimgrep`, and
`ugrep -HknI` all work.

## Examples

Expand grep lines into editable hunks:

```
echo "src/main.rs:10" | grug --expand
```

The round-trip — expand, edit the hunk bodies in your editor, then write them
back. `--write` auto-detects hunk input (lines carrying `@@@` headers):

```
rg --vimgrep TODO | grug --expand > /tmp/hunks   # edit /tmp/hunks, then:
grug --write < /tmp/hunks
```

`--write` also still takes raw grep lines directly:

```
echo "src/main.rs:10:new content" | grug --write
```

Preview what `--write` would change without touching files:

```
grug --preview < /tmp/hunks
```

## Installation

```
cargo install --git https://github.com/jtrv/grug
```

## Shell completions

`grug` generates completions for any shell clap supports. For fish:

```
grug --completions fish > ~/.config/fish/completions/grug.fish
```

Swap `fish` for `bash`, `zsh`, `elvish`, or `powershell` as needed.

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
