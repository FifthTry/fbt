fn main() {
    match fbt_lib::test_all() {
        Ok(res) => println!("Test Results {:?}", res),
        Err(e) => eprintln!("failed: {:?}", e),
    }
}
