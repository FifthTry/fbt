#[derive(Debug, Default)]
pub struct Config {
    cmd: Option<String>,
    env: Option<std::collections::HashMap<String, String>>,
    clear_env: bool,
    output: Option<String>,
    exit_code: Option<i32>,
}

impl Config {
    pub fn parse(s: &str) -> ftd::p1::Result<Self> {
        let parsed = ftd::p1::parse(s)?;
        let mut iter = parsed.iter();
        let mut c = match iter.next() {
            Some(p1) => {
                if p1.name != "fbt" {
                    return Err(ftd::p1::Error::InvalidInput {
                        message: "first section's name is not 'fbt'".to_string(),
                        context: p1.name.clone(),
                    });
                }

                Config {
                    cmd: p1.header.string_optional("cmd")?,
                    exit_code: p1.header.i32_optional("exit-code")?,
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
                "env" => {
                    if c.env.is_some() {
                        return Err(ftd::p1::Error::InvalidInput {
                            message: "env provided more than once".to_string(),
                            context: s.to_string(),
                        });
                    }
                    c.env = read_env(&s.body)?;
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

fn read_env(
    body: &Option<String>,
) -> ftd::p1::Result<Option<std::collections::HashMap<String, String>>> {
    Ok(match body {
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
    })
}

#[derive(Debug)]
pub struct TestConfig {
    cmd: String,
    env: Option<std::collections::HashMap<String, String>>,
    clear_env: bool,
    pub output: Option<String>,
    pub stdin: Option<String>,
    pub exit_code: i32,
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

    pub fn parse(s: &str, config: &Config) -> ftd::p1::Result<Self> {
        let parsed = ftd::p1::parse(s)?;
        let mut iter = parsed.iter();
        let mut c = match iter.next() {
            Some(p1) => {
                if p1.name != "fbt" {
                    return Err(ftd::p1::Error::InvalidInput {
                        message: "first section's name is not 'fbt'".to_string(),
                        context: p1.name.clone(),
                    });
                }

                TestConfig {
                    cmd: match p1.header.string_optional("cmd")?.or(config.cmd.clone()) {
                        Some(v) => v,
                        None => {
                            return Err(ftd::p1::Error::InvalidInput {
                                message: "cmd not found".to_string(),
                                context: s.to_string(),
                            })
                        }
                    },
                    exit_code: p1
                        .header
                        .i32_optional("exit-code")?
                        .or(config.exit_code)
                        .unwrap_or(0),
                    stdin: None,
                    stdout: None,
                    stderr: None,
                    env: config.env.clone(),
                    clear_env: p1
                        .header
                        .bool_optional("clear-env")?
                        .unwrap_or(config.clear_env),
                    output: p1
                        .header
                        .string_optional("output")?
                        .or(config.output.clone()),
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
                    c.env = match (read_env(&s.body)?, &c.env) {
                        (Some(v), Some(e)) => {
                            let mut e = e.clone();
                            e.extend(v.into_iter());
                            Some(e)
                        }
                        (Some(v), None) => Some(v),
                        (None, v) => v.clone(),
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
    pub result: Result<bool, crate::Failure>,
    pub duration: std::time::Duration,
}

#[derive(Debug)]
pub enum Error {
    TestsFolderMissing,
    CantReadConfig(std::io::Error),
    InvalidConfig(ftd::p1::Error),
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
