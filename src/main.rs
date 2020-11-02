use colored::Colorize;
use fbt_lib::types::SingleTestResult;

fn main() {
    match fbt_lib::test_all() {
        Ok(res) => {
            let mut tests: Vec<SingleTestResult> = vec![];
            match res.results {
                Ok(val) => tests = val,
                _ => {}
            }
            for test in tests.iter() {
                let id = test.id.clone();
                match &test.result {
                    Ok(_) => {
                        println!(
                            "Test Name: {}, Success: {}",
                            id.magenta().to_string(),
                            "TRUE".green().to_string()
                        );
                    }
                    Err(_e) => {
                        println!(
                            "Test Name: {}, Success: {}",
                            id.magenta().to_string(),
                            "FALSE".red().to_string()
                        );
                    }
                }
            }
            // println!("Test Results {:#?}", res)
        }
        Err(e) => eprintln!("failed: {:?}", e),
    }
}
