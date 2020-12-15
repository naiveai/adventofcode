#![feature(default_free_fn)]

use anyhow::{bail, Context};
use clap::{App, Arg};
use derive_more::{Add, AddAssign, From, SubAssign};
use itertools::Itertools;
use std::{cmp::Ordering, default::default, fmt, fs};

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2019-12")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .arg(
            Arg::from_usage("[required_steps] -n --num-steps 'Number of steps to simulate for'")
                .default_value("1000"),
        )
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let positions_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let positions = parse_input(&positions_str)?;

    let input_planets = positions
        .into_iter()
        .map(|pos| (pos, default()))
        .collect_vec();

    let required_steps = matches
        .value_of("required_steps")
        .and_then(|n_str| n_str.parse::<usize>().ok())
        .context("Number of steps provided couldn't be parsed as a positive number")?;

    let mut planets = input_planets.clone();
    let mut num_steps = 0_usize;
    let (mut x_loop, mut y_loop, mut z_loop) = (None, None, None);

    loop {
        num_steps += 1;
        planets = simulate_step(planets);

        if num_steps == required_steps {
            let total_energy = planets
                .iter()
                .map(|(pos, vel)| {
                    ((pos.x.abs() + pos.y.abs() + pos.z.abs())
                        * (vel.x.abs() + vel.y.abs() + vel.z.abs())) as usize
                })
                .sum::<usize>();

            println!(
                "Total energy after {} steps: {}",
                required_steps, total_energy
            );
        }

        let mut zipped_iter = input_planets.iter().zip(planets.iter());

        // The three coordinates don't affect each other, so we find the points
        // at which each of them individually loops around and then find their LCM.

        if x_loop.is_none()
            && zipped_iter
                .clone()
                .all(|((ipos, ivel), (pos, vel))| ipos.x == pos.x && ivel.x == vel.x)
        {
            x_loop = Some(num_steps);
        }

        if y_loop.is_none()
            && zipped_iter
                .clone()
                .all(|((ipos, ivel), (pos, vel))| ipos.y == pos.y && ivel.y == vel.y)
        {
            y_loop = Some(num_steps);
        }

        if z_loop.is_none()
            && zipped_iter.all(|((ipos, ivel), (pos, vel))| ipos.z == pos.z && ivel.z == vel.z)
        {
            z_loop = Some(num_steps);
        }

        if x_loop.is_some() && y_loop.is_some() && z_loop.is_some() {
            break;
        }
    }

    let (x_loop, y_loop, z_loop) = (x_loop.unwrap(), y_loop.unwrap(), z_loop.unwrap());

    let lcm =
        x_loop * y_loop * z_loop / gcd(y_loop * z_loop, gcd(z_loop * x_loop, x_loop * y_loop));

    println!("Number of steps until the universe loops around: {}", lcm);

    Ok(())
}

// See https://en.wikipedia.org/wiki/Greatest_common_divisor#Euclid%27s_algorithm
fn gcd(a: usize, b: usize) -> usize {
    if a == 0 {
        b
    } else if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

type Planet = (Coords3D, Coords3D);

fn simulate_step(mut planets: Vec<Planet>) -> Vec<Planet> {
    let mut velocity_deltas = vec![default(); planets.len()];

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
