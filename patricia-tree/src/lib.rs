use anyhow::Result;
use std::{collections::HashMap, mem};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PatriciaError {
    #[error("Key already exists")]
    KeyAlreadyExists,
}

#[derive(Debug, Clone)]
pub struct PatriciaNode {
    pub prefix: String,
    pub children: HashMap<char, Box<PatriciaNode>>,
    pub is_leaf: bool,
}

impl PatriciaNode {
    pub fn new(prefix: &str) -> Self {
        PatriciaNode {
            prefix: prefix.to_string(),
            children: HashMap::new(),
            is_leaf: true,
        }
    }

    pub fn insert(&mut self, key: &str) -> Result<(), PatriciaError> {
        let mut prefix_chars = self.prefix.chars().peekable();
        let mut key_chars = key.chars().peekable();

        let mut common_prefix = String::new();
        loop {
            match (prefix_chars.peek(), key_chars.peek()) {
                (Some(&p), Some(&k)) if p == k => {
                    prefix_chars.next();
                    key_chars.next();

                    common_prefix.push(p);
                }
                (None, None) => return Err(PatriciaError::KeyAlreadyExists),
                _ => break,
            }
        }

        match (prefix_chars.peek(), key_chars.peek()) {
            (None, Some(&k)) => {
                let new_node = PatriciaNode::new(&key_chars.collect::<String>());
                if let Some(child) = self.children.get_mut(&k) {
                    child.insert(&new_node.prefix)
                } else {
                    self.children.insert(k, Box::new(new_node));

                    Ok(())
                }
            }
            (Some(&p), None) => {
                let mut new_node = PatriciaNode::new(&prefix_chars.collect::<String>());
                new_node.children = mem::replace(&mut self.children, HashMap::new());

                self.prefix = common_prefix;
                self.children.insert(p, Box::new(new_node));

                Ok(())
            }
            (Some(&p), Some(&k)) => {
                let new_child_key = PatriciaNode::new(&key_chars.collect::<String>());
                let mut new_child_prefix = PatriciaNode::new(&prefix_chars.collect::<String>());
                new_child_prefix.children = mem::replace(&mut self.children, HashMap::new());

                self.prefix = common_prefix;
                self.is_leaf = false;
                self.children.insert(p, Box::new(new_child_prefix));
                self.children.insert(k, Box::new(new_child_key));

                Ok(())
            }
            _ => unreachable!(),
        }
    }

    pub fn search(&self, key: &str) -> bool {
        if !key.starts_with(&self.prefix) {
            return false;
        }
        if self.prefix == key && self.is_leaf {
            return true;
        }

        let key = &key[self.prefix.len()..];
        if let Some(child) = self.children.get(&key.chars().next().unwrap()) {
            child.search(key)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() -> Result<()> {
        let mut tree = PatriciaNode::new("apple");

        assert_eq!(tree.prefix, "apple");

        tree.insert("tea")?;

        assert_eq!(tree.prefix, "");
        assert_eq!(
            tree.children
                .iter()
                .map(|(_, child)| child.prefix.clone())
                .collect::<Vec<String>>(),
            vec!["apple".to_string(), "tea".to_string(),]
        );

        Ok(())
    }

    #[test]
    fn test_search() -> Result<()> {
        let mut tree = PatriciaNode::new("top");
        tree.insert("tea")?;
        tree.insert("teapot")?;
        tree.insert("tree")?;
        tree.insert("trie")?;
        tree.insert("teamwork")?;
        tree.insert("team")?;
        tree.insert("apple")?;

        println!("{:#?}", tree);

        assert!(tree.search("top"));
        assert!(tree.search("tea"));
        assert!(tree.search("teapot"));
        assert!(tree.search("tree"));
        assert!(tree.search("trie"));
        assert!(tree.search("teamwork"));
        assert!(tree.search("team"));
        assert!(tree.search("apple"));

        Ok(())
    }
}
