use anyhow::Result;
use assert_cmd::Command;

pub fn create_cmd() -> Result<Command> {
    let cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    Ok(cmd)
}
