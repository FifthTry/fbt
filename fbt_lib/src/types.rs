#[derive(Debug)]
pub struct TestConfig {
    pub cmd: String,
    pub code: Option<u8>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl TestConfig {
    pub fn parse(s: &str) -> ftd::p1::Result<Self> {
        let parsed = ftd::p1::parse(s)?;
        let mut iter = parsed.iter();
        let mut c = match iter.next() {
            Some(p1) => {
                if p1.name != "ftd" {
                    return Err(ftd::p1::Error::InvalidInput {
                        message: "first section's name is not 'ftd'".to_string(),
                        context: p1.name.clone(),
                    });
                }

                TestConfig {
                    cmd: p1.header.string("cmd")?,
                    code: p1.header.i32_optional("code")?.map(|v| v as u8),
                    stderr: None,
                    stdout: None,
                }
            }
            None => {
                return Err(ftd::p1::Error::InvalidInput {
                    message: "no sections found".to_string(),
                    context: s.to_string(),
                });
            }
        };

        for s in iter {
            match s.name.as_str() {
                "stdout" => {
                    if c.stdout.is_some() {
                        return Err(ftd::p1::Error::InvalidInput {
                            message: "stdout provided more than once".to_string(),
                            context: s.to_string(),
                        });
                    }
                    c.stdout = s.body.clone();
                }
                "stderr" => {
                    if c.stderr.is_some() {
                        return Err(ftd::p1::Error::InvalidInput {
                            message: "stderr provided more than once".to_string(),
                            context: s.to_string(),
                        });
                    }
                    c.stderr = s.body.clone();
                }
                _ => {
                    return Err(ftd::p1::Error::InvalidInput {
                        message: "unknown section".to_string(),
                        context: s.name.clone(),
                    });
                }
            }
        }

        Ok(c)
    }
}

#[derive(Debug)]
pub struct Case {
    pub id: String, // 01_basic
    // if Ok(true) => test passed
    // if Ok(false) => test skipped
    // if Err(Failure) => test failed
    pub result: Result<bool, vec1::Vec1<crate::Failure>>,
    pub duration: std::time::Duration,
}

#[derive(Debug)]
pub enum Error {
    TestsFolderMissing,
    TestsFolderNotReadable(std::io::Error),
}

#[derive(Debug)]
pub enum Failure {
    CmdFileMissing,
    CmdFileInvalid {
        error: ftd::p1::Error,
    },
    CantReadCmdFile {
        error: std::io::Error,
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
