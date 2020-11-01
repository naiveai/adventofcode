use anyhow::{anyhow, bail};
use clap::{App, Arg};
use itertools::Itertools;
use std::fs;

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2015-2")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .arg(
            Arg::from_usage("[required_value] -v, --required-value=<VALUE> 'Required value to produce for Part 2'")
                .default_value("19690720"),
        )
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let program_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let program = parse_input(&program_str)?;

    println!(
        "Program with input (12, 2): {}",
        run_program_with_inputs(&program, 12, 2)?[0],
    );

    let required_value = matches
        .value_of("required_value")
        .unwrap()
        .parse()
        .map_err(|_| anyhow!("Provided required value is not a number"))?;

    for (noun, verb) in (0usize..=99).permutations(2).map(|i| (i[0], i[1])) {
        if run_program_with_inputs(&program, noun, verb)?[0] == required_value {
            println!(
                "Program with input ({}, {}): {} (required value)",
                noun, verb, required_value
            );

            return Ok(());
        }
    }

    bail!(
        "Couldn't find a pair of inputs that produces {}",
        required_value
    );
}

fn run_program_with_inputs(
    program: &Vec<usize>,
    noun: usize,
    verb: usize,
) -> Result<Vec<usize>, anyhow::Error> {
    let mut modified_program = program.clone();

    modified_program[1] = noun;
    modified_program[2] = verb;

    run_program(modified_program)
}

fn run_program(mut program: Vec<usize>) -> Result<Vec<usize>, anyhow::Error> {
    let mut instruction_pointer = 0;

    loop {
        let instruction = program[instruction_pointer];

        match instruction {
            1 | 2 => {
                let ((x, y), result_idx) = (
                    (
                        program[program[instruction_pointer + 1]],
                        program[program[instruction_pointer + 2]],
                    ),
                    program[instruction_pointer + 3],
                );

                match instruction {
                    1 => program[result_idx] = x + y,
                    2 => program[result_idx] = x * y,
                    _ => unsafe { std::hint::unreachable_unchecked() },
                }

                instruction_pointer += 4;
            }
            99 => return Ok(program),
            op => bail!("Encountered an unknown opcode: {}", op),
        }
    }
}

fn parse_input(program_str: &str) -> Result<Vec<usize>, anyhow::Error> {
    program_str
        .split(",")
        .map(|opcode_str| {
            opcode_str
                .trim()
                .parse()
                .map_err(|_| anyhow!("Could not parse opcode as usize: '{}'", opcode_str))
        })
        .try_collect()
}
