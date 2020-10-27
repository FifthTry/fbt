use crate::types::{Failure, SingleTestResult, TestResult};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub mod types;

pub fn test_all() -> anyhow::Result<TestResult> {
    let mut results = vec![];
    let mut duration = Duration::from_millis(0);
    for dir in std::fs::read_dir("./tests")? {
        let result = test_one(dir?)?;
        duration += result.duration;
        results.push(result);
    }

    Ok(TestResult {
        results: Ok(results),
        duration: duration,
    })
}

fn get_file_path(
    curr_dir: &std::fs::DirEntry,
    file_obj: &str,
    is_dir: bool,
) -> anyhow::Result<Option<PathBuf>> {
    for dir in std::fs::read_dir(curr_dir.path())? {
        let entry = dir?;
        if is_dir {
            if entry.file_type()?.is_dir()
                && entry.file_name().to_str().unwrap_or("").contains(file_obj)
            {
                return Ok(Some(entry.path()));
            }
        } else {
            if entry.file_name().to_str().unwrap_or("").contains(file_obj) {
                return Ok(Some(entry.path()));
            }
        }
    }
    Ok(None)
}

fn get_err_from_stderr(stderr: &str) -> Failure {
    if stderr.contains("No such file or directory") {
        return Failure::ExpectedFileMissing {
            expected: stderr.to_string(),
        };
    }
    return Failure::CmdTomlMissing;
}

fn test_one(entry: std::fs::DirEntry) -> anyhow::Result<SingleTestResult> {
    let mut single_result = SingleTestResult {
        id: format!("{:?}", entry.path()),
        result: Ok(false),
        duration: Duration::from_millis(0),
    };

    let file_name = entry.file_name();
    let test_dir = match file_name.to_str() {
        Some(f) => f,
        None => {
            eprintln!("cant convert directory to str: {:?}", entry);
            single_result.result = Ok(false);
            return Ok(single_result);
        }
    };

    if !test_dir.starts_with('.') && entry.file_type()?.is_dir() {
        // Not testing fbt as of now
        if test_dir.contains("fbt") {
            single_result.result = Ok(false);
            return Ok(single_result);
        }

        println!("current folder {:?}", entry.path());
        let input_path = match get_file_path(&entry, "input", true) {
            Ok(res) => {
                if let Some(path) = res {
                    path
                } else {
                    eprintln!("not a valid test case, skipping test !!!");
                    single_result.result = Ok(false);
                    return Ok(single_result);
                }
            }
            _ => {
                eprintln!("not a valid test case, skipping test !!!");
                single_result.result = Ok(false);
                return Ok(single_result);
            }
        };
        let cmd_toml_path = match get_file_path(&entry, "cmd.toml", false) {
            Ok(res) => {
                if let Some(path) = res {
                    path
                } else {
                    eprintln!("not a valid test case, skipping test !!!");
                    single_result.result = Ok(false);
                    return Ok(single_result);
                }
            }
            _ => {
                eprintln!("not a valid test case, skipping test !!!");
                single_result.result = Ok(false);
                return Ok(single_result);
            }
        };

        println!("input: {:?}, cmd.toml {:?}", input_path, cmd_toml_path);

        let contents = std::fs::read_to_string(cmd_toml_path)?;
        let test_cmd: crate::types::TestCommand = toml::from_str(&contents)?;
        println!("Command: {:?}", test_cmd);

        let args: Vec<&str> = test_cmd.cmd.split(' ').collect();
        let mut cmd = std::process::Command::new(args[0]);
        cmd.current_dir(input_path);
        //will need to add code to handle multiple args
        cmd.arg(args[1]);
        let start = Instant::now();
        let cmd_result = cmd.output()?;
        let duration = start.elapsed();
        println!("cmd result {:?}", cmd_result);
        if String::from_utf8(cmd_result.stdout)? == test_cmd.stdout.trim()
            && cmd_result.status.success()
        {
            single_result.result = Ok(true);
            single_result.duration = duration;
        } else {
            single_result.result = Err(vec1::vec1![get_err_from_stderr(
                String::from_utf8(cmd_result.stderr)?.trim()
            )]);
            single_result.duration = duration;
        };

        println!("{:?}", single_result);
        Ok(single_result)
    } else {
        Ok(single_result)
    }
}
