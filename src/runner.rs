use std::{env, process::Command};

use anyhow::{Context, Result};

pub fn run(command: &str) -> Result<i32> {
    let status = shell_command(command)
        .status()
        .with_context(|| format!("failed to run `{command}`"))?;

    Ok(status.code().unwrap_or(1))
}

#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    let shell = env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string());
    let mut process = Command::new(shell);
    process.arg("/C").arg(command);
    process
}

#[cfg(not(windows))]
fn shell_command(command: &str) -> Command {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let mut process = Command::new(shell);
    process.arg("-lc").arg(command);
    process
}
