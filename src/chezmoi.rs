use std::process::Command;

use crate::app::{FileItem, Selection};
use crate::utils;

pub const HOME: &str = "~/";

pub fn check_installed() -> color_eyre::Result<()> {
    match std::process::Command::new("chezmoi")
        .arg("--version")
        .output()
    {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Err(color_eyre::eyre::eyre!("chezmoi is not installed. Please install it first: https://www.chezmoi.io/install/"))
            } else {
                Err(e.into())
            }
        }
    }
}

// TODO: Should this return a Result?
pub fn update_status() -> Vec<FileItem> {
    let output = Command::new("chezmoi")
        .arg("status")
        .stdin(std::process::Stdio::inherit()) // Allows user to enter lpass password if needed
        .output()
        .unwrap_or_else(|_| panic!("failed to execute chezmoi status"));

    let files: Vec<FileItem> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            let (path, local_status, source_status) = utils::extract_filename_and_status(line);
            FileItem {
                path,
                selected: Selection::None,
                local_status,
                source_status,
            }
        })
        .collect();
    files
}

// TODO: Should this return a Result?
pub fn diff(path: &str) -> String {
    let output = Command::new("chezmoi")
        .arg("diff")
        .arg(format!("{}{}", HOME, path))
        .output()
        .unwrap_or_else(|_| panic!("failed to execute chezmoi diff"));

    // Strip ANSI escape sequences from the output
    let diff = String::from_utf8_lossy(&output.stdout).to_string();
    let stripped = strip_ansi_escapes::strip(&diff);
    String::from_utf8_lossy(&stripped).to_string()
}

pub fn re_add(selected_files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::new("chezmoi");
    command.arg("re-add");

    // Add each file as a separate argument
    for file in selected_files {
        command.arg(format!("{}{}", HOME, file));
    }

    let output = command.output()?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }

    Ok(())
}

pub fn apply(selected_files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::new("chezmoi");
    command.arg("apply");

    for file in selected_files {
        command.arg(format!("{}{}", HOME, file));
    }

    let output = command.output()?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }

    Ok(())
}

pub fn edit(highlighted_file: String) {
    let _ = Command::new("chezmoi")
        .arg("edit")
        .arg(format!("{}{}", HOME, highlighted_file))
        .spawn()
        .unwrap_or_else(|_| panic!("failed to execute chezmoi edit"))
        .wait();
}

pub fn open_source() {
    let _ = Command::new("chezmoi")
        .arg("edit")
        .spawn()
        .unwrap_or_else(|_| panic!("failed to execute chezmoi edit"))
        .wait();
}
