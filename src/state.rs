use serde::Deserialize;
use std::{cell::RefCell, rc::Rc};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Turing machine is stuck")]
pub struct Stuck;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Direction {
    #[serde(
        alias = "left",
        alias = "L",
        alias = "l",
        alias = "<-",
        alias = "<",
        alias = "←"
    )]
    Left,

    #[serde(
        alias = "right",
        alias = "R",
        alias = "r",
        alias = "->",
        alias = ">",
        alias = "→"
    )]
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    to: Rc<RefCell<State>>,
    read: String,
    write: String,
    move_head: Direction,
}
impl Transition {
    pub fn new(to: &Rc<RefCell<State>>, read: &str, write: &str, move_head: Direction) -> Self {
        Transition {
            to: to.clone(),
            read: read.to_string(),
            write: write.to_string(),
            move_head,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub name: String,
    transitions: Vec<Transition>,
}
impl State {
    pub fn new(name: &str, transitions: impl AsRef<[Transition]>) -> Self {
        Self {
            name: name.to_string(),
            transitions: transitions.as_ref().to_vec(),
        }
    }

    pub fn add_transition(&mut self, transition: Transition) {
        self.transitions.push(transition);
    }

    pub fn add_transitions(&mut self, transitions: impl AsRef<[Transition]>) {
        self.transitions.extend_from_slice(transitions.as_ref());
    }

    pub fn transition(&self, read: &str) -> Result<(Rc<RefCell<State>>, String, Direction), Stuck> {
        let transition = self.transitions.iter().find(|&t| t.read == read);
        match transition {
            Some(transition) => Ok((
                transition.to.clone(),
                transition.write.clone(),
                transition.move_head.clone(),
            )),
            None => Err(Stuck),
        }
    }
}
