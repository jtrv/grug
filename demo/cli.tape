# vhs demo/cli.tape   (run from the repo root)
# The same round-trip without an editor: --expand -> edit -> --preview -> --write
Output demo/cli.gif

Require grug
Require rg

Set Shell bash
Set Theme "Catppuccin Mocha"
Set FontSize 20
Set Width 1200
Set Height 760
Set Padding 20
Set TypingSpeed 45ms
Set Framerate 24

Hide
Type "source demo/setup.sh" Enter
Show

Sleep 1s
Type "rg --vimgrep TODO" Enter
Sleep 2.5s

Type "rg --vimgrep TODO | grug --expand --context 1 | tee hunks" Enter
Sleep 4s

Type "clear" Enter
Sleep 500ms
Type `sed -i 's|// TODO: overflow check|let checked = a.checked_add(b);\` Enter
Type `    checked.expect("add overflowed");|' hunks` Enter
Sleep 500ms
Type `sed -i 's|// TODO: greet the user|let who = "grug";\` Enter
Type `    println!("hello, {who}");|' hunks` Enter
Type "cat hunks" Enter
Sleep 4s

Type "clear" Enter
Sleep 500ms
Type "grug --preview < hunks" Enter
Sleep 3.5s

Type "grug --write < hunks" Enter
Sleep 2s
Type "cat src/main.rs src/lib.rs" Enter
Sleep 4s

Type "clear" Enter
Sleep 500ms
Type `echo 'src/lib.rs:9:    "grug-o-matic"' | grug --write` Enter
Sleep 1.5s
Type "cat src/lib.rs" Enter
Sleep 4s
