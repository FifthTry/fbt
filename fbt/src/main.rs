fn main() {
    if version_asked() {
        println!("fbt: {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let mut args = std::env::args();
    args.next(); // get rid of first element (name of binary)
    let args: Vec<_> = args.filter(|v| !v.starts_with('-')).collect();

    if let Some(code) = fbt_lib::main_with_filters(&args) {
        std::process::exit(code)
    }
}

fn version_asked() -> bool {
    std::env::args().any(|e| e == "--version" || e == "-v")
}
