pub fn test_all() -> Result<Vec<crate::Case>, crate::Error> {
    let mut results = vec![];

    for dir in match std::fs::read_dir("./tests") {
        Ok(dir) => dir,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(crate::Error::TestsFolderMissing)
        }
        Err(e) => return Err(crate::Error::TestsFolderNotReadable(e)),
    } {
        results.push(test_one(match dir {
            Ok(d) => {
                let d = d.path();
                if d.file_name()
                    .map(|v| v.to_str())
                    .unwrap_or(None)
                    .unwrap_or("")
                    .starts_with(".")
                {
                    continue;
                }
                d
            }
            Err(e) => {
                // TODO: What is going on here? returning TestsFolderNotReadable
                //  is not great because we are losing the existing results, and
                //  we ideally want to mark this as failing and continue running
                //  tests. What error is this? How can I read a directory but
                //  know the name of this entry?
                return Err(crate::Error::TestsFolderNotReadable(e));
            }
        }));
    }

    Ok(results)
}

fn test_one(entry: std::path::PathBuf) -> crate::Case {
    use std::borrow::BorrowMut;
    use std::io::Write;

    let id = entry
        .file_name()
        .map(|v| v.to_str())
        .unwrap_or(None)
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("{:?}", entry.file_name()));

    let start = std::time::Instant::now();
    let err = |e: crate::Failure| crate::Case {
        id: id.clone(),
        result: Err(vec1::Vec1::new(e)),
        duration: std::time::Instant::now().duration_since(start),
    };

    // Not testing fbt as of now
    if id.contains("fbt") {
        eprintln!("not testing fbt as of now");
        return crate::Case {
            id,
            result: Ok(false),
            duration: std::time::Instant::now().duration_since(start),
        };
    }

    let config = match std::fs::read_to_string(entry.join("cmd.p1")) {
        Ok(c) => match crate::TestConfig::parse(c.as_str()) {
            Ok(c) => c,
            Err(e) => return err(crate::Failure::CmdFileInvalid { error: e }),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return err(crate::Failure::CmdFileMissing)
        }
        Err(e) => return err(crate::Failure::CantReadCmdFile { error: e }),
    };

    let fbt = std::env::temp_dir().join("fbt");
    if fbt.exists() {
        if let Err(e) = std::fs::remove_dir_all(&fbt) {
            return err(crate::Failure::Other { io: e });
        }
    }

    let input = entry.join("input");

    // if input folder exists, we copy it into tmp and run our command from
    // inside that folder, else we run it from tmp
    let dir = if input.exists() {
        let dir = fbt.join("input");
        if !dir.is_dir() {
            return err(crate::Failure::InputIsNotDir);
        }
        if let Err(e) = crate::copy_dir::copy_dir_all(&dir, &fbt) {
            return err(crate::Failure::Other { io: e });
        }
        dir
    } else {
        fbt
    };

    let mut child = match config.cmd().current_dir(&dir).spawn() {
        Ok(c) => c,
        Err(e) => {
            return err(crate::Failure::CommandFailed { io: e });
        }
    };

    if let (Some(ref stdin), Some(cstdin)) = (config.stdin, &mut child.stdin) {
        if let Err(e) = cstdin.borrow_mut().write_all(stdin.as_bytes()) {
            return err(crate::Failure::CommandFailed { io: e });
        }
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => return err(crate::Failure::CommandFailed { io: e }),
    };

    match output.status.code() {
        Some(code) => {
            if code != config.code {
                return err(crate::Failure::UnexpectedStatusCode {
                    expected: config.code,
                    output,
                });
            }
        }
        None => {
            return err(crate::Failure::UnexpectedStatusCode {
                expected: config.code,
                output,
            })
        }
    }

    if let Some(ref stdout) = config.stdout {
        if output.stdout != stdout.as_bytes() {
            return err(crate::Failure::StdoutMismatch {
                output,
                expected: stdout.clone(),
            });
        }
    }

    if let Some(ref stderr) = config.stderr {
        if output.stdout != stderr.as_bytes() {
            return err(crate::Failure::StdoutMismatch {
                output,
                expected: stderr.clone(),
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
        id,
        result: match crate::dir_diff::diff(output, reference) {
            Ok(diff) => {
                if diff.is_empty() {
                    Ok(true)
                } else {
                    return err(crate::Failure::OutputMismatch { diff });
                }
            }
            Err(e) => return err(crate::Failure::DirDiffError { error: e }),
        },
        duration: std::time::Instant::now().duration_since(start),
    };
}
