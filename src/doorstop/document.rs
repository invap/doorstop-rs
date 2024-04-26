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

use crate::doorstop::file;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use walkdir::WalkDir;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentSettings {
    pub digits: i32,
    pub parent: Option<String>,
    pub prefix: String,
    pub sep: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Attributes {
    pub publish: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentConfig {
    pub settings: DocumentSettings,
    pub attributes: Option<Attributes>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: Option<String>,
    pub active: Option<bool>,
    pub derived: Option<bool>,
    pub header: Option<String>,
    pub level: Option<String>,
    pub normative: Option<bool>,
    pub reviewed: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum LevelRelation {
    SameLevel,
    OutLevel(i32),
    InLevel(i32),
}

impl Item {
    pub fn level_relation(&self, other: &Self) -> LevelRelation {
        let mut self_level = self.get_level_key();
        let mut other_level = other.get_level_key();
        //remove trailings 0
        while self_level.last().is_some_and(|x| *x == 0) {
            self_level.pop();
        }
        while other_level.last().is_some_and(|x| *x == 0) {
            other_level.pop();
        }
        let self_len = self_level.len();
        let other_len = other_level.len();

        match self_len.cmp(&other_len) {
            std::cmp::Ordering::Less => LevelRelation::OutLevel((other_len - self_len) as i32),
            std::cmp::Ordering::Equal => LevelRelation::SameLevel,
            std::cmp::Ordering::Greater => LevelRelation::InLevel((self_len - other_len) as i32),
        }
    }

    ///Return the nesting level (depth)
    ///depth is 0 based.
    /// if last element is 0 does not count as doorstop convention uses that as a mark for headers.
    pub fn get_depth(&self) -> i32 {
        let mut level = self.get_level_key();
        while level.last().is_some_and(|l| 0 == *l) {
            level.pop();
        }
        (level.len() - 1) as i32
    }

    pub fn new(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let reader = std::fs::File::open(&path)?;
        let mut item: Item = serde_yaml::from_reader(reader).unwrap();
        let id = path
            .file_stem()
            .ok_or("Error on stem")?
            .to_str()
            .ok_or("Error on convert to str")?
            .to_string();
        item.id = Some(id);
        Ok(item)
    }

    pub fn get_level(&self) -> &str {
        match &self.level {
            Some(s) => s,
            None => "1",
        }
    }

    pub fn get_level_key(&self) -> Vec<i32> {
        let level = self.get_level();
        level
            .split(".")
            .filter_map(|level| i32::from_str(level).ok())
            .collect()
    }
}

#[derive(Debug)]
pub struct Document {
    pub config: DocumentConfig,
    pub root_path: PathBuf,
    pub items: HashMap<Rc<String>, Rc<Item>>,
    pub items_sorted_by_id: BTreeMap<Vec<i32>, Rc<Item>>,
}

impl Document {
    pub fn new(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let reader = std::fs::File::open(&path)?;
        let document_config: DocumentConfig = serde_yaml::from_reader(reader).unwrap();
        let id_item: HashMap<Rc<String>, Rc<Item>> = HashMap::new();

        let mut doc = Document {
            config: document_config,
            root_path: path,
            items: id_item,
            items_sorted_by_id: BTreeMap::new(),
        };

        for each_item_path in doc.get_items_files() {
            let i = Rc::new(Item::new(each_item_path)?);
            let a = Rc::new(i.id.as_ref().unwrap().clone());
            doc.items.insert(a.clone(), i.clone());
            doc.items_sorted_by_id.insert(i.get_level_key(), i);
        }
        Ok(doc)
    }

    fn get_items_files(&self) -> Vec<PathBuf> {
        let d = self.root_path.parent().unwrap();
        let into_iter = WalkDir::new(d).into_iter();

        let only_reqs_yml = into_iter
            .filter_map(|e| e.ok())
            .filter(|e| file::is_yml_with_prefix(&e, &self.config.settings.prefix));

        only_reqs_yml.filter_map(|e| Some(e.into_path())).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::doorstop::document::LevelRelation;

    use super::{Document, Item};
    use std::{path::Path, rc::Rc};

    #[test]
    fn test_new_document() {
        let p = Path::new("resources/reqs/.doorstop.yml");
        let d = Document::new(p.to_path_buf());
        let k = Rc::new("REQ012".to_string());
        assert!(d.is_ok());
        assert!(
            d.unwrap().items.get(k.as_ref()).is_some(),
            "Getting one item"
        );
    }

    #[test]
    fn test_new_document_ext() {
        let p = Path::new("resources/reqs/ext/.doorstop.yml");
        let d = Document::new(p.to_path_buf());
        assert!(d.is_ok());
    }

    #[test]
    fn test_get_items() {
        let p = Path::new("resources/reqs/.doorstop.yml");
        let d = Document::new(p.to_path_buf()).unwrap();
        let files = d.get_items_files();
        assert_eq!(18, files.len(), "Wrong amount of files detected")
    }

    #[test]
    fn test_deserialize_item() {
        let p = Path::new("resources/reqs/REQ004.yml");
        let item = Item::new(p.to_path_buf()).ok().unwrap();
        assert!(item.active.unwrap());
        assert!(!item.derived.unwrap());
        assert_eq!("REQ004", item.id.unwrap());
    }

    #[test]
    fn test_item_get_depth_default() {
        let item = Item {
            id: None,
            active: None,
            derived: None,
            header: None,
            level: None,
            normative: None,
            reviewed: None,
            text: None,
        };
        assert_eq!(0, item.get_depth());
    }

    #[test]
    fn test_item_get_depth_1() {
        let item = Item {
            id: None,
            active: None,
            derived: None,
            header: None,
            level: Some("2.10".to_string()),
            normative: None,
            reviewed: None,
            text: None,
        };
        assert_eq!(1, item.get_depth());
    }

    #[test]
    fn test_item_get_depth_0() {
        let item = Item {
            id: None,
            active: None,
            derived: None,
            header: None,
            level: Some("2.0".to_string()),
            normative: None,
            reviewed: None,
            text: None,
        };
        assert_eq!(0, item.get_depth());
    }

    #[test]
    fn test_sort_by_level() {
        let p = Path::new("resources/reqs/.doorstop.yml");
        let d = Document::new(p.to_path_buf()).unwrap();
        let level_stack = 0;
        for (_, each_item) in d.items_sorted_by_id.iter() {
            println!("{},{:?}-{:?}", level_stack, each_item.level, each_item.id);
        }
    }
    #[test]
    fn test_level_relation_same() {
        let a = Item {
            level: Some("1.0".to_string()),
            id: None,
            active: None,
            derived: None,
            header: None,
            normative: None,
            reviewed: None,
            text: None,
        };
        let b = Item {
            level: Some("2.0".to_string()),
            id: None,
            active: None,
            derived: None,
            header: None,
            normative: None,
            reviewed: None,
            text: None,
        };
        assert_eq!(a.level_relation(&b), LevelRelation::SameLevel);
    }

    #[test]
    fn test_level_relation_level_out() {
        let a = Item {
            level: Some("1.2".to_string()),
            id: None,
            active: None,
            derived: None,
            header: None,
            normative: None,
            reviewed: None,
            text: None,
        };
        let b = Item {
            level: Some("2.0".to_string()),
            id: None,
            active: None,
            derived: None,
            header: None,
            normative: None,
            reviewed: None,
            text: None,
        };
        assert_eq!(b.level_relation(&a), LevelRelation::OutLevel(1));
    }

    #[test]
    fn test_level_relation_level_out_2() {
        let a = Item {
            level: Some("1.2.1".to_string()),
            id: None,
            active: None,
            derived: None,
            header: None,
            normative: None,
            reviewed: None,
            text: None,
        };
        let b = Item {
            level: Some("2.0".to_string()),
            id: None,
            active: None,
            derived: None,
            header: None,
            normative: None,
            reviewed: None,
            text: None,
        };
        assert_eq!(b.level_relation(&a), LevelRelation::OutLevel(2));
    }

    #[test]
    fn test_level_relation_level_in() {
        let a = Item {
            level: Some("1.0".to_string()),
            id: None,
            active: None,
            derived: None,
            header: None,
            normative: None,
            reviewed: None,
            text: None,
        };
        let b = Item {
            level: Some("1.2".to_string()),
            id: None,
            active: None,
            derived: None,
            header: None,
            normative: None,
            reviewed: None,
            text: None,
        };
        assert_eq!(b.level_relation(&a), LevelRelation::InLevel(1));
    }
}
