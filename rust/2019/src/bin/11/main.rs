#![feature(entry_insert, destructuring_assignment)]

use anyhow::{anyhow, bail, ensure};
use clap::{App, Arg};
use derive_more::From;
use digits_iterator::*;
use itertools::Itertools;
use std::{collections::HashMap, convert::TryFrom, fmt, fs, iter, sync::Mutex};
use tokio::{
    pin,
    stream::{self, Stream, StreamExt},
};

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2019-11")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let program_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let robot_program = parse_input(&program_str)?;

    let painted_hull = paint_hull(robot_program.clone(), HashMap::new(), Color::Black)?;

    println!(
        "Number of panels painted at least once: {}",
        painted_hull.len()
    );

    let registration_id_hull = paint_hull(
        robot_program,
        iter::once((Point::origin(), Color::White)).collect(),
        Color::Black,
    )?;

    print_hull(&registration_id_hull, Color::Black);

    Ok(())
}

fn print_hull(hull: &HashMap<Point, Color>, default_color: Color) {
    let ((min_x, max_x), (min_y, max_y)) = (
        hull.keys()
            .map(|p| p.x)
            .minmax()
            .into_option()
            .unwrap_or_default(),
        hull.keys()
            .map(|p| p.y)
            .minmax()
            .into_option()
            .unwrap_or_default(),
    );

    for y in (min_y..=max_y).rev() {
        for x in min_x..=max_x {
            if hull.get(&Point::new(x, y)).unwrap_or(&default_color) == &Color::Black {
                print!("█");
            } else {
                print!(" ");
            }
        }

        println!()
    }
}

fn paint_hull(
    robot_program: Vec<isize>,
    starting_hull: HashMap<Point, Color>,
    default_color: Color,
) -> Result<HashMap<Point, Color>, anyhow::Error> {
    use Color::*;
    use Direction::*;

    // Basically, we're using Mutex as a way of telling Rust that we know
    // for sure we aren't gonna be accessing these values concurrently.
    // The borrow checker is then satisfied.
    let hull = Mutex::new(starting_hull);
    let current_location = Mutex::new(Point::origin());
    let mut is_paint_output = true;
    let mut facing_direction = Up;

    futures_executor::block_on(run_program(
        robot_program,
        stream::iter(iter::from_fn(|| {
            let current_location = *(current_location.lock().unwrap());

            Some(
                hull.lock()
                    .unwrap()
                    .get(&current_location)
                    .copied()
                    .unwrap_or(default_color),
            )
        }))
        .map(|color| if color == Black { 0 } else { 1 }),
        |output| {
            let mut current_location = current_location.lock().unwrap();

            if is_paint_output {
                hull.lock()
                    .unwrap()
                    .entry(*current_location)
                    .insert(if output == 0 { Black } else { White });
            } else {
                let turn_direction = if output == 0 { Left } else { Right };

                (*current_location, facing_direction) = match (turn_direction, facing_direction) {
                    (Left, Right) | (Right, Left) => {
                        (Point::new(current_location.x, current_location.y + 1), Up)
                    }
                    (Left, Left) | (Right, Right) => {
                        (Point::new(current_location.x, current_location.y - 1), Down)
                    }
                    (Left, Up) | (Right, Down) => {
                        (Point::new(current_location.x - 1, current_location.y), Left)
                    }
                    (Left, Down) | (Right, Up) => (
                        Point::new(current_location.x + 1, current_location.y),
                        Right,
                    ),
                    _ => unsafe { std::hint::unreachable_unchecked() },
                }
            }

            is_paint_output = !is_paint_output;
        },
    ))?;

    Ok(hull.into_inner().unwrap())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Color {
    White,
    Black,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, From)]
struct Point {
    x: isize,
    y: isize,
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("").field(&self.x).field(&self.y).finish()
    }
}

impl Point {
    fn origin() -> Self {
        Self::new(0, 0)
    }

    fn new(x: isize, y: isize) -> Self {
        Self::from((x, y))
    }
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
