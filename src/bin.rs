use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use snails_machine::{Config, State, Transition, TuringMachine};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the Turing machine definition file
    #[arg(short = 'f', long = "file", default_value = "machine.toml")]
    machine_file_path: String,

    /// Initial tape content
    tape: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::read_from_file(args.machine_file_path)?;

    let mut state_names = config
        .transitions
        .iter()
        .map(|t| [t.from.clone(), t.to.clone()])
        .flatten()
        .collect::<Vec<_>>();
    state_names.sort();
    state_names.dedup();
    let states = state_names
        .iter()
        .map(|name| {
            (
                name.as_str(),
                Rc::new(RefCell::new(State::new(name, vec![]))),
            )
        })
        .collect::<HashMap<_, _>>();

    for transition in config.transitions {
        let from = states.get(&transition.from.as_str()).unwrap();
        let to = states.get(&transition.to.as_str()).unwrap();
        from.borrow_mut().add_transition(Transition::new(
            to,
            transition.read.as_str(),
            transition.write.as_str(),
            transition.direction,
        ));
    }

    let initial_state = states.get(&config.initial_state.as_str()).unwrap();
    let accept_states = config
        .accept_states
        .iter()
        .map(|name| states.get(&name.as_str()).unwrap())
        .collect::<Vec<_>>();
    let accept_states = accept_states
        .into_iter()
        .map(|s| s.clone())
        .collect::<Vec<_>>();

    let mut tm = TuringMachine::new(
        &initial_state,
        accept_states.as_slice(),
        &args.tape.chars().map(|c| c.to_string()).collect::<Vec<_>>(),
        config.blank.as_str(),
    );

    while let Some(_) = tm.next() {}

    let tape_len = tm.tape.len();
    let offset = tm.start_ptr;
    for mut snapshot in tm.snapshots {
        let s = &snapshot.tape[snapshot.tape_ptr];
        snapshot.tape[snapshot.tape_ptr] = s.reversed().green().to_string();
        let tape = snapshot.tape.join("");
        let tape = format!("{}{}", tm.blank.repeat(offset - snapshot.start_ptr), tape,);

        println!(
            "{:>7}: [{}{blanks}]: ({}, {}) -> ({}, {})",
            snapshot.status,
            tape,
            snapshot.current_state,
            snapshot.read,
            snapshot.next_state,
            snapshot.write,
            blanks = tm.blank.repeat(tape_len - (tape.len() - 11))
        );
    }

    let s = &tm.tape[tm.tape_ptr];
    tm.tape[tm.tape_ptr] = s.reversed().green().to_string();
    println!("{:>7}: [{}]", tm.status.to_string(), tm.tape.join(""),);

    Ok(())
}
