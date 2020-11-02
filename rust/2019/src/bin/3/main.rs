#![feature(box_syntax, iterator_fold_self)]

use anyhow::bail;
use clap::{App, Arg};
use derive_more::From;
use indexmap::IndexSet;
use itertools::Itertools;
use std::{fmt, fs, iter, str::FromStr};

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2019-3")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();
    let all_wire_sections_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");

    let all_wire_sections: Vec<_> = all_wire_sections_str
        .lines()
        .map(parse_wire_sections)
        .try_collect()?;

    let all_wire_points: Vec<Vec<Point>> = all_wire_sections
        .into_iter()
        .map(expand_to_wire_points)
        .try_collect()?;

    let intersection_points = all_wire_points
        .iter()
        .map(|v| v.iter().copied().collect())
        .fold_first(|s1, s2| &s1 & &s2)
        .unwrap_or_else(IndexSet::new);

    if intersection_points.len() == 0 {
        bail!("No intersection points found.")
    }

    if let Some(closest_point) = intersection_points
        .iter()
        .min_by_key(|p| p.manhattan_distance(&Point::origin()))
    {
        println!(
            "Closest intersection point to central port: {:?}",
            closest_point
        );
    }

    if let Some((idx, min_steps, min_total_steps)) = intersection_points
        .iter()
        .enumerate()
        .map(|(idx, int_point)| {
            let all_steps = all_wire_points
                .iter()
                .map(|wp| wp.iter().position(|p| p == int_point).unwrap() + 1)
                .collect_vec();

            let total_steps = all_steps.iter().sum::<usize>();

            (idx, all_steps, total_steps)
        })
        .min_by_key(|&(_, _, total_steps)| total_steps)
    {
        println!(
            "Point {:?} is {} = {} steps from the wire starts",
            intersection_points[idx],
            min_steps
                .iter()
                .map(|n| n.to_string())
                .collect_vec()
                .join(" + "),
            min_total_steps
        );
    }

    Ok(())
}

fn expand_to_wire_points(
    wire_sections: Vec<(Direction, usize)>,
) -> Result<Vec<Point>, anyhow::Error> {
    let mut wire = Vec::with_capacity(wire_sections.iter().map(|(_, amount)| amount).sum());
    let mut wire_head = Point::origin();

    for (direction, amount) in wire_sections {
        let amount = amount as isize;

        let Point { x, y } = wire_head;

        use Direction::*;

        let section_end = Point::from(match direction {
            Up => (x + amount, y),
            Right => (x, y + amount),
            Down => (x - amount, y),
            Left => (x, y - amount),
        });

        // We have to use a Box with dyn because the Iterator
        // concrete types are technically different.
        let coordinate_range: Box<dyn Iterator<Item = isize>> = match direction {
            Up => box (x + 1..=section_end.x),
            Right => box (y + 1..=section_end.y),
            Down => box (section_end.x..=x - 1).rev(),
            Left => box (section_end.y..=y - 1).rev(),
        };

        let section_points: Box<dyn Iterator<Item = (isize, isize)>> = match direction {
            Up | Down => box coordinate_range.zip(iter::repeat(y)),
            Right | Left => box iter::repeat(x).zip(coordinate_range),
        };

        wire_head = section_end;

        wire.extend(section_points.map(Point::from))
    }

    Ok(wire)
}

fn parse_wire_sections(wire_sections_str: &str) -> Result<Vec<(Direction, usize)>, anyhow::Error> {
    wire_sections_str
        .split(",")
        .map(|ins| ins.split_at(1))
        .map(|(direction, amount_str)| Ok((direction.parse()?, amount_str.parse()?)))
        .try_collect()
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl FromStr for Direction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "U" => Self::Up,
            "D" => Self::Down,
            "L" => Self::Left,
            "R" => Self::Right,
            _ => bail!("Unknown direction: {}", s),
        })
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, From)]
struct Point {
    x: isize,
    y: isize,
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("P").field(&self.x).field(&self.y).finish()
    }
}

impl Point {
    fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    fn origin() -> Self {
        Self::new(0, 0)
    }

    fn manhattan_distance(&self, other: &Self) -> usize {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as usize
    }
}
