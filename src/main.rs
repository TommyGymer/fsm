mod fsm;
mod fsm_parser;

use crate::{fsm::*, fsm_parser::*};
use clap::Parser;
use color_eyre::Result;
use std::{io, path::PathBuf};

/// A simple program to emulate finite state machines
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    fsm_file: PathBuf,

    #[arg(short, long)]
    input_string: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let (_, parsed_fsm) = ParsedFSM::parse(
        "states:\n\tA\n\tB\n\tfinal: C\n\ntransitions:\n\t0: A -> B\n\t0: B -> C\n\t0: C -> A\n\t1: B -> A\n\t1: C -> B\n\t1: A -> C\n\nstart: A",
    )?;
    let fsm = validate_parsed_fsm(parsed_fsm)?;

    println!("{}", fsm.run(args.input_string.trim_end().to_string())?);

    Ok(())
}
