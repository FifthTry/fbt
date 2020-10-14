use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Command {
    pub cmd: String,
    pub stdout: String
}