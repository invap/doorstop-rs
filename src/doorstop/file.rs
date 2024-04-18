use walkdir::DirEntry;

pub fn is_hidden_dir(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') && entry.file_type().is_dir())
        .unwrap_or(false)
}

pub fn is_doorstop_config(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.eq(".doorstop.yml"))
        .unwrap_or(false)
}

pub fn is_yml_with_prefix(entry: &DirEntry, prefix: &str) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.file_type().is_file() && s.starts_with(prefix) && s.ends_with(".yml"))
        .unwrap_or(false)
}
