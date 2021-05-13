pub fn main() -> Option<i32> {
    use colored::Colorize;

    let cases = match test_all() {
        Ok(tr) => tr,
        Err(crate::Error::TestsFolderMissing) => {
            eprintln!("{}", "Tests folder is missing".red());
            return Some(1);
        }
        Err(crate::Error::TestsFolderNotReadable(e)) => {
            eprintln!("{}", format!("Tests folder is unreadable: {:?}", e).red());
            return Some(1);
        }
        Err(crate::Error::CantReadConfig(e)) => {
            eprintln!("{}", format!("Cant read config file: {:?}", e).red());
            return Some(1);
        }
        Err(crate::Error::InvalidConfig(e)) => {
            eprintln!("{}", format!("Cant parse config file: {:?}", e).red());
            return Some(1);
        }
        Err(crate::Error::BuildFailedToLaunch(e)) => {
            eprintln!(
                "{}",
                format!("Build command failed to launch: {:?}", e).red()
            );
            return Some(1);
        }
        Err(crate::Error::BuildFailed(e)) => {
            eprintln!("{}", format!("Build failed: {:?}", e).red());
            return Some(1);
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
            Err(crate::Failure::Skipped { reason }) => {
                println!("{}: {} ({})", case.id.blue(), "SKIPPED".yellow(), reason,);
            }
            Err(crate::Failure::UnexpectedStatusCode { expected, output }) => {
                any_failed = true;
                println!(
                    "{}: {} {} (exit code mismatch, expected={}, found={:?})",
                    case.id.blue(),
                    "FAILED".red(),
                    duration,
                    expected,
                    output.status.code()
                );
                println!(
                    "stdout:\n{}\n",
                    std::str::from_utf8(&output.stdout).unwrap_or("failed to decode")
                );
                println!(
                    "stderr:\n{}\n",
                    std::str::from_utf8(&output.stderr).unwrap_or("failed to decode")
                );
            }
            Err(crate::Failure::StdoutMismatch { expected, output }) => {
                any_failed = true;
                println!(
                    "{}: {} {} (stdout mismatch)",
                    case.id.blue(),
                    "FAILED".red(),
                    duration,
                );
                let stdout = std::str::from_utf8(&output.stdout).unwrap_or("failed to decode");
                println!("stdout:\n\n{}\n", stdout);
                println!(
                    "diff:\n\n{}\n",
                    diffy::create_patch(
                        (expected.to_owned() + "\n").as_str(),
                        (stdout.to_owned() + "\n").as_str()
                    )
                );
            }
            Err(crate::Failure::StderrMismatch { expected, output }) => {
                any_failed = true;
                println!(
                    "{}: {} {} (stderr mismatch)",
                    case.id.blue(),
                    "FAILED".red(),
                    duration,
                );
                let stderr = std::str::from_utf8(&output.stderr).unwrap_or("failed to decode");
                println!("stderr:\n\n{}\n", stderr);
                println!(
                    "diff:\n\n{}\n",
                    diffy::create_patch(
                        (expected.to_owned() + "\n").as_str(),
                        (stderr.to_owned() + "\n").as_str()
                    )
                );
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
        return Some(2);
    }

    None
}

pub fn test_all() -> Result<Vec<crate::Case>, crate::Error> {
    let mut results = vec![];

    let config = match std::fs::read_to_string("./tests/fbt.p1") {
        Ok(v) => match crate::Config::parse(v.as_str()) {
            Ok(config) => {
                if let Some(ref b) = config.build {
                    match if cfg!(target_os = "windows") {
                        let mut c = std::process::Command::new("cmd");
                        c.args(&["/C", b.as_str()]);
                        c
                    } else {
                        let mut c = std::process::Command::new("sh");
                        c.args(&["-c", b.as_str()]);
                        c
                    }
                    .output()
                    {
                        Ok(v) => {
                            if !v.status.success() {
                                return Err(crate::Error::BuildFailed(v));
                            }
                        }
                        Err(e) => return Err(crate::Error::BuildFailedToLaunch(e)),
                    }
                }
                config
            }
            Err(e) => return Err(crate::Error::InvalidConfig(e)),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => crate::Config::default(),
        Err(e) => return Err(crate::Error::CantReadConfig(e)),
    };

    let dirs = {
        let mut dirs: Vec<_> = match {
            match std::fs::read_dir("./tests") {
                Ok(dirs) => dirs,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    return Err(crate::Error::TestsFolderMissing)
                }
                Err(e) => return Err(crate::Error::TestsFolderNotReadable(e)),
            }
        }
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        {
            Ok(dirs) => dirs,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(crate::Error::TestsFolderMissing)
            }
            Err(e) => return Err(crate::Error::TestsFolderNotReadable(e)),
        };
        dirs.sort();
        dirs
    };

    for dir in dirs {
        if !dir.is_dir() {
            continue;
        }

        if dir
            .file_name()
            .map(|v| v.to_str())
            .unwrap_or(None)
            .unwrap_or("")
            .starts_with('.')
        {
            continue;
        }
        results.push(test_one(&config, dir));
    }

    Ok(results)
}

fn test_one(global: &crate::Config, entry: std::path::PathBuf) -> crate::Case {
    use std::borrow::BorrowMut;
    use std::io::Write;

    let id = entry
        .file_name()
        .map(|v| v.to_str())
        .unwrap_or(None)
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("{:?}", entry.file_name()));

    let start = std::time::Instant::now();
    let id_ = id.as_str();
    let err = |e: crate::Failure| crate::Case {
        id: id_.to_string(),
        result: Err(e),
        duration: std::time::Instant::now().duration_since(start),
    };

    let config = match std::fs::read_to_string(entry.join("cmd.p1")) {
        Ok(c) => match crate::TestConfig::parse(c.as_str(), global) {
            Ok(c) => c,
            Err(e) => return err(crate::Failure::CmdFileInvalid { error: e }),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return err(crate::Failure::CmdFileMissing)
        }
        Err(e) => return err(crate::Failure::CantReadCmdFile { error: e }),
    };

    if let Some(reason) = config.skip {
        return err(crate::Failure::Skipped { reason });
    };

    let fbt = {
        let fbt = std::env::temp_dir().join(format!("fbt/{}", rand::random::<i64>()));
        if fbt.exists() {
            // if we are not getting a unique directory from temp_dir and its
            // returning some standard path like /tmp, this fmt may contain the
            // output of last run, so we must empty it.
            if let Err(e) = std::fs::remove_dir_all(&fbt) {
                return err(crate::Failure::Other { io: e });
            }
        }
        if let Err(e) = std::fs::create_dir_all(&fbt) {
            return err(crate::Failure::Other { io: e });
        }
        fbt
    };

    let input = entry.join("input");

    // if input folder exists, we copy it into tmp and run our command from
    // inside that folder, else we run it from tmp
    let dir = if input.exists() {
        let dir = fbt.join("input");
        if !input.is_dir() {
            return err(crate::Failure::InputIsNotDir);
        }
        if let Err(e) = crate::copy_dir::copy_dir_all(&input, &dir) {
            return err(crate::Failure::Other { io: e });
        }
        dir
    } else {
        fbt
    };

    // eprintln!("executing '{}' in {:?}", &config.cmd, &dir);
    let mut child = match config.cmd().current_dir(&dir).spawn() {
        Ok(c) => c,
        Err(e) => {
            return err(crate::Failure::CommandFailed {
                io: e,
                reason: "cant fork process",
            });
        }
    };

    if let (Some(ref stdin), Some(cstdin)) = (config.stdin, &mut child.stdin) {
        if let Err(e) = cstdin.borrow_mut().write_all(stdin.as_bytes()) {
            return err(crate::Failure::CommandFailed {
                io: e,
                reason: "cant write to stdin",
            });
        }
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            return err(crate::Failure::CommandFailed {
                io: e,
                reason: "cant wait",
            })
        }
    };

    match output.status.code() {
        Some(code) => {
            if code != config.exit_code {
                return err(crate::Failure::UnexpectedStatusCode {
                    expected: config.exit_code,
                    output,
                });
            }
        }
        None => {
            return err(crate::Failure::UnexpectedStatusCode {
                expected: config.exit_code,
                output,
            })
        }
    }

    if let Some(ref stdout) = config.stdout {
        if std::str::from_utf8(&output.stdout).unwrap_or("").trim() != stdout.trim() {
            return err(crate::Failure::StdoutMismatch {
                output,
                expected: stdout.trim().to_string(),
            });
        }
    }

    if let Some(ref stderr) = config.stderr {
        if std::str::from_utf8(&output.stderr).unwrap_or("").trim() != stderr.trim() {
            return err(crate::Failure::StderrMismatch {
                output,
                expected: stderr.trim().to_string(),
            });
        }
    }

    // if there is `output` folder we will check if `dir` is equal to `output`.
    // if `config` has a `output key` set, then instead of the entire `dir`, we
    // will check for the folder named `output key`, which is resolved with
    // respect to `dir`

    let reference = entry.join("output");

    if !reference.exists() {
        return crate::Case {
            id,
            result: Ok(true),
            duration: std::time::Instant::now().duration_since(start),
        };
    }

    let output = match config.output {
        Some(v) => dir.join(v),
        None => dir,
    };

    crate::Case {
        id: id.clone(),
        result: match crate::dir_diff::diff(output, reference) {
            Ok(Some(diff)) => {
                return err(crate::Failure::OutputMismatch { diff });
            }
            Ok(None) => Ok(true),
            Err(e) => return err(crate::Failure::DirDiffError { error: e }),
        },
        duration: std::time::Instant::now().duration_since(start),
    }
}

fn is_test() -> bool {
    std::env::args().any(|e| e == "--test")
}
