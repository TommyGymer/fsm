use nom::{
    IResult,
    Parser,
    character::complete::{char, none_of},
    bytes::complete::{tag, is_a, is_not},
    multi::{many0, many1},
    branch::{permutation, alt},
    combinator::opt,
};

#[derive(Debug)]
enum ParsedState {
    State(String),
    AcceptState(String),
}

#[derive(Debug)]
struct ParsedTransition {
    input: char,
    start_state: String,
    end_state: String,
}

#[derive(Debug)]
struct ParsedFSM {
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
    if let Ok((i, Some(_))) = (opt((char::<&str, nom::error::Error<&str>>('\t'), tag("final:")))).parse(i) {
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
    let (i, (_, c, _, _)) = (blank_space_parser, none_of(" \t\r\n:"), char(':'), blank_space_parser).parse(i)?;
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
            }
    ))
}

fn transitions_block_parser(i: &str) -> IResult<&str, Vec<ParsedTransition>> {
    let (i, _) = (line_parser, tag("transitions:"), line_parser).parse(i)?;
    many1(transition_parser).parse(i)
}

fn definition_parser(i: &str) -> IResult<&str, ParsedFSM> {
    let (i, (states, transitions)) = permutation((state_block_parser, transitions_block_parser)).parse(i)?;
    Ok((i,
            ParsedFSM {
                states,
                transitions
            }
    ))
}

fn main() {
    println!("{:?}", definition_parser.parse("states:\n\tA\n\tB\n\tfinal: C\n\ntransitions:\n\t0: A -> B\n\t0: B -> C\n\t0: C -> A"));
}
