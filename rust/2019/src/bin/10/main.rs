#![feature(iter_partition_in_place, box_syntax)]

use anyhow::{anyhow, bail};
use clap::{App, Arg};
use derive_more::From;
use itertools::Itertools;
use multimap::MultiMap;
use ordered_float::OrderedFloat;
use std::{cmp::Reverse, collections::HashSet, fmt, fs, iter};

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2019-10")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let asteroid_map_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let asteroid_positions = parse_input(&asteroid_map_str)?;

    let (best_asteroid, best_asteroid_visibility) = asteroid_positions
        .iter()
        .map(|&potential_station| {
            (
                potential_station,
                iter_visible_from(potential_station, asteroid_positions.clone()).count(),
            )
        })
        .max_by_key(|&(_, visible)| visible)
        .ok_or_else(|| anyhow!("Couldn't find best asteroid - input empty"))?;

    println!(
        "Best place to position a new station is: {:?}, where {} asteroids are visibile",
        best_asteroid, best_asteroid_visibility,
    );

    println!(
        "200th asteroid to be vaporized is {:?}",
        iter_vaporize_from(best_asteroid, asteroid_positions)
            .nth(199)
            .ok_or_else(|| anyhow!("Less than 200 asteroids are vaporized"))?
    );

    Ok(())
}

fn iter_vaporize_from(
    station: Point,
    mut asteroid_positions: HashSet<Point>,
) -> impl Iterator<Item = Point> {
    let mut current_visible_iter: Option<Box<dyn Iterator<Item = Point>>> = None;

    iter::from_fn(move || {
        if let Some(next_vaporized) = current_visible_iter.as_mut().and_then(|i| i.next()) {
            asteroid_positions.remove(&next_vaporized);

            Some(next_vaporized)
        } else {
            current_visible_iter = Some(box iter_visible_from(station, asteroid_positions.clone()));

            current_visible_iter.as_mut().and_then(|i| i.next())
        }
    })
}

fn iter_visible_from(
    station: Point,
    asteroid_positions: HashSet<Point>,
) -> impl Iterator<Item = Point> {
    let mut relative_slopes = all_slopes_relative(station, asteroid_positions)
        .into_iter()
        .collect_vec();

    relative_slopes.sort_unstable_by_key(|&(slope, _)| Reverse(slope));

    IterVisible {
        center: station,
        pos: 0,
        on_right_side: true,
        ordered_relative_slopes: relative_slopes
            .into_iter()
            .map(|(slope, points)| (slope.into_inner(), points))
            .collect(),
    }
}

// We're rotating an imaginary line around the center of a Cartesian plane.
// The line rotates clockwise, so it goes from quadrant 1 to Q4 to Q3 to Q2.
// When we access the points on a given line from ordered_relative_slopes,
// we access the ones on both sides of the center (so in two different quadrants),
// so we need to keep track of which direction we're looking at.
struct IterVisible {
    center: Point,
    pos: usize,
    on_right_side: bool,
    ordered_relative_slopes: Vec<(f64, Vec<Point>)>,
}

impl Iterator for IterVisible {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        // The slope we're on might not have any points on its line,
        // at least not in the direction we're currently looking in, but that
        // doesn't mean we can terminate iteration. We have to keep checking
        // until we find the next visible point.
        loop {
            if self.pos >= self.ordered_relative_slopes.len() {
                if !self.on_right_side {
                    return None;
                }

                self.pos = 0;
                self.on_right_side = false;
            }

            let (slope, visibility_line) = &self.ordered_relative_slopes[self.pos];

            let (before_points, after_points) = visibility_line.into_iter().partition(|p| {
                if *slope != 0. {
                    p.y < self.center.y
                } else {
                    // The line is straight and horizontal,
                    // in which case all y's are the same.
                    p.x < self.center.x
                }
            });

            // For us to use the after points, we must either be in positive
            // slopes on the right side or negative slopes on the left side.
            let front_points: Vec<_> = if (*slope >= 0.) == self.on_right_side {
                after_points
            } else {
                before_points
            };

            let min_front_point = front_points
                .into_iter()
                .min_by_key(|&p| OrderedFloat(Point::distance(&self.center, p)))
                .copied();

            self.pos += 1;

            if min_front_point.is_some() {
                return min_front_point;
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.ordered_relative_slopes.len(), None)
    }
}

fn all_slopes_relative(
    station: Point,
    asteroid_positions: HashSet<Point>,
) -> MultiMap<OrderedFloat<f64>, Point> {
    asteroid_positions
        .iter()
        .filter(|&a| a != &station)
        .map(|&other_asteroid| {
            (
                OrderedFloat(Point::slope(&station, &other_asteroid)),
                other_asteroid,
            )
        })
        .collect()
}

fn parse_input(asteroid_map_str: &str) -> Result<HashSet<Point>, anyhow::Error> {
    let mut asteroid_positions = HashSet::new();

    for (row_idx, row) in asteroid_map_str.lines().enumerate() {
        for (column_idx, pos_char) in row.chars().enumerate() {
            match pos_char {
                '.' => continue,
                '#' => {
                    // The points are all represented as being in Q4 (positive X, negative Y),
                    // so that all the slope and distance calculations work out properly.
                    // If we used positive numbers for both of them, we'd end up with
                    // opposite-signed slopes for some points.
                    asteroid_positions.insert(Point::new(column_idx as isize, -(row_idx as isize)));
                }
                _ => bail!("Unknown character: {}", pos_char),
            }
        }
    }

    Ok(asteroid_positions)
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
    fn new(x: isize, y: isize) -> Self {
        Self::from((x, y))
    }

    fn slope(p1: &Self, p2: &Self) -> f64 {
        // Cast to isize to avoid overflows
        let slope = (p2.y - p1.y) as f64 / (p2.x - p1.x) as f64;

        if slope.is_infinite() {
            // We've done (y2 - y) / 0., which means the two points
            // are on a vertical line, in which case the sign
            // of the infinity doesn't matter.
            slope.abs()
        } else if slope.is_nan() {
            // We've done 0. / 0., which means the two points
            // are exactly the same.
            0.
        } else {
            slope
        }
    }

    fn distance(p1: &Self, p2: &Self) -> f64 {
        // sqrt returns NaN only if the original number is
        // negative, which isn't possible in this case.
        (((p2.x - p1.x).pow(2) + (p2.y - p1.y).pow(2)) as f64).sqrt()
    }
}
