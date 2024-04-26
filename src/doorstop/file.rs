// doorstop-rs: Help library to read doorstop documents implemented in Rust.
// Copyright (C) <2024>  INVAP S.E.
//
// This file is part of doorstop-rs.
//
// doorstop-rs is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
