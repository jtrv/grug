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

Add the following to your kakrc. It wires up the full round-trip: from a grep
buffer (`:grep`, ripgrep, etc.), run `grep-expand` to turn the matches into
editable hunks, edit them, optionally `grep-preview` the diff, then
`grep-write` to apply.

```
# Shared temp file used to hand the current buffer to grug.
declare-option -hidden str grug_buf

define-command grep-expand -docstring "
  grep-expand: expand the current grep buffer into editable hunks
" %{
  execute-keys '%|grug -e<ret>'
}

define-command grep-preview -docstring "
  grep-preview: preview a diff of files against the edited hunks in this
  buffer, without writing anything
" %{
  evaluate-commands -draft %{
    evaluate-commands %sh{
      echo "set-option buffer grug_buf '$(mktemp /tmp/grug_buf.XXX)'"
    }
    write -sync -force %opt{grug_buf}
    evaluate-commands %sh{
      diff=$(grug -p < "$kak_opt_grug_buf")
      [ -z "$diff" ] && diff="(no changes)"
      # escape single quotes for kakoune's string syntax
      diff=$(printf '%s' "$diff" | sed "s/'/''/g")
      printf "info -title 'grug preview' -- '%s'" "$diff"
    }
  }
}

define-command grep-write -docstring "
  grep-write: apply the current buffer to files (raw grep lines or edited
  hunks) and report the result
" %{
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

Expansion context follows grug's flags — pass them in the pipe, e.g.
`%|grug -e -C 3<ret>` for three lines of context. `grep-write` handles both a
raw grep buffer and an expanded-and-edited hunk buffer, since `grug -w`
auto-detects the two.

## License

This project is licensed under the MIT License.
