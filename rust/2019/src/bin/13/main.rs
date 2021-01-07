#![feature(default_free_fn, duration_zero)]

use anyhow::{anyhow, bail, ensure, Context};
use atomic::Atomic;
use clap::{App, Arg};
use colored::*;
use crossterm::{
    cursor, execute, style,
    terminal::{Clear, ClearType},
};
use derive_more::From;
use digits_iterator::*;
use itertools::Itertools;
use parking_lot::Mutex;
use std::{
    cmp::Ordering,
    collections::HashMap,
    convert::TryFrom,
    default::default,
    fmt, fs,
    io::{stdin, stdout, Write},
    iter, panic, process,
    sync::{
        atomic::{AtomicBool, AtomicIsize, Ordering::*},
        Arc,
    },
    thread,
    time::Duration,
};
use tokio::pin;
use tokio_stream::{Stream, StreamExt};

fn main() -> Result<(), anyhow::Error> {
    // Because we're doing fancy terminal stuff here, we should
    // set up sudden-termination handlers that clean up properly
    let game_running = Arc::new(AtomicBool::new(false));
    let game_running_ctrlc = game_running.clone();
    let game_running_panic = game_running.clone();

    ctrlc::set_handler(move || {
        if game_running_ctrlc.load(Acquire) {
            // Don't care if this failed, the exit needs to happen anyway.
            let _ = game_exit_handler();
        }

        process::exit(1);
    })?;

    panic::set_hook(Box::new(move |info| {
        if game_running_panic.load(Acquire) {
            // We can't panic *again* in here.
            let _ = game_exit_handler();
        }

        println!("{}", info);
    }));

    let matches = App::new("2019-13")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .arg(Arg::from_usage("[draw_intermediate] -d --draw-intermediate 'Draw the screen while the game is running'").takes_value(false))
        .arg(Arg::from_usage("[draw_fast] -f --draw-fast 'Speed the game up while drawing it'").takes_value(false))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let program_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let mut game_program = parse_input(&program_str)?;

    let (screen, _) = run_game(game_program.clone(), |_, _| JoystickInput::Neutral, None)?;

    println!(
        "Number of block tiles with no quarters: {}",
        screen.values().filter(|&tile| tile == &Tile::Block).count(),
    );

    let mut input = String::new();
    print!("Insert 2 quarters? (Y/n) ");
    stdout().flush()?;
    stdin().read_line(&mut input)?;

    let input = input.trim();

    if !(input.is_empty() || input.to_lowercase() == "y") {
        return Ok(());
    }

    // HACKERMAN
    game_program[0] = 2;

    game_running.store(true, Release);

    let (_, score) = run_game(
        game_program,
        |paddle_pos, ball_pos| {
            use JoystickInput::*;

            match ball_pos.x.cmp(&paddle_pos.x) {
                Ordering::Less => Left,
                Ordering::Greater => Right,
                Ordering::Equal => Neutral,
            }
        },
        if matches.is_present("draw_intermediate") {
            Some(if matches.is_present("draw_fast") {
                Duration::ZERO
            } else {
                Duration::from_millis(50)
            })
        } else {
            None
        },
    )?;

    println!("Final score: {}", score);

    Ok(())
}

#[derive(Copy, Clone)]
enum JoystickInput {
    Neutral,
    Left,
    Right,
}

fn run_game(
    game_program: Vec<isize>,
    mut input_fn: impl FnMut(Point, Point) -> JoystickInput,
    should_draw: Option<Duration>,
) -> Result<(HashMap<Point, Tile>, isize), anyhow::Error> {
    let screen = Mutex::new(HashMap::new());
    let current_score = Arc::new(AtomicIsize::new(0));
    let current_ball_pos = Arc::new(Atomic::new(default()));
    let current_paddle_pos = Arc::new(Atomic::new(default()));

    let current_score_input = if should_draw.is_some() {
        Some(current_score.clone())
    } else {
        None
    };
    let current_ball_pos_input = current_ball_pos.clone();
    let current_paddle_pos_input = current_paddle_pos.clone();

    // These are only accessed from the output closure, and
    // therefore don't need any synchronization.
    let mut current_tile_pos = Point::default();
    let mut current_screen_instruction = 0_u8;

    let mut stdout = stdout();

    if should_draw.is_some() {
        execute!(stdout, cursor::Hide).unwrap();
    }

    futures_executor::block_on(run_program(
        game_program,
        tokio_stream::iter(iter::from_fn(|| {
            if let Some(pause_duration) = should_draw {
                let screen_str = screen_to_string(&screen.lock());
                let current_score = current_score_input.as_ref().unwrap().load(Acquire);

                execute!(
                    stdout,
                    cursor::SavePosition,
                    style::Print(screen_str),
                    style::Print(format!(
                        "Score: {}\n",
                        current_score.to_string().underline()
                    )),
                    cursor::RestorePosition,
                )
                .unwrap();

                stdout.flush().unwrap();

                // Yes, we do this even if pause_duration.is_zero(), because
                // this will allow the OS to update the terminal before we
                // start printing it again. This is different from flushing
                // for reasons that I really can't understand.
                thread::sleep(pause_duration);
            }

            use JoystickInput::*;

            let joystick_input = input_fn(
                current_paddle_pos_input.load(Acquire),
                current_ball_pos_input.load(Acquire),
            );

            Some(match joystick_input {
                Neutral => 0,
                Left => -1,
                Right => 1,
            })
        })),
        |output| {
            if current_screen_instruction == 0 {
                current_tile_pos.x = output;

                current_screen_instruction = 1;
            } else if current_screen_instruction == 1 {
                current_tile_pos.y = output;

                current_screen_instruction = 2;
            } else if current_screen_instruction == 2 {
                if current_tile_pos == Point::new(-1, 0) {
                    current_score.store(output, Release);
                } else {
                    let tile = Tile::try_from(output as u8).unwrap();

                    if let Tile::Ball = tile {
                        current_ball_pos.store(current_tile_pos, Release);
                    } else if let Tile::Paddle = tile {
                        current_paddle_pos.store(current_tile_pos, Release);
                    }

                    screen.lock().insert(current_tile_pos, tile);
                }

                current_screen_instruction = 0;
            }
        },
    ))?;

    let screen = screen.into_inner();
    let score = current_score.load(Acquire);

    if should_draw.is_some() {
        let screen_str = screen_to_string(&screen);
        execute!(
            stdout,
            style::Print(screen_str),
            style::Print(format!("Score: {}\n", score.to_string().underline())),
        )
        .unwrap();

        execute!(stdout, cursor::Show).unwrap();
    }

    Ok((screen, score))
}

fn game_exit_handler() -> Result<(), anyhow::Error> {
    execute!(stdout(), Clear(ClearType::FromCursorDown), cursor::Show)?;

    Ok(())
}

fn screen_to_string(screen: &HashMap<Point, Tile>) -> String {
    let ((min_x, max_x), (min_y, max_y)) = (
        screen
            .keys()
            .map(|p| p.x)
            .minmax()
            .into_option()
            .unwrap_or_default(),
        screen
            .keys()
            .map(|p| p.y)
            .minmax()
            .into_option()
            .unwrap_or_default(),
    );

    let mut screen_str = String::new();

    use Tile::*;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            screen_str.push_str(&*match screen.get(&Point::new(x, y)).unwrap_or(&Empty) {
                Empty => " ".to_string(),
                Wall => "█".black().bold().to_string(),
                Block => "░".red().to_string(),
                Paddle => "_".bright_yellow().to_string(),
                Ball => "o".bright_green().bold().to_string(),
            });
        }

        screen_str.push('\n');
    }

    screen_str
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, From, Default)]
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
    fn new(x: isize, y: isize) -> Self {
        Self::from((x, y))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball,
}

impl TryFrom<u8> for Tile {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Empty,
            1 => Self::Wall,
            2 => Self::Block,
            3 => Self::Paddle,
            4 => Self::Ball,
            _ => bail!("Unknown parameter mode: {}", value),
        })
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
            .context("Found a negative integer where an opcode was expected")?;

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

                    let idx = usize::try_from(raw_idx).with_context(|| {
                        format!(
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
                    usize::try_from(get_param(1, false)?)
                        .context("Found a negative integer where a jump point was expected")?,
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
            num_str.trim().parse().with_context(|| {
                format!("Could not parse number in program as isize: '{}'", num_str)
            })
        })
        .try_collect()
}
