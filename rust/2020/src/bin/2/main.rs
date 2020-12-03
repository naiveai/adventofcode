#![feature(pattern, try_blocks)]

use anyhow::{anyhow, bail};
use clap::{App, Arg};
use itertools::Itertools;
use std::{fmt, fs, marker::PhantomData, ops::RangeInclusive, str::pattern::Pattern};

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2020-2")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let passwords_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let policies_and_passwords = parse_input(&passwords_str)?;

    println!(
        "Number of valid passwords in the list by num occurences policy: {}",
        policies_and_passwords
            .iter()
            .filter(|(policy, password)| policy.is_valid_in_range(password))
            .count()
    );

    println!(
        "Number of valid passwords in the list by positions policy: {}",
        policies_and_passwords
            .iter()
            .filter(|(policy, password)| policy.is_valid_in_positions(password))
            .count()
    );

    Ok(())
}

fn parse_input(passwords_str: &str) -> Result<Vec<(Policy<char>, &str)>, anyhow::Error> {
    passwords_str
        .lines()
        .map(|password_line| {
            let (policy_str, password_str) = password_line
                .split(':')
                .map(|s| s.trim())
                .collect_tuple()
                .ok_or_else(|| anyhow!("Couldn't find : in password line"))?;

            let (range_str, required_pattern_str) =
                policy_str
                    .split_whitespace()
                    .collect_tuple()
                    .ok_or_else(|| anyhow!("Invalid policy format"))?;

            let required_pattern = if required_pattern_str.len() == 1 {
                required_pattern_str.chars().next().unwrap()
            } else {
                bail!("Required pattern is not a character")
            };

            let (min, max) = range_str
                .split('-')
                .map(|n| {
                    n.parse::<usize>()
                        .map_err(|_| anyhow!("Couldn't parse policy into positive integer"))
                })
                .collect_tuple()
                .ok_or_else(|| anyhow!("Invalid amount of rules in policy"))?;

            Ok((Policy::new(min?..=max?, required_pattern), password_str))
        })
        .try_collect()
}

#[derive(Clone)]
struct Policy<'a, P: Pattern<'a>> {
    range: RangeInclusive<usize>,
    required_pattern: P,
    phantom: PhantomData<&'a str>,
}

impl<'a, P: Pattern<'a>> Policy<'a, P> {
    fn new(range: RangeInclusive<usize>, required_pattern: P) -> Self {
        Self {
            range,
            required_pattern,
            phantom: PhantomData,
        }
    }
}

impl<'a, P: Pattern<'a> + Clone> Policy<'a, P> {
    fn is_valid_in_range(&self, s: &'a str) -> bool {
        self.range
            .contains(&s.matches(self.required_pattern.clone()).count())
    }
}

impl<'a, P: Pattern<'a> + PartialEq<char>> Policy<'a, P> {
    fn is_valid_in_positions(&self, s: &'a str) -> bool {
        let (a, b) = (
            s.chars().nth(self.range.start() - 1),
            s.chars().nth(self.range.end() - 1),
        );

        // Annyoying workaround because bool doesn't impl Try
        let (a, b) = (
            match a {
                Some(a) => a,
                None => return false,
            },
            match b {
                Some(b) => b,
                None => return false,
            },
        );

        (self.required_pattern == a) != (self.required_pattern == b)
    }
}

impl<'a, P: Pattern<'a> + fmt::Debug> fmt::Debug for Policy<'a, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}-{} {:?}",
            self.range.start(),
            self.range.end(),
            self.required_pattern
        )
    }
}
