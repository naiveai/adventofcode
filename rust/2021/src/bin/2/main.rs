use anyhow::bail;
use clap::{Command, Arg};
use itertools::Itertools;
use std::fs;

fn main() -> Result<(), anyhow::Error> {
    let matches = Command::new("2021-2")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let submarine_instructions_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let submarine_instructions = parse_input(&submarine_instructions_str)?;

    let (final_position, final_depth) = path_simple(0, 0, &submarine_instructions);

    println!("You'll end up at ({final_position}, {final_depth}) with the simple approach.");

    let (final_position, final_depth, _) = path_with_aim(0, 0, 0, &submarine_instructions);

    println!("Taking into account aim, you'll end up at ({final_position}, {final_depth})");

    Ok(())
}

fn path_with_aim(initial_position: usize, initial_depth: usize,
    initial_aim: usize, submarine_instructions: &Vec<Instruction>)
    -> (usize, usize, usize) {
    let mut current_position = initial_position;
    let mut current_depth = initial_depth;
    let mut current_aim = initial_aim;

    for instruction in submarine_instructions {
        match instruction {
            Instruction::Forward(units) => {
                current_position += units;
                current_depth += current_aim * units;
            },
            Instruction::Down(units) => current_aim += units,
            Instruction::Up(units) => current_aim -= units,
        }
    }

    (current_position, current_depth, current_aim)
}

fn path_simple(initial_position: usize, initial_depth: usize, submarine_instructions: &Vec<Instruction>)
    -> (usize, usize) {
    let mut current_position = initial_position;
    let mut current_depth = initial_depth;

    for instruction in submarine_instructions {
        match instruction {
            Instruction::Forward(units) => current_position += units,
            Instruction::Down(units) => current_depth += units,
            Instruction::Up(units) => current_depth -= units,
        }
    }

    (current_position, current_depth)
}

fn parse_input(submarine_instructions_str: &str) -> Result<Vec<Instruction>, anyhow::Error> {
    submarine_instructions_str
        .lines()
        .map(|instruction_str| {
            let instruction = instruction_str.split(" ").collect_vec();
            let direction_str = instruction[0];
            let units_str = instruction[1];

            let units = units_str.parse()?;
            let direction = match direction_str {
                "forward" => Instruction::Forward(units),
                "down" => Instruction::Down(units),
                "up" => Instruction::Up(units),
                _ => bail!("Invalid instruction")
            };

            Ok(direction)
        })
        .try_collect()
}

#[derive(Debug)]
enum Instruction {
    Forward(usize),
    Down(usize),
    Up(usize)
}