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

    todo!()
}
