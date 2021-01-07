use anyhow::anyhow;
use clap::{App, Arg};
use itertools::Itertools;
use maplit::{hashmap, hashset};
use std::{
    collections::{HashMap, HashSet},
    fmt, fs,
};

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2019-14")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .arg(Arg::from_usage("[raw_resource] -r --raw-resource 'Name of the initial raw resource to find the amount of'").takes_value(true).default_value("ORE"))
        .arg(Arg::from_usage("[goal] -g --goal 'Name of the goal chemical to reach'").takes_value(true).default_value("FUEL"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let reactions_str = fs::read_to_string(&input_filename)?.replace("\r\n", "\n");

    let possible_reactions = parse_input(&reactions_str)?;
    let raw_resource = matches
        .value_of("raw_resource")
        .map(|s| s.to_owned())
        .unwrap();
    let goal = matches.value_of("goal").map(|s| s.to_owned()).unwrap();

    let requirements = find_requirements_alt(
        &possible_reactions,
        &hashset! {raw_resource.clone()},
        goal.clone(),
        1,
    )
    .ok_or_else(|| anyhow!("Couldn't find a way to obtain the target chemical."))?;

    println!(
        "You need {} {} to produce 1 {}.",
        requirements[&raw_resource], raw_resource, goal
    );

    Ok(())
}

fn find_requirements_alt(
    possible_reactions: &HashMap<Chemical, Reaction>,
    bases: &HashSet<Chemical>,
    goal_chemical: Chemical,
    goal_amount: usize,
) -> Option<HashMap<Chemical, usize>> {
    let mut bucket = hashmap! {
        goal_chemical => goal_amount
    };

    while !bucket.iter().all(|(chemical, _)| bases.contains(chemical)) {
        let mut to_add = HashMap::with_capacity(bucket.len());
        let mut to_remove = Vec::with_capacity(bucket.len());

        for (chemical, &amount) in &bucket {
            // Check if we need this chemical to produce anything else in the bucket
            let mut chemical_needed_later = false;

            for other_chemical in bucket.keys() {
                if other_chemical == chemical {
                    continue;
                }

                if let Some(other_chemical_reaction) = possible_reactions.get(other_chemical) {
                    if other_chemical_reaction
                        .inputs
                        .iter()
                        .any(|(input, _)| input == chemical)
                    {
                        chemical_needed_later = true;
                        break;
                    }
                } else if bases.contains(other_chemical) {
                    continue;
                } else {
                    // There's a chemical here that we have no way of producing.
                    return None;
                }
            }

            if chemical_needed_later {
                continue;
            }

            let chemical_reaction = possible_reactions.get(chemical)?;

            for (input_chemical, &input_amount) in chemical_reaction.inputs.iter() {
                *to_add.entry(input_chemical.clone()).or_insert(0) += input_amount
                    * (amount as f64 / chemical_reaction.output_amount as f64).ceil() as usize;
            }

            to_remove.push(chemical.clone());
        }

        if to_remove.is_empty() {
            // We're stuck in a loop, there's nothing we can remove from the bucket.
            return None;
        }

        for (chemical, amount) in to_add {
            *bucket.entry(chemical).or_insert(0) += amount;
        }

        for chemical in to_remove {
            bucket.remove(&chemical);
        }
    }

    Some(bucket)
}

// TODO: This does not work accurately because the bucket is created
// while the input chemicals are being iterated through. So depending
// on the order in which that happens (which is arbitrary, because
// goal_reaction.inputs is a HashSet), we may perform the reactions in
// an ineffecient order. This can sometimes be "solved" by re-running
// the program in hopes to get a different iteration order, but
// that obviously isn't brilliant either.
fn find_requirements(
    possible_reactions: &HashMap<Chemical, Reaction>,
    bases: &HashSet<Chemical>,
    goal_chemical: Chemical,
    goal_amount: usize,
    mut bucket: HashMap<Chemical, usize>,
) -> Option<(usize, HashMap<Chemical, usize>, HashMap<Chemical, usize>)> {
    let mut requirements = HashMap::with_capacity(bases.len());

    let goal_reaction = possible_reactions.get(&goal_chemical)?;

    for (input_chemical, &input_amount) in goal_reaction.inputs.iter() {
        let amount_in_bucket = bucket.get(input_chemical).copied().unwrap_or(0);

        if amount_in_bucket > input_amount {
            bucket.get_mut(input_chemical).map(|amount_in_bucket_mut| {
                *amount_in_bucket_mut -= input_amount;
            });

            continue;
        } else {
            bucket.remove(input_chemical);
        }

        // This can't overflow because we checked earlier if the RHS >= LHS.
        let input_required_amount = input_amount - amount_in_bucket;

        if bases.contains(input_chemical) {
            *requirements.entry(input_chemical.clone()).or_insert(0) += input_required_amount;
        } else {
            let (input_produced_amount, input_requirements, input_leftovers) = find_requirements(
                possible_reactions,
                bases,
                input_chemical.to_owned(),
                input_required_amount,
                bucket,
            )?;

            for (base, base_amount) in input_requirements {
                *requirements.entry(base).or_insert(0) += base_amount;
            }

            bucket = input_leftovers;

            if input_produced_amount > input_required_amount {
                *bucket.entry(input_chemical.clone()).or_insert(0) +=
                    input_produced_amount - input_required_amount;
            }
        }
    }

    let mut produced_amount = goal_reaction.output_amount;

    if goal_reaction.output_amount < goal_amount {
        let (rest_produced_amount, rest_requirements, rest_leftovers) = find_requirements(
            possible_reactions,
            bases,
            goal_chemical,
            goal_amount - goal_reaction.output_amount,
            bucket,
        )?;

        for (base, base_amount) in rest_requirements {
            *requirements.entry(base).or_insert(0) += base_amount;
        }

        produced_amount += rest_produced_amount;
        bucket = rest_leftovers
    }

    Some((produced_amount, requirements, bucket))
}

fn parse_input(reactions_str: &str) -> Result<HashMap<Chemical, Reaction>, anyhow::Error> {
    reactions_str
        .lines()
        .map(|reaction_str| {
            let (inputs_str, output_str) = reaction_str
                .split("=>")
                .map(|s| s.trim())
                .collect_tuple()
                .ok_or_else(|| anyhow!("Invalid reaction string: Couldn't find separator"))?;

            let (output_chemical, output_amount) = parse_chemical_amount(&output_str)?;
            let inputs = inputs_str
                .split(',')
                .map(|input_chemical_amount| parse_chemical_amount(input_chemical_amount.trim()))
                .try_collect()?;

            Ok((
                output_chemical,
                Reaction {
                    inputs,
                    output_amount,
                },
            ))
        })
        .try_collect()
}

fn parse_chemical_amount(chemical_amount_str: &str) -> Result<(Chemical, usize), anyhow::Error> {
    let (amount_str, chemical) = chemical_amount_str
        .split_whitespace()
        .collect_tuple()
        .ok_or_else(|| anyhow!("Couldn't find a chemical amount in {}", chemical_amount_str))?;

    Ok((chemical.to_owned(), amount_str.parse()?))
}

type Chemical = String;

#[derive(Clone)]
struct Reaction {
    inputs: HashMap<Chemical, usize>,
    output_amount: usize,
}

impl fmt::Debug for Reaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{} => {}]",
            self.inputs
                .iter()
                .map(|(chemical, amount)| format!("{} {}", amount, chemical))
                .join(", "),
            self.output_amount
        )
    }
}
