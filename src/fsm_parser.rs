use nom::{
    IResult, Parser,
    branch::permutation,
    bytes::complete::{is_a, is_not, tag},
    character::complete::{char, none_of},
    combinator::opt,
    multi::{many0, many1},
};
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum ParsedState {
    State(String),
    AcceptState(String),
}

#[derive(Debug, Clone)]
pub struct ParsedTransition {
    pub input: char,
    pub start_state: String,
    pub end_state: String,
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
pub struct ParsedFSM {
    pub start_state: String,
    pub states: Vec<ParsedState>,
    pub transitions: Vec<ParsedTransition>,
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
    let (i, _) = (line_parser, tag("start:"), blank_space_parser).parse(i)?;
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

impl ParsedFSM {
    pub fn parse(i: &str) -> IResult<&str, ParsedFSM> {
        definition_parser(i)
    }
}
