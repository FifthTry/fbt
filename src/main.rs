fn main() {
    use colored::Colorize;

    let cases = match fbt_lib::test_all() {
        Ok(tr) => tr,
        Err(fbt_lib::Error::TestsFolderMissing) => {
            eprintln!("{}", "Tests folder is missing".red());
            std::process::exit(1);
        }
        Err(fbt_lib::Error::TestsFolderNotReadable(e)) => {
            eprintln!("{}", format!("Tests folder is unreadable: {:?}", e).red());
            std::process::exit(1);
        }
        Err(fbt_lib::Error::CantReadConfig(e)) => {
            eprintln!("{}", format!("Cant read config file: {:?}", e).red());
            std::process::exit(1);
        }
        Err(fbt_lib::Error::InvalidConfig(e)) => {
            eprintln!("{}", format!("Cant parse config file: {:?}", e).red());
            std::process::exit(1);
        }
        Err(fbt_lib::Error::BuildFailedToLaunch(e)) => {
            eprintln!(
                "{}",
                format!("Build command failed to launch: {:?}", e).red()
            );
            std::process::exit(1);
        }
        Err(fbt_lib::Error::BuildFailed(e)) => {
            eprintln!("{}", format!("Build failed: {:?}", e).red());
            std::process::exit(1);
        }
    };

    let mut any_failed = false;
    for case in cases.iter() {
        let duration = if is_test() {
            "".to_string()
        } else {
            format!("in {}", format!("{:?}", &case.duration).yellow())
        };

        match &case.result {
            Ok(status) => {
                if *status {
                    println!("{}: {} {}", case.id.blue(), "PASSED".green(), duration);
                } else {
                    println!("{}: {}", case.id.blue(), "SKIPPED".magenta(),);
                }
            }
            Err(e) => {
                any_failed = true;
                println!(
                    "{}: {} {} ({:?})",
                    case.id.blue(),
                    "FAILED".red(),
                    duration,
                    e
                );
            }
        }
    }

    if any_failed {
        std::process::exit(2)
    }
}

fn is_test() -> bool {
    std::env::args().any(|e| e == "--test" || e == "--replay")
}
