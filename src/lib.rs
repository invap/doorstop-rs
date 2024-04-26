/// doorstop-rs: Help library to read doorstop documents implemented in Rust.
/// Copyright (C) <2024>  INVAP S.E.
///
/// This file is part of doorstop-rs.
///
/// doorstop-rs is free software: you can redistribute it and/or modify
/// it under the terms of the GNU Affero General Public License as published by
/// the Free Software Foundation, either version 3 of the License, or
/// (at your option) any later version.
///
/// This program is distributed in the hope that it will be useful,
/// but WITHOUT ANY WARRANTY; without even the implied warranty of
/// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
/// GNU Affero General Public License for more details.
///
/// You should have received a copy of the GNU Affero General Public License
/// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub mod doorstop {
    pub mod document;
    pub mod document_tree;
    pub mod file;
}

#[cfg(test)]
mod tests {

    use crate::doorstop::document_tree::DocumentTree;
    #[test]
    fn test_build_tree() {
        let document_tree = DocumentTree::load("resources/reqs").unwrap();
        let root_document = &document_tree.borrow().document;
        for (_, each_item) in root_document.items_sorted_by_level.iter() {
            let default_string_value = "N/A".to_string();
            println!(
                "{}-({}){}",
                each_item.id.as_ref().unwrap_or_else(|| &default_string_value),
                each_item.level.as_ref().unwrap_or_else(|| &default_string_value),
                each_item.header.as_ref().unwrap_or_else(|| &default_string_value)
            )
        }
    }
}
