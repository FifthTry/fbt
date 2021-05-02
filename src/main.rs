use colored::Colorize;

fn main() {
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
    };

    for case in cases.iter() {
        match &case.result {
            Ok(status) => {
                if *status {
                    println!(
                        "{}: {} in {}",
                        case.id.blue(),
                        "PASSED".green(),
                        format!("{:?}", &case.duration).yellow()
                    );
                } else {
                    println!("{}: {}", case.id.blue(), "SKIPPED".magenta(),);
                }
            }
            Err(e) => {
                println!(
                    "{}: {} in {} ({:?})",
                    case.id.blue(),
                    "FAILED".red(),
                    format!("{:?}", &case.duration).yellow(),
                    e
                );
            }
        }
    }
}
