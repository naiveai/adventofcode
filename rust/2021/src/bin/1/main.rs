use anyhow;
use clap::{Command, Arg};
use itertools::Itertools;
use std::{fs, num};

fn main() -> Result<(), anyhow::Error> {
    let matches = Command::new("2021-1")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .arg(Arg::from_usage("[group_length] 'Length of groups to compare for Part 2'").default_value("3"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();
    let group_length = matches.value_of("group_length").unwrap().parse::<usize>()?;

    let depth_measurements_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let depth_measurements = parse_input(&depth_measurements_str)?;

    let num_increases = find_depth_increases(&depth_measurements);

    println!("The depth increases {num_increases} times.");

    let num_summed_increases = find_summed_depth_increases(&depth_measurements, group_length);

    println!("In groups of {group_length}, the depths increase {num_summed_increases} times.");

    Ok(())
}

fn find_summed_depth_increases(depth_measurements: &Vec<usize>, group_length: usize) -> usize {
    let mut depth_increases = 0;
    let mut previous_sum = usize::MAX;

    for depths in depth_measurements.windows(group_length) {
        let sum = depths.iter().sum();

        if previous_sum < sum {
            depth_increases += 1;
        }

        previous_sum = sum;
    }

    depth_increases
}

fn find_depth_increases(depth_measurements: &Vec<usize>) -> usize {
    let mut depth_increases = 0;

    for depths in depth_measurements.windows(2) {
        // These are certain to be there but the Rust
        // type system is not yet smart enough to know that.
        if depths[0] < depths[1] {
            depth_increases += 1;
        }
    }

    depth_increases
}

fn parse_input(depth_measurements_str: &str) -> Result<Vec<usize>, num::ParseIntError> {
    depth_measurements_str
        .lines()
        .map(|depth_str| depth_str.parse())
        .try_collect()
}