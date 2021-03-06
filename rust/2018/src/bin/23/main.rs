use anyhow::{anyhow, Context};
use clap::{App, Arg};
use itertools::Itertools;
use std::{fmt, fs, num::ParseIntError, str::FromStr};

pub fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2018-23")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let bot_info_str = fs::read_to_string(input_filename)?;
    let bots = parse_input(&bot_info_str)?;

    let best_point = find_best_point_z3(bots).ok_or_else(|| anyhow!("No best point found"))?;

    println!(
        "Best teleporation point: {:?}. Manhattan distance to origin: {}",
        best_point,
        (best_point.x + best_point.y + best_point.z)
    );

    Ok(())
}

// This is basically cheating because it's stolen from /u/mserrano on the
// /r/AdventOfCode solutions thread for this problem, and even if it wasn't
// stolen it's a really unsatisfying solution because it basically just
// assembles a problem description and asks another, far more advanced,
// third-party dependency to just magically solve it. But I had no idea how to
// solve it and this is really slow anyway.
fn find_best_point_z3(bots: Vec<Bot>) -> Option<Location> {
    use z3::{ast::*, *};

    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let opt = Optimize::new(&ctx);

    let (x, y, z) = (
        Int::new_const(&ctx, "x"),
        Int::new_const(&ctx, "y"),
        Int::new_const(&ctx, "z"),
    );

    fn zabs<'a>(ctx: &'a Context, v: &'a Int) -> Int<'a> {
        v.ge(&Int::from_i64(ctx, 0)).ite(v, &v.unary_minus())
    }

    let in_range_flags = (0..bots.len())
        .map(|i| Int::new_const(&ctx, format!("in_range_{}", i)))
        .collect_vec();

    for (i, bot) in bots.iter().enumerate() {
        let (bot_x, bot_y, bot_z, bot_radius) = (
            Int::from_i64(&ctx, bot.location.x as i64),
            Int::from_i64(&ctx, bot.location.y as i64),
            Int::from_i64(&ctx, bot.location.z as i64),
            Int::from_u64(&ctx, bot.signal_radius as u64),
        );

        // If (x, y, z) is in range of the current bot, it'll be 1, otherwise 0
        opt.assert(
            &in_range_flags[i]._eq(
                &Int::add(
                    &ctx,
                    &[
                        &zabs(&ctx, &Int::sub(&ctx, &[&x, &bot_x])),
                        &zabs(&ctx, &Int::sub(&ctx, &[&y, &bot_y])),
                        &zabs(&ctx, &Int::sub(&ctx, &[&z, &bot_z])),
                    ],
                )
                .le(&bot_radius)
                .ite(&Int::from_u64(&ctx, 1), &ast::Int::from_u64(&ctx, 0)),
            ),
        );
    }

    // Maximize the number of bots in range
    opt.maximize(&Int::add(
        &ctx,
        // Convert Vec<T> to Vec<&T>
        &in_range_flags.iter().collect_vec(),
    ));

    // Minimize the manhattan distance from the origin
    opt.minimize(&Int::add(
        &ctx,
        &[&zabs(&ctx, &x), &zabs(&ctx, &y), &zabs(&ctx, &z)],
    ));

    if opt.check(&[]) != SatResult::Sat {
        return None;
    }

    let model = opt.get_model()?;

    let (res_x, res_y, res_z) = (
        model.eval(&x).unwrap().as_i64().unwrap() as isize,
        model.eval(&y).unwrap().as_i64().unwrap() as isize,
        model.eval(&z).unwrap().as_i64().unwrap() as isize,
    );

    Some(Location {
        x: res_x,
        y: res_y,
        z: res_z,
    })
}

fn parse_input(bot_info_str: &str) -> Result<Vec<Bot>, anyhow::Error> {
    let mut bots = vec![];

    for bot_info_line in bot_info_str.lines() {
        let (position_str, radius_str) = bot_info_line
            .split(", ")
            .collect_tuple()
            .ok_or_else(|| anyhow!("Invalid bot info line format"))?;

        bots.push(Bot {
            location: position_str
                .strip_prefix("pos=")
                .ok_or_else(|| anyhow!("Invalid position format"))?
                .trim_matches(|c| c == '<' || c == '>')
                .parse()?,
            signal_radius: radius_str
                .strip_prefix("r=")
                .ok_or_else(|| anyhow!("Invalid radius string format"))?
                .parse()
                .context("Radius string is not a number")?,
        });
    }

    Ok(bots)
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
struct Bot {
    location: Location,
    signal_radius: usize,
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
struct Location {
    x: isize,
    y: isize,
    z: isize,
}

impl FromStr for Location {
    type Err = ParseLocationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ParseLocationError::*;

        let (x_str, y_str, z_str) = s.split(',').collect_tuple().ok_or(CommaFormatError)?;

        Ok(Self {
            x: x_str.trim().parse().map_err(|e| ParseCoordinateError {
                coord: 'x',
                source: e,
            })?,
            y: y_str.trim().parse().map_err(|e| ParseCoordinateError {
                coord: 'y',
                source: e,
            })?,
            z: z_str.trim().parse().map_err(|e| ParseCoordinateError {
                coord: 'z',
                source: e,
            })?,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseLocationError {
    #[error("String isn't formatted like 'x,y,z'")]
    CommaFormatError,
    #[error("The coordinate {} can't be parsed into an isize", coord)]
    ParseCoordinateError { coord: char, source: ParseIntError },
}

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("")
            .field(&self.x)
            .field(&self.y)
            .field(&self.z)
            .finish()
    }
}
