use std::convert::TryFrom;

#[derive(Debug, Default)]
pub(crate) struct Config {
    pub build: Option<String>,
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
                    build: p1.header.string_optional("build")?,
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
                let mut parts = line.splitn(2, '=');
                match (parts.next(), parts.next()) {
                    (Some(k), Some(v)) => {
                        m.insert(k.to_string(), v.to_string());
                    }
                    _ => {
                        return Err(ftd::p1::Error::InvalidInput {
                            message: "invalid line in env".to_string(),
                            context: line.to_string(),
                        });
                    }
                }
            }
            Some(m)
        }
        None => None,
    })
}

#[derive(Debug)]
pub(crate) struct TestConfig {
    pub cmd: String,
    env: Option<std::collections::HashMap<String, String>>,
    clear_env: bool,
    pub skip: Option<String>,
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
        cmd.env(
            "FBT_CWD",
            std::env::current_dir()
                .map(|v| v.to_string_lossy().to_string())
                .unwrap_or_else(|_| "".into()),
        );

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
                    cmd: match p1
                        .header
                        .string_optional("cmd")?
                        .or_else(|| config.cmd.clone())
                    {
                        Some(v) => v,
                        None => {
                            return Err(ftd::p1::Error::InvalidInput {
                                message: "cmd not found".to_string(),
                                context: s.to_string(),
                            })
                        }
                    },
                    skip: p1.header.string_optional("skip")?,
                    exit_code: p1
                        .header
                        .i32_optional("exit-code")?
                        .or(config.exit_code)
                        .unwrap_or(0),
                    stdin: None,
                    stdout: None,
                    stderr: None,
                    env: config.env.clone(),
                    clear_env: p1.header.bool_with_default("clear-env", config.clear_env)?,
                    output: p1
                        .header
                        .string_optional("output")?
                        .or_else(|| config.output.clone()),
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
pub enum Error {
    TestsFolderMissing,
    CantReadConfig(std::io::Error),
    InvalidConfig(ftd::p1::Error),
    BuildFailedToLaunch(std::io::Error),
    BuildFailed(std::process::Output),
    TestsFolderNotReadable(std::io::Error),
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
pub struct Output {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Output {
    pub fn replace(mut self, v: String) -> Self {
        // on mac /private is added to temp folders
        // amitu@MacBook-Pro fbt % ls /var/folders/kf/jfmbkscj7757mmr29mn3rksm0000gn/T/fbt/874862845293569866/input
        // one
        // amitu@MacBook-Pro fbt % ls /private/var/folders/kf/jfmbkscj7757mmr29mn3rksm0000gn/T/fbt/874862845293569866/input
        // one
        // both of them are the same folder, and we see the former path, but the lauched processes see the later

        let private_v = format!("/private{}", v.as_str());
        self.stdout = self.stdout.replace(private_v.as_str(), "<cwd>");
        self.stderr = self.stderr.replace(private_v.as_str(), "<cwd>");
        self.stdout = self.stdout.replace(v.as_str(), "<cwd>");
        self.stderr = self.stderr.replace(v.as_str(), "<cwd>");

        self
    }
}

impl TryFrom<&std::process::Output> for Output {
    type Error = &'static str;

    fn try_from(o: &std::process::Output) -> std::result::Result<Self, Self::Error> {
        Ok(Output {
            exit_code: match o.status.code() {
                Some(code) => code,
                None => return Err("cant read exit_code"),
            },
            stdout: {
                std::str::from_utf8(&o.stdout)
                    .unwrap_or("")
                    .trim()
                    .to_string()
            },
            stderr: {
                std::str::from_utf8(&o.stderr)
                    .unwrap_or("")
                    .trim()
                    .to_string()
            },
        })
    }
}

#[derive(Debug)]
pub enum Failure {
    Skipped {
        reason: String,
    },
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
        reason: &'static str,
    },
    UnexpectedStatusCode {
        expected: i32,
        output: Output,
    },
    CantReadOutput {
        output: std::process::Output,
        reason: &'static str,
    },
    StdoutMismatch {
        expected: String,
        output: Output,
    },
    StderrMismatch {
        expected: String,
        output: Output,
    },
    DirDiffError {
        error: crate::DirDiffError,
    },
    OutputMismatch {
        diff: crate::DirDiff,
    },
}
