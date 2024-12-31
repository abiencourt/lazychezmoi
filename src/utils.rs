#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
    Unchanged,
}

pub fn extract_filename_and_status(line: &str) -> (String, FileStatus, FileStatus) {
    let status_chars = &line[0..2];
    let path = line[2..].trim().to_string();

    // Extract local status (first char)
    let local_status = match status_chars.chars().next().unwrap_or(' ') {
        'M' => FileStatus::Modified,
        'A' => FileStatus::Added,
        'D' => FileStatus::Deleted,
        '?' => FileStatus::Untracked,
        _ => FileStatus::Unchanged,
    };

    // Extract source status (second char)
    let source_status = match status_chars.chars().nth(1).unwrap_or(' ') {
        'M' => FileStatus::Modified,
        'A' => FileStatus::Added,
        'D' => FileStatus::Deleted,
        '?' => FileStatus::Untracked,
        _ => FileStatus::Unchanged,
    };

    (path, local_status, source_status)
}
