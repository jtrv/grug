# grug.kak: drive the grug round-trip from kakoune.
#
# Flow: :grep foo  ->  :grep-expand  ->  edit the hunks  ->
#       :grep-preview (optional)  ->  :grep-write
#
# grug -w auto-detects raw grep lines vs edited @@@ hunks, so grep-write
# works on both a plain grep buffer and an expanded-and-edited one.

# Shared temp file used to hand the current buffer to grug.
declare-option -hidden str grug_buf

define-command grep-expand -params 0.. -docstring "
  grep-expand [flags]: expand the current grep buffer into editable hunks.
  Forwards grug's context flags to control how far each hunk expands:
  -C N (around), -A N (above), -B N (below). Defaults to one line.
" %{
  execute-keys '%' "|grug -e %arg{@}<ret>"
  set-option buffer filetype grep-expand
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
