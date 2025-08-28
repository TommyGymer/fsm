use color_eyre::Result;
use nom::{
    IResult, Parser,
    branch::permutation,
    bytes::complete::{is_a, is_not, tag},
    character::complete::{char, none_of},
    combinator::opt,
    multi::{many0, many1},
};
use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    io,
};

#[derive(Debug)]
enum ParsedState {
    State(String),
    AcceptState(String),
}

#[derive(Debug, Clone)]
struct ParsedTransition {
    input: char,
    start_state: String,
    end_state: String,
}

impl Display for ParsedTransition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} -> {}",
            self.input, self.start_state, self.end_state
        )
    }
}

#[derive(Debug)]
struct ParsedFSM {
    start_state: String,
    states: Vec<ParsedState>,
    transitions: Vec<ParsedTransition>,
}

fn line_parser(i: &str) -> IResult<&str, ()> {
    let (i, _) = many0(is_a("\r\n\0")).parse(i)?;
    Ok((i, ()))
}

fn blank_space_parser(i: &str) -> IResult<&str, ()> {
    let (i, _) = many0(is_a(" \t")).parse(i)?;
    Ok((i, ()))
}

fn state_name_parser(i: &str) -> IResult<&str, String> {
    let (i, name) = (is_not(" \t\r\n:")).parse(i)?;
    Ok((i, String::from(name)))
}

fn state_parser(i: &str) -> IResult<&str, ParsedState> {
    if let Ok((i, Some(_))) =
        (opt((char::<&str, nom::error::Error<&str>>('\t'), tag("final:")))).parse(i)
    {
        let (i, _) = blank_space_parser(i)?;
        let (i, name) = state_name_parser(i)?;
        let (i, _) = line_parser(i)?;
        Ok((i, ParsedState::AcceptState(String::from(name))))
    } else {
        let (i, _) = char('\t')(i)?;
        let (i, name) = state_name_parser(i)?;
        let (i, _) = line_parser(i)?;
        Ok((i, ParsedState::State(String::from(name))))
    }
}

fn state_block_parser(i: &str) -> IResult<&str, Vec<ParsedState>> {
    let (i, _) = (line_parser, tag("states:"), line_parser).parse(i)?;
    many0(state_parser).parse(i)
}

fn input_char_parser(i: &str) -> IResult<&str, char> {
    let (i, (_, c, _, _)) = (
        blank_space_parser,
        none_of(" \t\r\n:"),
        char(':'),
        blank_space_parser,
    )
        .parse(i)?;
    Ok((i, c))
}

fn transition_parser(i: &str) -> IResult<&str, ParsedTransition> {
    let (i, input) = input_char_parser(i)?;
    let (i, start_state) = state_name_parser(i)?;
    let (i, _) = (blank_space_parser, tag("->"), blank_space_parser).parse(i)?;
    let (i, end_state) = state_name_parser(i)?;
    let (i, _) = line_parser(i)?;
    Ok((
        i,
        ParsedTransition {
            input,
            start_state,
            end_state,
        },
    ))
}

fn transitions_block_parser(i: &str) -> IResult<&str, Vec<ParsedTransition>> {
    let (i, _) = (line_parser, tag("transitions:"), line_parser).parse(i)?;
    many1(transition_parser).parse(i)
}

fn start_block_parser(i: &str) -> IResult<&str, String> {
    let (i, _) = (line_parser, tag("start:")).parse(i)?;
    state_name_parser(i)
}

fn definition_parser(i: &str) -> IResult<&str, ParsedFSM> {
    let (i, (start_state, states, transitions)) = permutation((
        start_block_parser,
        state_block_parser,
        transitions_block_parser,
    ))
    .parse(i)?;
    Ok((
        i,
        ParsedFSM {
            start_state,
            states,
            transitions,
        },
    ))
}

#[derive(Debug, Eq, Clone)]
enum State {
    State(String),
    AcceptState(String),
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
struct FSM {
    start_state: Option<State>,
    states: HashSet<State>,
    transitions: HashSet<Transition>,
}

impl FSM {
    fn run(&self, input: String) -> bool {
        let mut current_state = self.start_state;
        for c in input.chars() {
            let transition_pattern = Transition {
                input: c,
                start_state: current_state,
                end_state: State::State(""),
            };
            let transition = self
                .transitions
                .get(transition_pattern)
                .expect("This should always exist as the FSM should be well formed");
        }
    }
}

#[derive(Debug)]
enum FSMError {
    MissingTransition(char, State),
    ExtraTransition(ParsedTransition, ParsedTransition),
    UnknownState(String),
    NoStartState,
}

impl std::error::Error for FSMError {}

impl Display for FSMError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::MissingTransition(c, state) => {
                write!(f, "Missing transition on '{}' from {}", c, state)
            }
            Self::ExtraTransition(a, b) => write!(f, "Transition {} and {} conflict", a, b),
            Self::UnknownState(name) => write!(f, "Unknown state '{}'", name),
            Self::NoStartState => write!(f, "No start state set"),
        }
    }
}

fn validate_parsed_fsm(parsed_fsm: ParsedFSM) -> Result<FSM, FSMError> {
    let mut fsm = FSM {
        start_state: None,
        states: HashSet::new(),
        transitions: HashSet::new(),
    };
    for state in parsed_fsm.states {
        match state {
            ParsedState::State(name) => {
                fsm.states.insert(State::State(name.to_owned()));
                if name == parsed_fsm.start_state {
                    fsm.start_state = Some(State::State(name))
                }
            }
            ParsedState::AcceptState(name) => {
                fsm.states.insert(State::AcceptState(name.to_owned()));
                if name == parsed_fsm.start_state {
                    fsm.start_state = Some(State::AcceptState(name))
                }
            }
        };
    }

    let mut input_alphabet: HashSet<char> = HashSet::new();
    for transition in &parsed_fsm.transitions {
        input_alphabet.insert(transition.input);
    }

    for input_character in input_alphabet {
        for state in &fsm.states {
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
                    if let Some(end_state) = fsm
                        .states
                        .iter()
                        .filter(|state| match state {
                            State::State(name) => name == &end_state_name,
                            State::AcceptState(name) => name == &end_state_name,
                        })
                        .collect::<Vec<&State>>()
                        .first()
                    {
                        fsm.transitions.insert(Transition {
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

    if fsm.start_state == None {
        return Err(FSMError::NoStartState);
    }

    Ok(fsm)
}

fn main() -> Result<()> {
    let (_, parsed_fsm) = definition_parser.parse("states:\n\tA\n\tB\n\tfinal: C\n\ntransitions:\n\t0: A -> B\n\t0: B -> C\n\t0: C -> A\n\t1: B -> A\n\t1: C -> B\n\t1: A -> C")?;
    let fsm = validate_parsed_fsm(parsed_fsm)?;
    println!("{:?}", fsm);

    println!("Enter input string:");
    let mut buffer = String::new();
    let stdin = io::stdin();
    stdin.read_line(&mut buffer)?;

    Ok(())
}
