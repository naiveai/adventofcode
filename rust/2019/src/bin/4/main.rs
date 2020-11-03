use anyhow::anyhow;
use clap::{App, Arg};
use digits_iterator::*;
use itertools::Itertools;
use std::fs;

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2019-4")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();
    let password_range_str = fs::read_to_string(input_filename)?;

    let (password_min, password_max) = parse_input(&password_range_str)?;

    println!(
        "Number of valid passwords: {}",
        (password_min..=password_max)
            .filter(|&num| is_valid_password(num, true))
            .count()
    );

    println!(
        "Number of valid passwords if >2 matching digits is considered invalid: {}",
        (password_min..=password_max)
            .filter(|&num| is_valid_password(num, false))
            .count()
    );

    Ok(())
}

fn is_valid_password(num: usize, multiple_matching_digits_valid: bool) -> bool {
    let mut all_increasing = true;
    let mut any_repeated = false;
    let mut repeated_len = 1;

    // 1234 -> [(1, 2), (2, 3), (3, 4)]
    for (d1, d2) in num.digits().tuple_windows() {
        if d1 > d2 {
            all_increasing = false;
            break;
        }

        if d1 == d2 && !any_repeated {
            if multiple_matching_digits_valid {
                any_repeated = true;
                continue;
            }

            repeated_len += 1;
        } else if repeated_len == 2 {
            any_repeated = true;
        } else {
            repeated_len = 1;
        }
    }

    // The very last pair of digits could've formed the
    // required repetition, so we need to check repeated_len
    // again here in case any_repeated couldn't be updated.
    all_increasing && (any_repeated || repeated_len == 2)
}

fn parse_input(password_range_str: &str) -> Result<(usize, usize), anyhow::Error> {
    let (min, max) = password_range_str
        .split("-")
        .map(|s| s.trim())
        .collect_tuple()
        .ok_or(anyhow!("Could not find 2 values in the input"))?;

    Ok((min.parse()?, max.parse()?))
}
