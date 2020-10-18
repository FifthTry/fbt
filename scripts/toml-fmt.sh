test -f .cargo/bin/cargo-tomlfmt || cargo install cargo-tomlfmt

find . | grep Cargo.toml | grep -v .cargo | grep -v target-nix | grep -v venv | xargs -n 1 cargo tomlfmt --path
