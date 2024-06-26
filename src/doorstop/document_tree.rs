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

use crate::doorstop::document::Document;
use crate::doorstop::file;
use thiserror::Error;
use walkdir::WalkDir;

use std::{cell::RefCell, collections::HashMap, error::Error, path::PathBuf, rc::Rc};

#[derive(Debug)]
pub struct DocumentTree {
    pub document: Rc<Document>,
    pub children: Vec<Rc<RefCell<DocumentTree>>>,
    ///Complete document tree index, k= document prefix, v= document tree
    pub prefix_index: Rc<RefCell<HashMap<String, Rc<RefCell<DocumentTree>>>>>,
}

#[derive(Error, Debug)]
#[error("{msg}")]
pub struct LoadError {
    msg: String,
}

impl DocumentTree {
    fn add_child(&mut self, child: Rc<RefCell<DocumentTree>>) {
        self.children.push(child);
    }

    pub fn load(path: &str) -> Result<Rc<RefCell<DocumentTree>>, Box<dyn Error>> {
        let ymls = find_document_yml(path);
        if ymls.len() == 0 {
            let s = "No documents found in ".to_string() + path;
            return Err(Box::new(LoadError { msg: s }));
        }
        build_doc_tree(ymls)
    }
}

fn build_doctree_index(
    paths: Vec<PathBuf>,
) -> Result<Rc<RefCell<HashMap<String, Rc<RefCell<DocumentTree>>>>>, Box<dyn Error>> {
    let prefix_doc_map: Rc<RefCell<HashMap<String, Rc<RefCell<DocumentTree>>>>> =
        Rc::new(RefCell::new(HashMap::new()));

    for each_path in paths {
        let doc = Document::new(each_path)?;

        let each_tree = DocumentTree {
            document: Rc::new(doc),
            children: Vec::new(),
            prefix_index: Rc::clone(&prefix_doc_map),
        };

        prefix_doc_map.borrow_mut().insert(
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
    for (_, value) in prefix_doc_map.borrow().iter() {
        let doc_tree = (**value).borrow();
        let parent_prefix = &doc_tree.document.config.settings.parent;

        match parent_prefix {
            Some(p) => {
                let h = prefix_doc_map.borrow();
                let parent = h.get(p).unwrap();
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
            "testing REQ is the root prefix"
        );

        assert_eq!(2, document_tree.children.len(), "Wrong amount of children");
    }

    #[test]
    fn get_document_by_index() {
        let document_tree = DocumentTree::load("resources/reqs").unwrap();
        let document_tree = document_tree.borrow();

        let h = document_tree.prefix_index.borrow();
        let doc = h.get("TUT");
        assert_eq!(
            String::from("TUT"),
            doc.unwrap().borrow().document.config.settings.prefix,
            "testing REQ is the root prefix"
        );

    }

    #[test]
    fn test_err_build_tree() {
        let document_tree = DocumentTree::load("nonexistent-resources/reqs");
        assert!(document_tree.is_err());
    }
}
