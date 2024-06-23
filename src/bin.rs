use anyhow::{anyhow, Result};
use clap::Parser;
use colored::Colorize;
use patricia_tree::PatriciaNode;
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

    let mut state_names = (config.transitions)
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

    for transition in &config.transitions {
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
        .into_iter()
        .map(|name| states.get(name.as_str()).unwrap().clone())
        .collect::<Vec<_>>();

    let mut alphabet = config
        .transitions
        .iter()
        .map(|t| [t.read.clone(), t.write.clone()])
        .flatten()
        .collect::<Vec<_>>();
    alphabet.sort();
    alphabet.dedup();

    let mut tm = TuringMachine::new(
        &initial_state,
        accept_states.as_slice(),
        &string_to_tape(
            &args.tape,
            alphabet.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
        )?,
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

fn string_to_tape<'a>(s: &str, alphabet: impl Into<Vec<&'a str>>) -> Result<Vec<String>> {
    let alphabet = alphabet.into();
    let mut tree = PatriciaNode::new(alphabet[0]);
    for name in alphabet.iter().skip(1) {
        tree.insert(name).unwrap();
    }

    let mut tape = vec![];
    let mut chars = s.chars().peekable();
    let mut buf = String::new();
    while let Some(&c) = chars.peek() {
        buf.push(c);
        match (buf.len() == 1, tree.search(&buf)) {
            (true, false) => {
                chars.next();
            }
            (false, false) => {
                buf.pop();
                if alphabet.contains(&buf.as_str()) {
                    tape.push(buf.clone());
                    buf.clear();
                } else {
                    chars.next();
                }
            }
            _ => {
                chars.next();
            }
        }
    }
    if !buf.is_empty() {
        if alphabet.contains(&buf.as_str()) {
            tape.push(buf);
        } else {
            return Err(anyhow!(
                "Invalid tape symbol: \"{}\". Tape symbol must be one of {}.",
                buf,
                alphabet
                    .iter()
                    .map(|a| format!("\"{}\"", a))
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
        }
    }

    Ok(tape)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_vec_string(v: Vec<&str>) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_string_to_tape() {
        let alphabet = ["a", "b", "c", "d", "e"];
        let tape = string_to_tape("abcde", &alphabet);

        assert!(tape.is_ok());
        assert_eq!(tape.unwrap(), to_vec_string(vec!["a", "b", "c", "d", "e"]));

        let alphabet = ["a", "a'", "#a", "a''", "b", "c"];
        let tape = string_to_tape("aaaabbbb#aa'a''", &alphabet);

        assert!(tape.is_ok());
        assert_eq!(
            tape.unwrap(),
            to_vec_string(vec![
                "a", "a", "a", "a", "b", "b", "b", "b", "#a", "a'", "a''"
            ])
        );

        let alphabet = ["a", "a'", "#a", "a''", "b", "c"];
        let tape = string_to_tape("ab#a'a''", &alphabet);

        assert!(tape.is_err());
    }
}
