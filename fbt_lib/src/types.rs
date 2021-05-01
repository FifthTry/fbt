#[derive(Debug)]
pub struct TestConfig {
    cmd: String,
    env: Option<std::collections::HashMap<String, String>>,
    clear_env: bool,
    pub output: Option<String>,
    pub stdin: Option<String>,
    pub code: i32,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl TestConfig {
    pub fn cmd(&self) -> std::process::Command {
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = std::process::Command::new("cmd");
            c.args(&["/C", self.cmd.as_str()]);
            c
        } else {
            let mut c = std::process::Command::new("sh");
            c.args(&["-c", self.cmd.as_str()]);
            c
        };

        if self.clear_env {
            cmd.env_clear();
        }

        if let Some(ref env) = self.env {
            cmd.envs(env.iter());
        }

        if self.stdin.is_some() {
            cmd.stdin(std::process::Stdio::piped());
        }

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        cmd
    }

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
                    code: p1.header.i32("code")?,
                    stdin: None,
                    stdout: None,
                    stderr: None,
                    env: None,
                    clear_env: p1.header.bool_with_default("clear-env", false)?,
                    output: p1.header.string_optional("output")?,
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
                "stdin" => {
                    if c.stdin.is_some() {
                        return Err(ftd::p1::Error::InvalidInput {
                            message: "stdin provided more than once".to_string(),
                            context: s.to_string(),
                        });
                    }
                    c.stdin = s.body.clone();
                }
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
                "env" => {
                    if c.env.is_some() {
                        return Err(ftd::p1::Error::InvalidInput {
                            message: "env provided more than once".to_string(),
                            context: s.to_string(),
                        });
                    }
                    c.env = match s.body {
                        Some(ref v) => {
                            let mut m = std::collections::HashMap::new();
                            for line in v.split('\n') {
                                let mut parts = line.splitn(1, '=');
                                match (parts.next(), parts.next()) {
                                    (Some(k), Some(v)) => {
                                        m.insert(k.to_string(), v.to_string());
                                    }
                                    _ => {
                                        return Err(ftd::p1::Error::InvalidInput {
                                            message: "invalid line in env".to_string(),
                                            context: line.to_string(),
                                        })
                                    }
                                }
                            }
                            Some(m)
                        }
                        None => None,
                    };
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
    CantReadCWD(std::io::Error),
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
    InputIsNotDir,
    Other {
        io: std::io::Error,
    },
    CommandFailed {
        io: std::io::Error,
    },
    UnexpectedStatusCode {
        expected: i32,
        output: std::process::Output,
    },
    StdoutMismatch {
        expected: String,
        output: std::process::Output,
    },
    StderrMismatch {
        expected: String,
        output: std::process::Output,
    },
    DirDiffError {
        error: crate::DirDiffError,
    },
    OutputMismatch {
        diff: crate::DirDiff,
    },
}
