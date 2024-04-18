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
        for (_, each_item) in root_document.items.iter() {
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
