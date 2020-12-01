use anyhow::anyhow;
use clap::{App, Arg};
use itertools::Itertools;
use std::{fs, num};

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2020-1")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .arg(
            Arg::from_usage(
                "[required_sum] -s --req-sum 'Will find values that sum to this number'",
            )
            .default_value("2020"),
        )
        .arg(
            Arg::from_usage(
                "[num_parts] -n --num-parts 'Will use these many numbers to add up to the required sum'",
            )
            .default_value("2"),
        )
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();
    let required_sum = matches
        .value_of("required_sum")
        .and_then(|n| n.parse::<usize>().ok())
        .ok_or_else(|| anyhow!("Required sum parameter is not a positive integer"))?;
    let num_parts = matches
        .value_of("num_parts")
        .and_then(|n| n.parse::<usize>().ok())
        .ok_or_else(|| anyhow!("Num parts parameter is not a positive integer"))?;

    let numbers_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");

    let numbers = parse_input(&numbers_str)?;

    let parts = find_required_sum(&numbers, required_sum, num_parts)
        .ok_or_else(|| anyhow!("Couldn't find {} values that sum to the required sum", num_parts))?;

    println!("{} = {}", parts.iter().join(" + "), required_sum);

    Ok(())
}

fn find_required_sum(numbers: &[usize], req_sum: usize, num_parts: usize) -> Option<Vec<usize>> {
    for parts in numbers.iter().combinations(num_parts) {
        let parts = parts.into_iter().copied().collect_vec();

        if parts.iter().sum::<usize>() == req_sum {
            return Some(parts);
        }
    }

    None
}

fn parse_input(numbers_str: &str) -> Result<Vec<usize>, num::ParseIntError> {
    numbers_str
        .lines()
        .map(|num_str| num_str.parse())
        .try_collect()
}
