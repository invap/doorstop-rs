use crate::doorstop::document::Document;
use crate::doorstop::file;
use walkdir::WalkDir;
use thiserror::Error;

use std::{cell::RefCell, collections::HashMap, error::Error, path::PathBuf, rc::Rc};

#[derive(Debug)]
pub struct DocumentTree {
    pub document: Rc<Document>,
    pub children: Vec<Rc<RefCell<DocumentTree>>>,
}

#[derive(Error,Debug)]
#[error("{msg}")]
pub struct LoadError {
    msg: String,
}

impl DocumentTree  {
    fn add_child(&mut self, child: Rc<RefCell<DocumentTree>>) {
        self.children.push(child);
    }

    pub fn load(path: &str) -> Result<Rc<RefCell<DocumentTree>>, Box<dyn Error>> {
        let ymls = find_document_yml(path);
        if ymls.len() == 0 {
            let s = "No documents found in ".to_string() + path;
            return Err(Box::new(LoadError{msg: s}));
        }
        build_doc_tree(ymls)
    }
}

fn build_doctree_index(
    paths: Vec<PathBuf>,
) -> Result<HashMap<String, Rc<RefCell<DocumentTree>>>, Box<dyn Error>> {
    let mut prefix_doc_map: HashMap<String, Rc<RefCell<DocumentTree>>> = HashMap::new();

    for each_path in paths {
        let doc = Document::new(each_path)?;

        let each_tree = DocumentTree {
            document: Rc::new(doc),
            children: Vec::new(),
        };

        prefix_doc_map.insert(
            each_tree.document.config.settings.prefix.clone(),
            Rc::new(RefCell::new(each_tree)),
        );
    }

    Ok(prefix_doc_map)
}

fn build_doc_tree(paths: Vec<PathBuf>) -> Result<Rc<RefCell<DocumentTree>>, Box<dyn Error>> {

    let mut root: Option<Rc<RefCell<DocumentTree>>> = None;
    let prefix_doc_map = build_doctree_index(paths)?;

    //Go over all the documents if no parent is the root
    for (_, value) in prefix_doc_map.iter() {
        let doc_tree = (**value).borrow();
        let parent_prefix = &doc_tree.document.config.settings.parent;

        match parent_prefix {
            Some(p) => {
                let parent = prefix_doc_map.get(p).unwrap();
                (**parent).borrow_mut().add_child(value.clone());
            }
            None => {
                root = Some(value.clone());
            }
        }
    }
    Ok(root.unwrap())
}

pub fn find_document_yml(path: &str) -> Vec<PathBuf> {
    let into_iter = WalkDir::new(path).into_iter();

    let only_doorstop_yml = into_iter.filter_entry(|e| !file::is_hidden_dir(e));

    let iterator = only_doorstop_yml.filter(|e| e.as_ref().is_ok_and(file::is_doorstop_config));

    let res: Vec<PathBuf> = iterator
        .filter_map(|e| e.ok())
        .filter_map(|e| Some(e.into_path()))
        .collect();
    res
}

#[cfg(test)]
mod tests {

    use crate::doorstop::document_tree::DocumentTree;

    use super::find_document_yml;

    #[test]
    fn test_find_document_yml() {
        let ymls = find_document_yml("resources/reqs");
        assert_eq!(3, ymls.len());
    }

    #[test]
    fn test_build_tree() {
        let document_tree = DocumentTree::load("resources/reqs").unwrap();
        let document_tree = document_tree.borrow();

        assert_eq!(
            String::from("REQ"),
            document_tree.document.config.settings.prefix,
            "testing CAUR is the root prefix"
        );

        assert_eq!(2, document_tree.children.len(), "Wrong amount of children");
    }

    #[test]
    fn test_err_build_tree() {
        let document_tree = DocumentTree::load("nonexistent-resources/reqs");
        assert!(document_tree.is_err());
    }
}
