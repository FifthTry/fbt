use fbt_lib;

fn main() {
    if let Err(e) = fbt_lib::test_all() {
        eprintln!("failed: {:?}", e)
    }
}
