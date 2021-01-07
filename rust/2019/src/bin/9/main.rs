use anyhow::{anyhow, bail, ensure};
use clap::{App, Arg};
use digits_iterator::*;
use itertools::Itertools;
use std::{convert::TryFrom, fs};
use tokio::pin;
use tokio_stream::{Stream, StreamExt};

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2019-9")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let program_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let program = parse_input(&program_str)?;

    let mut output = vec![];

    futures_executor::block_on(run_program(program.clone(), tokio_stream::once(1), |o| {
        output.push(o)
    }))?;

    println!("BOOST keycode: {:?}", output.first().ok_or_else(|| anyhow!("Invalid output for BOOST test mode"))?);

    output.clear();

    futures_executor::block_on(run_program(program.clone(), tokio_stream::once(2), |o| {
        output.push(o)
    }))?;

    println!("Distress coordinates: {:?}", output.first().ok_or_else(|| anyhow!("Invalid output for BOOST sensor mode"))?);

    Ok(())
}

async fn run_program(
    mut program: Vec<isize>,
    input: impl Stream<Item = isize>,
    mut output_fn: impl FnMut(isize),
) -> Result<Vec<isize>, anyhow::Error> {
    pin!(input);

    let mut instruction_pointer = 0;
    let mut relative_base = 0;

    loop {
        let opcode = usize::try_from(program[instruction_pointer])
            .map_err(|_| anyhow!("Found a negative integer where an opcode was expected"))?;

        let parameter_modes = get_parameter_modes(opcode)?;

        let parameter_mode_of = |param: usize| {
            parameter_modes
                .get(param)
                .unwrap_or(&ParameterModes::Position)
        };

        let mut get_param = |param: usize, need_write: bool| {
            let param_value = program
                .get(instruction_pointer + param + 1)
                .copied()
                .ok_or(anyhow!("Parameter not found"))?;

            let param_mode = parameter_mode_of(param);

            if need_write {
                ensure!(
                    [ParameterModes::Position, ParameterModes::Relative].contains(param_mode),
                    "Invalid argument for opcode {}: {}",
                    opcode,
                    param_value
                );
            }

            Ok(match param_mode {
                ParameterModes::Position | ParameterModes::Relative => {
                    let raw_idx = if param_mode == &ParameterModes::Relative {
                        relative_base + param_value
                    } else {
                        param_value
                    };

                    let idx = usize::try_from(raw_idx).map_err(|_| {
                        anyhow!(
                            "The program is attempting to access a negative index: {}",
                            raw_idx
                        )
                    })?;

                    if idx >= program.len() {
                        program.resize_with(idx + 1, || 0);
                    }

                    if !need_write {
                        program[idx]
                    } else {
                        raw_idx
                    }
                }
                ParameterModes::Immediate => param_value,
            })
        };

        // x % 100 gets the last 2 digits of a number,
        // no matter how long it is.
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
            3 | 4 | 9 => {
                match opcode % 100 {
                    3 => {
                        let input = input
                            .next()
                            .await
                            .ok_or(anyhow!("Found an input opcode but no input was provided"))?;
                        let input_storage = get_param(0, true)? as usize;

                        program[input_storage] = input;
                    }
                    4 => output_fn(get_param(0, false)?),
                    9 => relative_base += get_param(0, false)?,
                    _ => unsafe { std::hint::unreachable_unchecked() },
                }

                instruction_pointer += 2;
            }
            99 => return Ok(program),
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
    Relative,
}

impl TryFrom<u8> for ParameterModes {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Position,
            1 => Self::Immediate,
            2 => Self::Relative,
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
