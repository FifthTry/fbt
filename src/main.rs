fn main() {
    if let Err(e) = fbt::test_all() {
        eprintln!("failed: {:?}", e)
    }
}
