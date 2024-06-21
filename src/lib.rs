use anyhow::Result;
pub use config::Config;
pub use state::{Direction, State, Transition};
pub use turing_machine::TuringMachine;

mod config;
mod state;
mod turing_machine;

// pub fn build_turing_machine() -> Result<TuringMachine> {}
