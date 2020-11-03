use anyhow::{anyhow, bail, ensure};
use clap::{App, Arg};
use digits_iterator::*;
use itertools::Itertools;
use std::{convert::TryFrom, fs};

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2019-5")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let program_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let program = parse_input(&program_str)?;

    let (_, output) = run_program(program.clone(), vec![1])?;

    println!(
        "Diagnostic code for ID = 1: {}",
        output.last().ok_or(anyhow!("Program produced no output"))?
    );

    let (_, output) = run_program(program.clone(), vec![5])?;

    println!(
        "Diagnostic code for ID = 5: {}",
        output.last().ok_or(anyhow!("Program produced no output"))?
    );

    Ok(())
}

fn run_program(
    mut program: Vec<isize>,
    input: impl IntoIterator<Item = isize>,
) -> Result<(Vec<isize>, Vec<isize>), anyhow::Error> {
    let mut input = input.into_iter();
    let mut output = vec![];

    let mut instruction_pointer = 0;

    loop {
        let opcode = usize::try_from(program[instruction_pointer])
            .map_err(|_| anyhow!("Found a negative integer where an opcode was expected"))?;

        let parameter_modes = get_parameter_modes(opcode)?;

        let parameter_mode_of = |param: usize| {
            parameter_modes
                .get(param)
                .unwrap_or(&ParameterModes::Position)
        };

        let get_param = |param: usize, need_write: bool| {
            let param_value = program
                .get(instruction_pointer + param + 1)
                .copied()
                .ok_or(anyhow!("Parameter not found"))?;

            let param_mode = parameter_mode_of(param);

            if need_write {
                ensure!(
                    param_mode == &ParameterModes::Position,
                    "Invalid argument for opcode {}: {}",
                    opcode,
                    param_value
                );
            }

            Ok(match param_mode {
                ParameterModes::Position => {
                    let idx = usize::try_from(param_value).map_err(|_| {
                        anyhow!("Found a negative integer where a position param was expected")
                    })?;

                    if !need_write {
                        ensure!(
                            idx < program.len(),
                            "Invalid result index for opcode {}: {}",
                            opcode,
                            idx
                        );

                        program[idx]
                    } else {
                        param_value
                    }
                }
                ParameterModes::Immediate => param_value,
            })
        };

        match opcode % 100 {
            1 | 2 | 7 | 8 => {
                let (x, y, result_idx) = (
                    get_param(0, false)?,
                    get_param(1, false)?,
                    get_param(2, true)? as usize,
                );

                match opcode % 100 {
                    1 => program[result_idx] = x + y,
                    2 => program[result_idx] = x * y,
                    7 => program[result_idx] = (x < y) as isize,
                    8 => program[result_idx] = (x == y) as isize,
                    _ => unsafe { std::hint::unreachable_unchecked() },
                }

                instruction_pointer += 4;
            }
            5 | 6 => {
                let (checked_value, jump_point) = (
                    get_param(0, false)?,
                    usize::try_from(get_param(1, false)?).map_err(|_| {
                        anyhow!("Found a negative integer where a jump point was expected")
                    })?,
                );

                let should_jump = match opcode % 100 {
                    5 => checked_value != 0,
                    6 => checked_value == 0,
                    _ => unsafe { std::hint::unreachable_unchecked() },
                };

                if should_jump {
                    instruction_pointer = jump_point;
                } else {
                    instruction_pointer += 3;
                }
            }
            3 | 4 => {
                match opcode % 100 {
                    3 => {
                        let input = input
                            .next()
                            .ok_or(anyhow!("Found an input opcode but no input was provided"))?;
                        let input_storage = get_param(0, true)? as usize;

                        program[input_storage] = input;
                    }
                    4 => output.push(get_param(0, false)?),
                    _ => unsafe { std::hint::unreachable_unchecked() },
                }

                instruction_pointer += 2;
            }
            99 => return Ok((program, output)),
            op => bail!("Encountered an unknown opcode: {}", op),
        }
    }
}

fn get_parameter_modes(opcode: usize) -> Result<Vec<ParameterModes>, anyhow::Error> {
    opcode
        .digits()
        .rev()
        .skip(2)
        .map(ParameterModes::try_from)
        .try_collect()
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ParameterModes {
    Position,
    Immediate,
}

impl TryFrom<u8> for ParameterModes {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Position,
            1 => Self::Immediate,
            _ => bail!("Unknown parameter mode: {}", value),
        })
    }
}

fn parse_input(program_str: &str) -> Result<Vec<isize>, anyhow::Error> {
    program_str
        .split(",")
        .map(|num_str| {
            num_str
                .trim()
                .parse()
                .map_err(|_| anyhow!("Could not parse number in program as isize: '{}'", num_str))
        })
        .try_collect()
}
