use crate::fsm_parser::*;
use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};

#[derive(Debug, Eq, Clone)]
pub enum State {
    State(String),
    AcceptState(String),
}

impl Display for State {
    fn fmt<'a>(&self, f: &mut Formatter<'a>) -> fmt::Result {
        match self {
            Self::State(name) => write!(f, "State({})", name),
            Self::AcceptState(name) => write!(f, "AcceptState({})", name),
        }
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::State(self_name), Self::State(other_name)) => self_name == other_name,
            (Self::State(self_name), Self::AcceptState(other_name)) => self_name == other_name,
            (Self::AcceptState(self_name), Self::State(other_name)) => self_name == other_name,
            (Self::AcceptState(self_name), Self::AcceptState(other_name)) => {
                self_name == other_name
            }
        }
    }
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::State(name) => name.hash(state),
            Self::AcceptState(name) => name.hash(state),
        };
    }
}

impl State {
    fn get_name(&self) -> String {
        match self {
            Self::State(name) => name.to_owned(),
            Self::AcceptState(name) => name.to_owned(),
        }
    }
}

#[derive(Debug, Eq, Clone)]
struct Transition {
    input: char,
    start_state: State,
    end_state: State,
}

impl PartialEq for Transition {
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input && self.start_state == other.start_state
    }
}

impl Hash for Transition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.input.hash(state);
        self.start_state.hash(state);
    }
}

#[derive(Debug)]
pub struct FSM {
    start_state: State,
    transitions: HashSet<Transition>,
    input_alphabet: HashSet<char>,
}

impl FSM {
    pub fn run(&self, input: String) -> Result<bool, FSMError> {
        let mut current_state = self.start_state.to_owned();
        for c in input.chars() {
            if !self.input_alphabet.to_owned().contains(&c) {
                return Err(FSMError::CharNotInInputAlphabet(c));
            }
            let transition_pattern = Transition {
                input: c,
                start_state: current_state.to_owned(),
                end_state: State::State(String::from("")),
            };
            let transition = self
                .transitions
                .get(&transition_pattern)
                .expect("Transition should always exist if the FSM should be well formed");
            current_state = transition.end_state.to_owned();
        }
        Ok(match current_state {
            State::State(_) => false,
            State::AcceptState(_) => true,
        })
    }
}

#[derive(Debug)]
pub enum FSMError {
    MissingTransition(char, State),
    ExtraTransition(ParsedTransition, ParsedTransition),
    UnknownState(String),
    NoStartState,
    CharNotInInputAlphabet(char),
}

impl std::error::Error for FSMError {}

impl Display for FSMError {
    fn fmt<'a>(&self, f: &mut Formatter<'a>) -> fmt::Result {
        match self {
            Self::MissingTransition(c, state) => {
                write!(f, "Missing transition on '{}' from {}", c, state)
            }
            Self::ExtraTransition(a, b) => write!(f, "Transition {} and {} conflict", a, b),
            Self::UnknownState(name) => write!(f, "Unknown state '{}'", name),
            Self::NoStartState => write!(f, "No start state set"),
            Self::CharNotInInputAlphabet(c) => write!(f, "'{}' not in input alphabet", c),
        }
    }
}

pub fn validate_parsed_fsm(parsed_fsm: ParsedFSM) -> Result<FSM, FSMError> {
    let mut start_state = None;
    let mut states = HashSet::new();
    let mut transitions = HashSet::new();

    for state in parsed_fsm.states {
        match state {
            ParsedState::State(name) => {
                states.insert(State::State(name.to_owned()));
                if name == parsed_fsm.start_state {
                    start_state = Some(State::State(name))
                }
            }
            ParsedState::AcceptState(name) => {
                states.insert(State::AcceptState(name.to_owned()));
                if name == parsed_fsm.start_state {
                    start_state = Some(State::AcceptState(name))
                }
            }
        };
    }

    let mut input_alphabet: HashSet<char> = HashSet::new();
    for transition in &parsed_fsm.transitions {
        input_alphabet.insert(transition.input);
    }

    for input_character in input_alphabet.to_owned() {
        for state in states.to_owned() {
            let found: Vec<&ParsedTransition> = parsed_fsm
                .transitions
                .iter()
                .filter(|t| t.input == input_character && t.start_state == state.get_name())
                .collect();
            match found.len() {
                0 => {
                    return Err(FSMError::MissingTransition(
                        input_character,
                        state.to_owned(),
                    ));
                }
                1 => {
                    let end_state_name = found.first().unwrap().end_state.to_owned();
                    if let Some(end_state) = states
                        .iter()
                        .filter(|state| match state {
                            State::State(name) => name == &end_state_name,
                            State::AcceptState(name) => name == &end_state_name,
                        })
                        .collect::<Vec<&State>>()
                        .first()
                    {
                        transitions.insert(Transition {
                            input: input_character,
                            start_state: state.to_owned(),
                            end_state: end_state.to_owned().to_owned(),
                        });
                    } else {
                        return Err(FSMError::UnknownState(end_state_name));
                    }
                }
                2.. => {
                    return Err(FSMError::ExtraTransition(
                        found.get(0).unwrap().to_owned().to_owned(),
                        found.get(1).unwrap().to_owned().to_owned(),
                    ));
                }
            }
        }
    }

    if let Some(start_state) = start_state {
        Ok(FSM {
            start_state,
            transitions,
            input_alphabet,
        })
    } else {
        return Err(FSMError::NoStartState);
    }
}
