fn main() {
    if version_asked() {
        println!("fbt: {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    fbt_lib::main()
}

fn version_asked() -> bool {
    std::env::args().any(|e| e == "--version" || e == "-v")
}
