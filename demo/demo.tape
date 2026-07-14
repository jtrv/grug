# vhs demo/demo.tape   (run from the repo root)
# Drives rc/grug.kak in kakoune: :grep -> :grep-expand -> edit -> :grep-preview -> :grep-write
Output demo/demo.gif

Require grug
Require kak
Require rg

Set Shell bash
Set Theme "Catppuccin Mocha"
Set FontSize 20
Set Width 1200
Set Height 700
Set Padding 20
Set TypingSpeed 60ms
Set Framerate 24

Hide
Type "source demo/setup.sh" Enter
Type `kak -e "set global disabled_hooks .*lsp.*;" src/main.rs` Enter
Sleep 2s
Show

Sleep 1s

Type ":grep TODO" Sleep 500ms Enter
Sleep 3s

Type ":grep-expand -C 1" Sleep 500ms Enter
Sleep 3s

# kak copy-indents on <ret>, so the added lines carry no leading spaces of their own
Type "/// TODO: overflow check" Sleep 500ms Enter
Sleep 800ms
Type "c"
Type "let checked = a.checked_add(b);" Enter
Type `checked.expect("add overflowed");`
Escape
Sleep 2s

Type "/// TODO: greet the user" Sleep 500ms Enter
Sleep 800ms
Type "c"
Type `let who = "grug";` Enter
Type `println!("hello, {who}");`
Escape
Sleep 2s

Type "/// TODO: read from config" Sleep 500ms Enter
Sleep 800ms
Type "c"
Type `let env = option_env!("GRUG_NAME");` Enter
Type "if let Some(n) = env { return n; }"
Escape
Sleep 2s

Type ":grep-preview" Sleep 500ms Enter
Sleep 3.5s

Type ":grep-write" Sleep 500ms Enter
Sleep 3s

Type ":edit! src/main.rs" Sleep 300ms Enter
Sleep 2.5s
Type ":edit! src/lib.rs" Sleep 300ms Enter
Sleep 3s

Hide
Type ":quit!" Enter
