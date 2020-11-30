#![feature(default_free_fn)]

use anyhow::{anyhow, bail};
use clap::{App, Arg};
use derive_more::{Add, AddAssign, From, SubAssign};
use itertools::Itertools;
use std::{cmp::Ordering, default::default, fmt, fs};

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2019-5")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .arg(
            Arg::from_usage("[num_steps] -n --num-steps 'Number of steps to simulate for'")
                .default_value("1000"),
        )
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let positions_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let positions = parse_input(&positions_str)?;

    let mut planets = positions
        .into_iter()
        .map(|pos| (pos, default()))
        .collect_vec();

    let num_steps = matches
        .value_of("num_steps")
        .and_then(|n_str| n_str.parse::<usize>().ok())
        .ok_or_else(|| anyhow!("Number of steps provided couldn't be parsed as a positive number"))?;

    for _ in 1..=num_steps {
        planets = simulate_step(planets);
    }

    let total_energy = planets
        .iter()
        .map(|(pos, vel)| {
            ((pos.x.abs() + pos.y.abs() + pos.z.abs()) * (vel.x.abs() + vel.y.abs() + vel.z.abs()))
                as usize
        })
        .sum::<usize>();

    println!("Total energy after {} steps: {}", num_steps, total_energy);

    Ok(())
}

type Planet = (Coords3D, Coords3D);

fn simulate_step(mut planets: Vec<Planet>) -> Vec<Planet> {
    let mut velocity_deltas= vec![default(); planets.len()];

    for ((a_idx, (a_pos, _)), (b_idx, (b_pos, _))) in
        planets.iter().enumerate().tuple_combinations()
    {
        let vel_delta = Coords3D::from(
            vec![a_pos.x, a_pos.y, a_pos.z]
                .into_iter()
                .zip(vec![b_pos.x, b_pos.y, b_pos.z])
                .map(|(a_coord, b_coord)| match a_coord.cmp(&b_coord) {
                    // Yes, this is the right way around. Planets with
                    // lower coordinates are pulled *towards* planets
                    // with higher coordinates.
                    Ordering::Less => 1,
                    Ordering::Greater => -1,
                    Ordering::Equal => 0,
                })
                .collect_tuple::<(_, _, _)>()
                .unwrap(),
        );

        velocity_deltas[a_idx] += vel_delta;
        velocity_deltas[b_idx] -= vel_delta;
    }

    for ((planet_pos, planet_vel), vel_delta) in planets.iter_mut().zip(velocity_deltas) {
        *planet_vel += vel_delta;
        *planet_pos += *planet_vel;
    }

    planets
}

fn parse_input(positions_str: &str) -> Result<Vec<Coords3D>, anyhow::Error> {
    positions_str
        .lines()
        .map(|coords_str| {
            let coords: Vec<_> = coords_str
                .trim()
                .trim_matches(&['<', '>'] as &[_])
                .split(',')
                .map(|coord_str| coord_str.trim()[2..].parse::<isize>())
                .try_collect()?;

            Ok(Coords3D::from(match &coords[..] {
                &[x, y, z] => (x, y, z),
                _ => bail!("Non-3d coordinate found"),
            }))
        })
        .try_collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, From, Add, AddAssign, SubAssign)]
struct Coords3D {
    x: isize,
    y: isize,
    z: isize,
}

impl fmt::Debug for Coords3D {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("")
            .field(&self.x)
            .field(&self.y)
            .field(&self.z)
            .finish()
    }
}
