fn main() {
    if version_asked() {
        println!("fbt: {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if let Some(code) = fbt_lib::main() {
        std::process::exit(code)
    }
}

fn version_asked() -> bool {
    std::env::args().any(|e| e == "--version" || e == "-v")
}
