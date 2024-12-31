use crate::app::FileStatus;

pub fn extract_filename_and_status(line: &str) -> (String, FileStatus) {
    // chezmoi status output format is similar to git status --porcelain
    let status = if line.starts_with("A ") {
        FileStatus::Added
    } else if line.starts_with("M ") {
        FileStatus::Modified
    } else if line.starts_with("D ") {
        FileStatus::Deleted
    } else if line.starts_with("?? ") {
        FileStatus::Untracked
    } else if line.starts_with("R ") {
        FileStatus::Renamed
    } else {
        FileStatus::Modified // default case
    };

    // Extract filename by removing status prefix
    let path = line.split_whitespace().last().unwrap_or("").to_string();
    (path, status)
}
