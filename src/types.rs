use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TestCommand {
    pub cmd: String,
    pub stdout: String,
}
