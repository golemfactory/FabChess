#[cfg(target_arch = "wasm32")]
use fake_instant::FakeClock as Instant;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

pub mod uci_engine;
pub mod uci_parser;

use std::fs::File;
use std::io::{self, BufReader, BufWriter};

const ERROR_MSG: &str =
    "Invalid command line, expected: ./app [input] [output]. If omitted, stdin/stdout is assumed.";

fn main() -> Result<(), String> {
    let now = Instant::now();
    core_sdk::move_generation::magic::init_magics();
    core_sdk::board_representation::zobrist_hashing::init_at_program_start();
    core_sdk::search::init_constants();

    let new_now = Instant::now();
    println!(
        "{}",
        format!(
            "info string Initialization Time: {}ms",
            new_now.duration_since(now).as_secs() * 1000
                + u64::from(new_now.duration_since(now).subsec_millis())
        )
    );
    let args: Vec<_> = std::env::args().skip(1).collect();
    match &args[..] {
        [] => uci_parser::parse_loop(io::stdin().lock(), io::stdout()),
        [input, tail @ ..] => {
            let file = File::open(input)
                .map_err(|e| format!("opening the input file: {}: {}", input, e))?;
            let input = BufReader::new(file);
            match tail {
                [output] => {
                    let output = File::create(output)
                        .map_err(|e| format!("creating the output file: {}: {}", output, e))?;
                    let output = BufWriter::new(output);
                    uci_parser::parse_loop(input, output);
                }
                [] => uci_parser::parse_loop(input, io::stdout()),
                _ => return Err(ERROR_MSG.to_owned()),
            };
        }
    };
    Ok(())
}
