use crate::{Direction, State, Transition, TuringMachine};
use anyhow::{anyhow, bail, Context, Result};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("State {0} is not found")]
    StateNotFound(String),
}

pub struct TuringMachineBuilder<'a> {
    states: HashMap<&'a str, Rc<RefCell<State>>>,
    initial_state: Option<Rc<RefCell<State>>>,
    accept_states: Vec<Rc<RefCell<State>>>,
    tape: Vec<String>,
    blank_symbol: Option<&'a str>,
}
impl<'a> TuringMachineBuilder<'a> {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            initial_state: None,
            accept_states: vec![],
            tape: vec![],
            blank_symbol: None,
        }
    }

    pub fn add_state(mut self, name: &'a str) -> Self {
        self.states
            .insert(name, Rc::new(RefCell::new(State::new(name, vec![]))));

        self
    }

    pub fn add_states(mut self, names: &[&'a str]) -> Self {
        self.states.extend(
            names
                .iter()
                .map(|&name| (name, Rc::new(RefCell::new(State::new(name, vec![])))))
                .collect::<HashMap<_, _>>(),
        );

        self
    }

    pub fn add_transition(
        self,
        from: &'a str,
        to: &'a str,
        read: &'a str,
        write: &'a str,
        direction: Direction,
    ) -> Result<Self> {
        let from = self
            .states
            .get(from)
            .ok_or(BuilderError::StateNotFound(from.to_string()))?;
        let to = self
            .states
            .get(to)
            .ok_or(BuilderError::StateNotFound(to.to_string()))?;
        from.borrow_mut()
            .add_transition(Transition::new(to, read, write, direction));

        Ok(self)
    }

    pub fn add_transitions(
        self,
        transitions: &[(&'a str, &'a str, &'a str, &'a str, Direction)],
    ) -> Result<Self> {
        for (from, to, read, write, direction) in transitions {
            let from = self
                .states
                .get(from)
                .ok_or(BuilderError::StateNotFound(from.to_string()))?;
            let to = self
                .states
                .get(to)
                .ok_or(BuilderError::StateNotFound(to.to_string()))?;
            let transition = Transition::new(to, read, write, direction.clone());

            from.borrow_mut().add_transition(transition);
        }

        Ok(self)
    }

    pub fn set_initial_state(mut self, name: &'a str) -> Result<Self> {
        self.initial_state = match self.states.get(name) {
            Some(state) => Some(state.clone()),
            None => bail!(BuilderError::StateNotFound(name.to_string())),
        };

        Ok(self)
    }

    pub fn set_accept_states(mut self, names: &[&'a str]) -> Result<Self> {
        self.accept_states = names
            .iter()
            .map(|&name| {
                self.states
                    .get(name)
                    .map(|state| state.clone())
                    .ok_or(anyhow!(BuilderError::StateNotFound(name.to_string())))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(self)
    }

    pub fn set_tape(mut self, tape: Vec<String>) -> Self {
        self.tape = tape;

        self
    }

    pub fn set_blank_symbol(mut self, blank_symbol: &'a str) -> Self {
        self.blank_symbol = Some(blank_symbol);

        self
    }

    pub fn build(self) -> Result<TuringMachine> {
        let initial_state = self.initial_state.context("Initial state is not set")?;
        let accept_states = self.accept_states;
        let tape = self.tape;
        let blank_symbol = self.blank_symbol.context("Blank symbol is not set")?;

        let tm = TuringMachine::new(
            &initial_state,
            accept_states.as_slice(),
            &tape,
            blank_symbol,
        );

        Ok(tm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::turing_machine::Status;

    #[test]
    fn test_builder() -> Result<()> {
        let tm = TuringMachineBuilder::new()
            .add_states(&["A", "B"])
            .add_transition("A", "B", "0", "1", Direction::Right)?
            .set_initial_state("A")?
            .set_accept_states(&["B"])?
            .set_tape(vec!["0".to_string(), "1".to_string()])
            .set_blank_symbol("0")
            .build()
            .unwrap();

        let mut tm = tm;
        while let Some(_) = tm.next() {}

        assert_eq!(tm.tape, vec!["1".to_string(), "1".to_string()]);
        assert_eq!(tm.status, Status::Accept);

        Ok(())
    }
}
