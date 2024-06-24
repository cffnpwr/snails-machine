use crate::state::{Direction, State};
use core::fmt;
use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    rc::Rc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    Running,
    Accept,
    Reject,
}
impl Display for Status {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Status::Running => write!(f, "Running"),
            Status::Accept => write!(f, "Accept"),
            Status::Reject => write!(f, "Reject"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub current_state: String,
    pub next_state: String,
    pub tape: Vec<String>,
    pub tape_ptr: usize,
    pub start_ptr: usize,
    pub read: String,
    pub write: String,
    pub status: Status,
}

#[derive(Debug, Clone)]
pub struct TuringMachine {
    pub current_state: Rc<RefCell<State>>,
    accept_states: Vec<Rc<RefCell<State>>>,
    pub tape: Vec<String>,
    pub tape_ptr: usize,
    pub start_ptr: usize,
    pub blank: String,
    pub status: Status,
    pub snapshots: Vec<Snapshot>,
}
impl TuringMachine {
    pub fn new(
        initial_state: &Rc<RefCell<State>>,
        accept_states: &[Rc<RefCell<State>>],
        tape: &[impl ToString],
        blank: impl ToString,
    ) -> Self {
        Self {
            current_state: initial_state.clone(),
            accept_states: accept_states.to_vec(),
            tape: tape.into_iter().map(|s| s.to_string()).collect(),
            tape_ptr: 0,
            start_ptr: 0,
            status: Status::Running,
            blank: blank.to_string(),
            snapshots: Vec::new(),
        }
    }
}
impl Iterator for TuringMachine {
    type Item = Rc<RefCell<State>>;

    fn next(&mut self) -> Option<Self::Item> {
        let read = self.tape[self.tape_ptr].clone();
        let (next_state, write, move_head) = self
            .current_state
            .borrow()
            .transition(&read)
            .map_err(|_| {
                self.status = match self
                    .accept_states
                    .iter()
                    .find(|&s| s.borrow().name == self.current_state.borrow().name)
                {
                    Some(_) => Status::Accept,
                    None => Status::Reject,
                };
            })
            .ok()?;
        self.snapshots.push(Snapshot {
            current_state: self.current_state.borrow().name.clone(),
            next_state: next_state.borrow().name.clone(),
            tape: self.tape.clone(),
            tape_ptr: self.tape_ptr,
            start_ptr: self.start_ptr,
            read: read.clone(),
            write: write.clone(),
            status: self.status.clone(),
        });

        self.current_state = next_state;
        self.tape[self.tape_ptr] = write;
        match move_head {
            Direction::Left => {
                if self.tape_ptr == 0 {
                    self.tape.insert(0, self.blank.clone());
                    self.start_ptr += 1;
                } else {
                    self.tape_ptr -= 1;
                }
            }
            Direction::Right => {
                self.tape_ptr += 1;
                if self.tape_ptr == self.tape.len() {
                    self.tape.push(self.blank.clone());
                }
            }
        }

        Some(self.current_state.clone())
    }
}
