use anyhow::{anyhow, bail, ensure};
use clap::{App, Arg};
use digits_iterator::*;
use futures::executor::ThreadPool;
use itertools::Itertools;
use std::{convert::TryFrom, fs, iter};

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2019-5")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let program_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let program = parse_input(&program_str)?;

    let (max_thruster_val, max_phase_settings) = find_max_thruster_val(program.clone(), 5)?;

    println!(
        "Maximum thruster value: {} achieved with phase settings {:?}, without feedback loops",
        max_thruster_val, max_phase_settings
    );

    let (max_thruster_val, max_phase_settings) = find_max_thruster_val_looped(program.clone(), 5)?;

    println!(
        "Maximum thruster value: {} achieved with phase settings {:?}, with feedback loops",
        max_thruster_val, max_phase_settings
    );

    Ok(())
}

// Eric asks us to effectively implement Intcode multithreading,
// or at the very least concurrency. To which I say, "Hah! No."
// and use Rust futures, which makes for a really overengineered
// solution but whatever I wanted to learn about async in Rust anyway.
fn find_max_thruster_val_looped(
    program: Vec<isize>,
    num_amps: usize,
) -> Result<(isize, Vec<usize>), anyhow::Error> {
    ensure!(num_amps != 0, "Can't have 0 amplifiers");

    let mut thruster_outputs = vec![];

    // Yes I do realize that this only works because it effectively
    // just uses threads directly, and doesn't make use of task switching.
    // If you have a ThreadPool of size < num_amps, you'll keep blocking forever.
    // I intend to fix this... sometime, hopefully.
    let thread_pool = ThreadPool::builder()
        .pool_size(num_amps)
        .name_prefix("amplifier")
        .create()
        .map_err(|_| anyhow!("Unable to create thread pool"))?;

    for phase_settings in (5usize..=9).permutations(num_amps) {
        // We're using mpsc channels to set up a pipeline for the signals that goes
        // Main ═╦═ Amp 1 ══ Amp 2 ════ ... ════╦═ Main
        //       ╚══════════════════════════════╝
        // So we need to get the previous iteration's RX for input, and create a
        // new channel and use its TX for each amp's output.
        let (main_tx, first_rx) = flume::unbounded();

        let mut curr_rx = first_rx;

        for &current_phase_setting in phase_settings.iter() {
            let (output_tx, next_rx) = flume::unbounded();
            let input_rx = curr_rx;
            curr_rx = next_rx;

            let program = program.clone();
            let mut disconnected_tx = false;

            thread_pool.spawn_ok(async move {
                run_program(
                    program,
                    iter::once(current_phase_setting as isize).chain(input_rx),
                    |o| {
                        if !disconnected_tx {
                            if output_tx.send(o).is_err() {
                                disconnected_tx = true;
                                eprintln!("An amplifier has disconnected while output is still available.")
                            }
                        }
                    },
                ).unwrap();
            });
        }

        let main_rx = curr_rx;

        main_tx.send(0)?;

        for n in main_rx {
            // Loop back around, unless the first amplifier is done.
            if main_tx.send(n).is_err() {
                thruster_outputs.push((n, phase_settings));
                break;
            }
        }
    }

    thruster_outputs
        .into_iter()
        .max_by_key(|&(val, _)| val)
        .ok_or_else(|| anyhow!("Couldn't find a maximum thruster value"))
}

fn find_max_thruster_val(
    program: Vec<isize>,
    num_amps: usize,
) -> Result<(isize, Vec<usize>), anyhow::Error> {
    let mut thruster_outputs = vec![];

    for phase_settings in (0..=4).permutations(num_amps) {
        let thruster_val = (0..num_amps).try_fold(0, |acc, i| {
            let mut output = vec![];

            run_program(
                program.clone(),
                vec![phase_settings[i] as isize, acc],
                |o| output.push(o),
            )?;

            Ok::<_, anyhow::Error>(*output.last().ok_or_else(|| {
                anyhow!(
                    "Amplifier {} gave no output on phase settings {:?}",
                    i,
                    phase_settings
                )
            })?)
        })?;

        thruster_outputs.push((thruster_val, phase_settings));
    }

    thruster_outputs
        .into_iter()
        .max_by_key(|&(val, _)| val)
        .ok_or_else(|| anyhow!("Couldn't find a maximum thruster value"))
}

fn run_program(
    mut program: Vec<isize>,
    input: impl IntoIterator<Item = isize>,
    mut output_fn: impl FnMut(isize),
) -> Result<Vec<isize>, anyhow::Error> {
    let mut input = input.into_iter();

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
            3 | 4 => {
                match opcode % 100 {
                    3 => {
                        let input = input
                            .next()
                            .ok_or(anyhow!("Found an input opcode but no input was provided"))?;
                        let input_storage = get_param(0, true)? as usize;

                        program[input_storage] = input;
                    }
                    4 => output_fn(get_param(0, false)?),
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
