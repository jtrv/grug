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

The round-trip: expand, edit the hunk bodies in your editor, then write them
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

grug ships a kakoune plugin ([`rc/grug.kak`](rc/grug.kak)) that wires up the
round-trip.

### Install with [kak-bundle](https://codeberg.org/jdugan6240/kak-bundle)

Add to your kakrc:

```kak
bundle grug https://github.com/jtrv/grug %{
  # optional: your own mappings, e.g.
  # map global user e ':grep-expand<ret>'  -docstring 'grug: expand grep buffer'
  # map global user w ':grep-write<ret>'   -docstring 'grug: write changes'
}
bundle-install-hook grug %{
  cargo install --locked --force --path .
}
```

Then run `:bundle-install`. The install hook compiles and installs the `grug`
binary, and kak-bundle sources the plugin's commands.

### Manual install

Without a plugin manager, `source` the file from your kakrc (and make sure the
`grug` binary is on your `PATH`):

```kak
source "/path/to/grug/rc/grug.kak"
```

### Commands

- `grep-expand [flags]` expands the grep buffer into a `*grep-expand*` buffer of editable hunks, leaving `*grep*` intact (`grug -e`).
- `grep-preview` shows the pending changes as a diff in a `*grep-expand-review*` buffer (`grug -p`).
- `grep-write` applies the hunks, closing the grug buffers on a clean apply and keeping them (with a report) if anything is skipped (`grug -w`).

Typical flow: `:grep foo` → `:grep-expand` → edit → `:grep-preview` (optional) →
`:grep-write`. `grep-expand` forwards grug's context flags, so `:grep-expand -C 3`
gives three lines around each match (`-A` above, `-B` below); re-run it from the
`*grep-expand*` buffer to re-expand the same matches with a different context.

## License

This project is licensed under the MIT License.
