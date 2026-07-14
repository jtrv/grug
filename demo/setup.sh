# sourced by demo.tape to build a throwaway playground
# prefer a local release build over whatever grug is installed
[ -x target/release/grug ] && PATH="$PWD/target/release:$PATH"
# demo.tape runs your own kak config; point it at the rc in this checkout so the
# recording shows the working tree, not whatever version is installed
cd "$(mktemp -d)" || exit 1
export PS1='$ '

# a real crate, so rust-analyzer doesn't pop a "failed to discover workspace" error
mkdir -p src
cat > Cargo.toml <<'EOF'
[package]
name = "demo"
version = "0.1.0"
edition = "2021"
EOF

cat > src/lib.rs <<'EOF'
pub fn add(a: i32, b: i32) -> i32 {
    // TODO: overflow check
    a + b
}

pub fn name() -> &'static str {
    // TODO: read from config
    "grug"
}
EOF

cat > src/main.rs <<'EOF'
fn main() {
    // TODO: greet the user
    println!("{} says {}", demo::name(), demo::add(1, 2));
}
EOF

clear
