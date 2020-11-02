use colored::Colorize;
use fbt_lib::types::SingleTestResult;

fn main() {
    match fbt_lib::test_all() {
        Ok(test_result) => {
            let mut single_results: Vec<SingleTestResult> = vec![];
            match test_result.results {
                Ok(result) => single_results = result,
                _ => {}
            }
            for test in single_results.iter() {
                let id = test.id.clone();
                match &test.result {
                    Ok(status) => {
                        if *status {
                            println!(
                                "Test: {}, Status: {}, Time: {}",
                                id.blue().to_string(),
                                "SKIPPED".magenta().to_string(),
                                format!("{:?}", &test.duration).yellow().to_string()
                            );
                        } else {
                            println!(
                                "Test: {}, Status: {}, Time: {}",
                                id.blue().to_string(),
                                "SUCCESS".green().to_string(),
                                format!("{:?}", &test.duration).yellow().to_string()
                            );
                        }
                    }
                    Err(_e) => {
                        println!(
                            "Test: {}, Status: {}, Time: {}",
                            id.blue().to_string(),
                            "FAILED".red().to_string(),
                            format!("{:?}", &test.duration).yellow().to_string()
                        );
                    }
                }
            }
            // println!("Test Results {:#?}", res)
        }
        Err(e) => eprintln!("failed: {:?}", e),
    }
}
