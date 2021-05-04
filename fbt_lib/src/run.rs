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

    for dir in {
        match std::fs::read_dir("./tests") {
            Ok(dir) => dir,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(crate::Error::TestsFolderMissing)
            }
            Err(e) => return Err(crate::Error::TestsFolderNotReadable(e)),
        }
    } {
        let dir = match dir {
            Ok(d) => d.path(),
            Err(e) => {
                // TODO: What is going on here? returning TestsFolderNotReadable
                //  is not great because we are losing the existing results, and
                //  we ideally want to mark this as failing and continue running
                //  tests. What error is this? How can I read a directory but
                //  know the name of this entry?
                return Err(crate::Error::TestsFolderNotReadable(e));
            }
        };

        if !dir.is_dir() {
            continue;
        }

        if dir
            .file_name()
            .map(|v| v.to_str())
            .unwrap_or(None)
            .unwrap_or("")
            .starts_with(".")
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

    return crate::Case {
        id: id.clone(),
        result: match crate::dir_diff::diff(output, reference) {
            Ok(Some(diff)) => {
                return err(crate::Failure::OutputMismatch { diff });
            }
            Ok(None) => Ok(true),
            Err(e) => return err(crate::Failure::DirDiffError { error: e }),
        },
        duration: std::time::Instant::now().duration_since(start),
    };
}
