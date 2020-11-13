use colored::Colorize;

fn main() {
    let test_result = match fbt_lib::test_all() {
        Ok(tr) => tr,
        Err(e) => return eprintln!("failed: {:?}", e),
    };
    let results = match test_result.results {
        Ok(r) => r,
        Err(fbt_lib::OverallFailure::TestsFolderMissing) => {
            return eprintln!("test folder missing");
        }
        Err(fbt_lib::OverallFailure::TestsFolderNotReadable(m)) => {
            return eprintln!("test folder not readable: {}", m);
        }
    };

    for result in results.iter() {
        match &result.result {
            Ok(status) => {
                if *status {
                    println!(
                        "{}: {} in {}",
                        result.id.blue(),
                        "PASSED".green(),
                        format!("{:?}", &result.duration).yellow()
                    );
                } else {
                    println!("{}: {}", result.id.blue(), "SKIPPED".magenta(),);
                }
            }
            Err(_e) => {
                println!(
                    "{}: {} in {}",
                    result.id.blue(),
                    "FAILED".red(),
                    format!("{:?}", &result.duration).yellow()
                );
            }
        }
    }
}
