use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TestCommand {
    pub cmd: String,
    pub stdout: String,
}

#[derive(Debug)]
pub struct TestResult {
    // only failure is test folder was not readable / found
    pub results: Result<Vec<SingleTestResult>, OverallFailure>,
    pub duration: std::time::Duration,
}

#[derive(Debug)]
pub struct SingleTestResult {
    pub id: String, // 01_basic
    // if Ok(true) => test passed
    // if Ok(false) => test skipped
    // if Err(Failure) => test failed
    pub result: Result<bool, vec1::Vec1<Failure>>,
    pub duration: std::time::Duration,
}

#[derive(Debug)]
pub enum OverallFailure {
    TestsFolderMissing,
    TestsFolderNotReadable(String),
}

#[derive(Debug)]
pub enum Failure {
    CmdTomlMissing,
    CmdTomlInvalid {
        parsing_error: String,
    },
    UnexpectedStatusCode {
        expected: i32,
        found: i32,
        stdout_found: String,
        stderr_found: String,
    },
    StdoutMismatch {
        expected: String,
        found: String,
    },
    StderrMismatch {
        expected: String,
        found: String,
    },
    ExpectedFileMissing {
        expected: String,
    },
    ExpectedFolderMissing {
        expected: String,
    },
    UnexpectedFileFound {
        found: String,
    },
    UnexpectedFolderFound {
        found: String,
    },
    ContentMismatch {
        file: String,
        expected: String,
        found: String,
    },
}
