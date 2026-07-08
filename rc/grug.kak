# grug.kak: drive the grug round-trip from kakoune.
#
# Flow: :grep foo  ->  :grep-expand  ->  edit the hunks  ->
#       :grep-preview (optional)  ->  :grep-write
#
# grep-expand reads the current grep buffer and writes hunks to a dedicated
# *grep-expand* buffer, leaving *grep* intact. grep-preview and grep-write act
# on *grep-expand* whatever buffer you call them from. grug -w auto-detects raw
# grep lines vs edited @@@ hunks.

declare-option -hidden str grug_src   # temp holding the grep lines to expand
declare-option -hidden str grug_tmp   # temp for preview/write payloads

define-command grep-expand -params 0.. -docstring "
  grep-expand [flags]: expand the grep buffer into a *grep-expand* buffer.
  Forwards grug's context flags: -C N (around), -A N (above), -B N (below).
  Re-run it from *grep-expand* to re-expand the same matches with new context.
" %{
  # Stash the source grep lines, unless we're re-expanding from *grep-expand*.
  evaluate-commands %sh{
    if [ "$kak_bufname" != '*grep-expand*' ]; then
      src=$(mktemp "${TMPDIR:-/tmp}/grug_src.XXXXXX")
      printf "write -sync -force '%s'\n" "$src"
      printf "set-option global grug_src '%s'\n" "$src"
    fi
  }
  edit! -scratch *grep-expand*
  set-option buffer filetype grep-expand
  evaluate-commands %sh{
    out=$(mktemp "${TMPDIR:-/tmp}/grug_out.XXXXXX")
    grug -e "$@" < "$kak_opt_grug_src" > "$out"
    printf "execute-keys '%%' '|cat %s<ret>gg'\n" "$out"
    printf "nop %%sh{ rm -f '%s' }\n" "$out"
  }
}

define-command grep-preview -docstring "
  grep-preview: show a diff of the *grep-expand* hunks against the files in a
  *grep-expand-preview* buffer, without writing anything.
" %{
  evaluate-commands %sh{ printf "set-option global grug_tmp '%s'\n" "$(mktemp "${TMPDIR:-/tmp}/grug_rev.XXXXXX")" }
  evaluate-commands -buffer *grep-expand* %{ write -sync -force %opt{grug_tmp} }
  edit! -scratch *grep-expand-preview*
  set-option buffer filetype diff
  evaluate-commands %sh{
    out=$(mktemp "${TMPDIR:-/tmp}/grug_revout.XXXXXX")
    grug -p < "$kak_opt_grug_tmp" > "$out"
    [ -s "$out" ] || printf '(no changes)\n' > "$out"
    printf "execute-keys '%%' '|cat %s<ret>gg'\n" "$out"
    printf "nop %%sh{ rm -f '%s' '%s' }\n" "$out" "$kak_opt_grug_tmp"
  }
}

define-command grep-write -docstring "
  grep-write: apply the *grep-expand* hunks (or raw grep lines) to files. On a
  clean apply the grug buffers close; if anything is skipped they stay open and
  the report is shown.
" %{
  evaluate-commands %sh{ printf "set-option global grug_tmp '%s'\n" "$(mktemp "${TMPDIR:-/tmp}/grug_w.XXXXXX")" }
  evaluate-commands -buffer *grep-expand* %{ write -sync -force %opt{grug_tmp} }
  evaluate-commands %sh{
    err=$(mktemp)
    out=$(grug -w < "$kak_opt_grug_tmp" 2>"$err")
    errmsg=$(cat "$err"); rm -f "$err" "$kak_opt_grug_tmp"
    skipped=$(printf '%s' "$out" | sed -n 's/.*, \([0-9][0-9]*\) \(skipped\|ignored\)$/\1/p')
    if [ -z "$errmsg" ] && [ "${skipped:-0}" = 0 ]; then
      printf "echo -markup '{Information}grug: %s'\n" "$out"
      printf "try %%{ delete-buffer! *grep-expand-preview* }\n"
      printf "try %%{ delete-buffer! *grep-expand* }\n"
    else
      esc=$(printf '%s\n%s' "$out" "$errmsg" | sed "s/'/''/g")
      printf "echo -markup '{Error}grug: not fully applied'\n"
      printf "info -title 'grug' -- '%s'\n" "$esc"
    fi
  }
}

# Highlight the @@@ hunk headers and the stream terminator. Hunk bodies are
# arbitrary file content, so they keep the buffer's default (plain) face.
provide-module grug-highlight %{
  add-highlighter shared/grep-expand group
  add-highlighter shared/grep-expand/header regex ^(@@@)\h(.+?)\h(\d+,\d+)\h([0-9a-f]+)\h(@@@)$ 1:meta 2:module 3:value 4:comment 5:meta
  add-highlighter shared/grep-expand/close regex ^@@@$ 0:meta
}

hook global WinSetOption filetype=grep-expand %{
  require-module grug-highlight
  add-highlighter window/grep-expand ref grep-expand
  hook -once -always window WinSetOption filetype=.* %{
    remove-highlighter window/grep-expand
  }
}
