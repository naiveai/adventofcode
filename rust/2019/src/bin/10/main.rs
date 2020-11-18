use anyhow::{anyhow, bail};
use clap::{App, Arg};
use derive_more::From;
use multimap::MultiMap;
use ordered_float::OrderedFloat;
use std::{collections::HashSet, fmt, fs};

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2019-10")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let asteroid_map_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let asteroid_positions = parse_input(&asteroid_map_str)?;

    let asteroid_visibility = find_asteroid_visibilities(&asteroid_positions);

    let (best_asteroid, best_asteroid_visibility) = asteroid_visibility
        .iter_all()
        .max_by_key(|(_, visibile)| visibile.len())
        .ok_or_else(|| anyhow!("Couldn't find best asteroid - input empty"))?;

    println!(
        "Best place to position a new station is: {:?}, where {} asteroids are visibile",
        best_asteroid,
        best_asteroid_visibility.len()
    );

    Ok(())
}

fn find_asteroid_visibilities(asteroid_positions: &HashSet<Point>) -> MultiMap<Point, Point> {
    let mut asteroid_visibility = MultiMap::with_capacity(asteroid_positions.len());

    for &potential_station in asteroid_positions {
        let slopes =
            all_slopes_with(&asteroid_positions, potential_station);

        for (slope, visibility_line) in slopes {
            let (before_point, after_point) =
                visibility_line.into_iter().partition(|p| {
                    if slope != 0. {
                        p.y < potential_station.y
                    } else {
                        // The line is straight & horizontal,
                        // in which case all y's are the same.
                        p.x < potential_station.x
                    }
                });

            let min_distance = |v: Vec<&Point>| {
                v.into_iter()
                    .min_by_key(|&p| OrderedFloat(Point::distance(&potential_station, p)))
                    .cloned()
            };

            asteroid_visibility.insert_many(
                potential_station,
                [min_distance(before_point), min_distance(after_point)]
                    // Remove either if it's None, meaning one side is empty.
                    .iter()
                    .filter_map(|&min_point| min_point),
            );
        }
    }

    asteroid_visibility
}

fn all_slopes_with(asteroid_positions: &HashSet<Point>, station: Point) -> MultiMap<OrderedFloat<f64>, &Point> {
    asteroid_positions
        .iter()
        .filter(|&a| a != &station)
        .map(|other_asteroid| {
            (
                OrderedFloat(Point::slope(&station, other_asteroid)),
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
                    asteroid_positions.insert(Point::from((column_idx as isize, -(row_idx as isize))));
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
