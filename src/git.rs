use std::process::Command;

use crate::Version;

fn git_tag(version: &Version) -> Result<(), String> {
    Command::new("git")
        .args(&["tag", &format!("v{}", version.to_string())])
        .status()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn git_stage(path: &str) -> Result<(), String> {
    Command::new("git")
        .args(&["add", path])
        .status()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn git_commit(
    version: &Version,
    is_signature: bool,
    message: Option<String>,
    path: &str,
) -> Result<(), String> {
    git_stage(path)?;
    let message = message.unwrap_or_else(|| format!("📚 bump version v{}", version.to_string()));
    if is_signature {
        Command::new("git")
            .args(&["commit", "-m", &message, "-S"])
            .status()
            .map(|_| ())
            .map_err(|e| e.to_string())
    } else {
        Command::new("git")
            .args(&["commit", "-m", &message])
            .status()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

pub fn git_commit_and_tag(
    version: &Version,
    is_signature: bool,
    message: Option<String>,
    path: &str,
) -> Result<(), String> {
    git_tag(version)?;
    git_commit(version, is_signature, message, path)?;
    Ok(())
}
