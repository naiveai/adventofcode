use anyhow::anyhow;
use clap::{App, Arg};
use itertools::Itertools;
use std::fs;

pub fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2015-1")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let module_masses_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let module_masses = parse_input(&module_masses_str)?;

    println!(
        "Total fuel requirements based purely on module mass: {}",
        module_masses
            .iter()
            .map(|&m| calculate_fuel(m))
            .sum::<usize>()
    );

    println!(
        "Total fuel requirements with fuel mass: {}",
        module_masses
            .iter()
            .map(|&m| calculate_all_fuel(m))
            .sum::<usize>()
    );

    Ok(())
}

fn calculate_all_fuel(mass: usize) -> usize {
    match calculate_fuel(mass) {
        0 => 0,
        x => x + calculate_all_fuel(x),
    }
}

fn calculate_fuel(mass: usize) -> usize {
    (mass / 3).saturating_sub(2)
}

fn parse_input(module_masses_str: &str) -> Result<Vec<usize>, anyhow::Error> {
    module_masses_str
        .lines()
        .map(|mass_str| {
            mass_str
                .parse()
                .map_err(|_| anyhow!("Could not parse module mass as usize"))
        })
        .try_collect()
}
