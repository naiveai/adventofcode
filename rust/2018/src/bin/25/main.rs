#![allow(incomplete_features)]
#![feature(const_generics, specialization, refcell_take, type_alias_impl_trait)]

mod disjoint_set;

use anyhow::anyhow;
use clap::{App, Arg};
use derive_more::{From, Index};
use disjoint_set::DisjointSet;
use itertools::Itertools;
use num::{
    traits::{AsPrimitive, NumAssignOps},
    Num, Unsigned,
};
use std::{convert::TryInto, fmt, fs, iter, slice, str::FromStr};

pub fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2018-25")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let coords_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let points = parse_input::<i8, 4>(&coords_str)?;

    let points_ds = find_chains(&points, 3u8);

    println!("The number of constellations is {}", points_ds.num_sets());

    Ok(())
}

// Most of these generic requirements are because of the
// requirements on `Point::manhattan_distance`. See there for details.
fn find_chains<N, const D: usize, C>(
    points: &Vec<Point<N, D>>,
    chain_distance: C,
) -> DisjointSet<Point<N, D>>
where
    N: Num + Eq + PartialOrd + AsPrimitive<C>,
    C: 'static + Unsigned + Copy + NumAssignOps + PartialOrd,
{
    let mut points_ds = DisjointSet::with_capacity(points.len());

    // We map the index of a point in the original list to its index in the DisjointSet.
    let mut points_set_idxs: Vec<(usize, usize)> = Vec::with_capacity(points.len());

    for (point_idx, point) in points.iter().copied().enumerate() {
        let point_set_idx = match points_ds.make_set(point) {
            Ok(i) => i,
            // This means there are duplicate points, which we can ignore.
            Err(_) => continue,
        };

        for &(other_point_idx, other_point_set_idx) in points_set_idxs.iter() {
            let other_point = &points[other_point_idx];

            if point.manhattan_distance(other_point) <= chain_distance {
                points_ds.union(point_set_idx, other_point_set_idx);
            }
        }

        points_set_idxs.push((point_idx, point_set_idx));
    }

    points_ds
}

fn parse_input<N, const D: usize>(coords_str: &str) -> Result<Vec<Point<N, D>>, anyhow::Error>
where
    N: Num + FromStr,
{
    coords_str
        .lines()
        .map(|line| {
            line.trim()
                .trim_matches(&['(', ')', '[', ']'] as &[_])
                .split(',')
                .map(|c| c.parse().map_err(|_| anyhow!("Could not parse coordinate")))
                .try_collect()
                .and_then(|coords: Vec<_>| {
                    // Coerce this Vec into a fixed-size array
                    // and error out if it doesn't work due to length
                    Ok(Point(coords.try_into().map_err(|_| {
                        anyhow!("Could not find {} coordinates in a line", D)
                    })?))
                })
        })
        .try_collect()
}

#[derive(Clone, Copy, From, Index, Eq, PartialEq, Hash)]
struct Point<N: Num, const D: usize>([N; D]);

impl<N: Num + Default, const D: usize> Default for Point<N, D> {
    fn default() -> Self {
        Self(
            match iter::repeat_with(N::default)
                .take(D)
                .collect_vec()
                .try_into()
            {
                Ok(array) => array,
                Err(_) => unsafe { std::hint::unreachable_unchecked() },
            },
        )
    }
}

impl<N: Num + fmt::Debug, const D: usize> fmt::Debug for Point<N, D> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut tuple_output = fmt.debug_tuple(&format!("P{}", D));

        for c in self.iter() {
            tuple_output.field(c);
        }

        tuple_output.finish()
    }
}

impl<N: Num, const D: usize> Point<N, D> {
    fn iter(&self) -> slice::Iter<N> {
        self.0.iter()
    }
}

// Did I make this unnecesssarily generic and therefore
// complicated? Yes. But to be fair I thought it'd
// be simpler than this and I wanted to mess around
// with generics anyway.
impl<N, const D: usize> Point<N, D>
where
    N: Num + PartialOrd,
{
    fn manhattan_distance<R>(&self, other: &Point<N, D>) -> R
    where
        R: 'static + Unsigned + Copy + NumAssignOps,
        N: AsPrimitive<R>,
    {
        let mut total = R::zero();

        for (&self_coord, &other_coord) in self.iter().zip(other.iter()) {
            // We could use num::abs here, but for some absurd
            // reason that requires N: num::Signed, which is unnecessarily
            // restrictive for our purposes.
            total += (if self_coord > other_coord {
                self_coord - other_coord
            } else {
                other_coord - self_coord
            })
            .as_();
            // We know for a fact that this has to be positive,
            // so the "as" conversion to an R: Unsigned type
            // will be fine.
        }

        total
    }
}
