use anyhow::{anyhow, Result};
use clap::Parser;
use colored::Colorize;
use console::Term;
use log::info;
use patricia_tree::PatriciaNode;
use snails_machine::{Config, State, Transition, TuringMachine};
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{stdout, Write as _},
    rc::Rc,
};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the Turing machine definition file
    #[arg(short = 'f', long = "file", default_value = "./machine.toml")]
    machine_file_path: String,

    /// Whether to use monospace font
    #[arg(short = 'm', long = "monospace")]
    is_monospace: bool,

    /// Whether to show tape separator (Show '|' between tape symbols)
    #[arg(short = 's', long = "separator")]
    show_separator: bool,

    /// Whether to use snail mode
    #[arg(long = "snail")]
    is_snail_mode: bool,

    /// Whether to show step-by-step execution
    #[arg(short = 'S', long = "step-by-step")]
    is_step_by_step: bool,

    /// Initial tape content
    tape: String,
}

fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let args = Args::parse();
    info!("Monospace mode: {}", args.is_monospace);
    info!("Show separator: {}", args.show_separator);
    info!("Snail mode: {}", args.is_snail_mode);
    info!("Step-by-step mode: {}", args.is_step_by_step);
    info!(
        "Loading Turing machine definition from \"{}\"...",
        args.machine_file_path
    );

    let config = Config::read_from_file(args.machine_file_path)?;
    let mut tm = turing_machine_from_config(&config, args.tape)?;
    while let Some(_) = tm.next() {}

    let separator = if args.show_separator { "|" } else { "" };
    let tape_len = tm.tape.len();
    let offset = tm.start_ptr;
    let mut max_tape_symbol_lens = vec![1; tm.tape.len()];
    for snapshot in &tm.snapshots {
        for (i, s) in snapshot.tape.iter().enumerate() {
            let i = i + (offset - snapshot.start_ptr);
            max_tape_symbol_lens[i] = max_tape_symbol_lens
                .get(i)
                .map_or(s.len(), |len| *len.max(&s.len()));
        }
    }
    let max_tape_symbol_len = *max_tape_symbol_lens.iter().max().unwrap_or(&1);
    let get_max_tape_symbol_len = |i: usize| {
        if args.is_monospace {
            max_tape_symbol_len
        } else {
            max_tape_symbol_lens[i]
        }
    };
    let blank = if args.is_monospace {
        tm.blank.repeat(max_tape_symbol_len)
    } else {
        tm.blank.to_string()
    };

    let term = Term::stdout();
    for snapshot in &tm.snapshots {
        let tape = (&snapshot.tape).clone();
        let tape_ptr = snapshot.tape_ptr;
        let start_ptr = snapshot.start_ptr;
        let tape = build_tape_string(
            tape,
            tape_ptr,
            offset - start_ptr,
            &blank,
            separator,
            tape_len,
            get_max_tape_symbol_len,
        );

        if args.is_snail_mode {
            let ptr = tape_ptr + offset - start_ptr;
            let blanks = (0..ptr)
                .map(|i| {
                    " ".repeat(get_max_tape_symbol_len(i) + if args.show_separator { 1 } else { 0 })
                })
                .collect::<String>();
            println!("[{}]", tape);
            println!(
                " {blanks}🐌<[({}, {}) -> ({}, {})]",
                snapshot.current_state, snapshot.read, snapshot.next_state, snapshot.write,
            );
        } else {
            println!(
                "{:>7}: [{}]: ({}, {}) -> ({}, {})",
                snapshot.status,
                tape,
                snapshot.current_state,
                snapshot.read,
                snapshot.next_state,
                snapshot.write,
            );
        }

        if args.is_step_by_step {
            while {
                term.clear_line()?;
                print!("Press Enter to continue...");
                stdout().flush()?;
                let key = term.read_key()?;
                key != console::Key::Enter
            } {}
            term.clear_line()?;
        }
    }

    let tape = tm.tape;
    let tape_ptr = tm.tape_ptr;
    let start_ptr = tm.start_ptr;
    let tape = build_tape_string(
        tape,
        tape_ptr,
        offset - start_ptr,
        &blank,
        separator,
        tape_len,
        get_max_tape_symbol_len,
    );
    let status = tm.status.to_string();
    if args.is_snail_mode {
        let ptr = tape_ptr + offset - start_ptr;
        let blanks = (0..ptr)
            .map(|i| {
                " ".repeat(get_max_tape_symbol_len(i) + if args.show_separator { 1 } else { 0 })
            })
            .collect::<String>();
        println!("[{}]", tape);
        println!(" {blanks}🐌<[{}]", status)
    } else {
        println!("{:>7}: [{}]", status, tape);
    }

    Ok(())
}

fn turing_machine_from_config(config: &Config, tape: String) -> Result<TuringMachine> {
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
    let accept_states = (&config.accept_states)
        .into_iter()
        .map(|name| states.get(name.as_str()))
        .collect::<Vec<_>>();
    if accept_states.iter().any(|s| s.is_none()) {
        return Err(anyhow!(
            "Invalid accept state: {}. Accept state must be one of {}.",
            config
                .accept_states
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<String>>()
                .join(", "),
            state_names
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<String>>()
                .join(", ")
        ));
    }
    let accept_states = accept_states
        .into_iter()
        .map(|s| s.unwrap().clone())
        .collect::<Vec<_>>();

    let mut alphabet = config
        .transitions
        .iter()
        .map(|t| [t.read.clone(), t.write.clone()])
        .flatten()
        .collect::<Vec<_>>();
    alphabet.sort();
    alphabet.dedup();

    let tm = TuringMachine::new(
        &initial_state,
        accept_states.as_slice(),
        &string_to_tape(
            &tape,
            alphabet.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
        )?,
        config.blank.as_str(),
    );

    Ok(tm)
}

fn build_tape_string(
    tape: Vec<String>,
    tape_ptr: usize,
    offset: usize,
    blank: &str,
    separator: &str,
    tape_len: usize,
    get_max_tape_symbol_len: impl Fn(usize) -> usize,
) -> String {
    let tape = tape
        .into_iter()
        .enumerate()
        .map(|(i, s)| {
            let i = i + offset;
            let len = get_max_tape_symbol_len(i);
            let s = if s[..1] == blank[..1] {
                s.repeat(len)
            } else {
                s
            };
            format!("{:<len$}", s)
        })
        .collect::<Vec<_>>();

    let mut tmp = vec![blank.to_string(); offset];
    tmp.extend(tape.clone());
    tmp.extend(vec![blank.to_string(); tape_len - tmp.len()]);
    let mut tape = tmp;

    let tape_ptr = tape_ptr + offset;
    let s = &tape[tape_ptr];
    tape[tape_ptr] = s.reversed().green().to_string();
    let tape = tape.join(separator);

    tape
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
