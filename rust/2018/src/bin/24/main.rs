use anyhow::anyhow;
use clap::{App, Arg, ArgGroup};
use itertools::Itertools;
use regex::Regex;
use std::cmp::Reverse;
use std::collections::HashSet;
use std::fmt;
use std::fs;

pub fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2018-24")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .arg(Arg::from_usage("[p1] -1 --part1 'Solves Part 1'"))
        .arg(Arg::from_usage("[p2] -2 --part2 'Solves Part 2'").overrides_with("p1"))
        .arg(
            Arg::from_usage("[boosted] --boosted 'Teams that should be boosted in Part 2'")
                .requires("p2")
                .takes_value(true)
                .multiple(true)
                .default_value_if("p2", None, "Immune System"),
        )
        .group(
            ArgGroup::with_name("part")
                .args(&["p1", "p2"])
                .required(true),
        )
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let battle_info_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let groups = parse_input(&battle_info_str)?;

    if matches.is_present("p1") {
        println!("Battle without boosts:");

        let no_boost_result = battle_to_end(groups, None, None)
            .ok_or_else(|| anyhow!("Input results in a stalemate"))?;
        battle_victor_info(&no_boost_result);
    } else if let Some(boosted_armies_iter) = matches.values_of("boosted") {
        let boosted_armies = boosted_armies_iter.collect_vec();

        for boost in 1..=usize::MAX {
            if let Some(boosted_result) =
                battle_to_end(groups.clone(), Some(&boosted_armies), Some(boost))
            {
                if boosted_armies.contains(&&*boosted_result[0].army) {
                    println!(
                        "Battle with a boost of {} to {:?}:",
                        boost,
                        boosted_armies.join(", and ")
                    );
                    battle_victor_info(&boosted_result);
                    break;
                }
            }
        }
    }

    Ok(())
}

fn battle_victor_info(groups: &[UnitGroup]) {
    println!(
        "{:?} wins with {:?} units left",
        groups[0].army,
        groups.iter().map(|g| g.num_units).sum::<usize>()
    );
}

fn battle_to_end(
    mut groups: Vec<UnitGroup>,
    boost_armies: Option<&[&str]>,
    boost_amount: Option<usize>,
) -> Option<Vec<UnitGroup>> {
    if let Some(boost_amount) = boost_amount {
        let boost_armies = boost_armies.unwrap();

        for group in groups.iter_mut() {
            if boost_armies.contains(&&*group.army) {
                group.attack_dmg += boost_amount;
            }
        }
    }

    while groups.iter().any(|g| g.army != groups[0].army) {
        let new_groups = battle_tick(groups.clone());

        if new_groups == groups {
            // Stalemate
            return None;
        }

        groups = new_groups;
    }

    Some(groups)
}

fn battle_tick(mut groups: Vec<UnitGroup>) -> Vec<UnitGroup> {
    groups.sort_unstable_by_key(|g| Reverse((g.effective_power(), g.initiative)));

    let mut attacks = Vec::new();

    fn calculate_attack_dmg(attacker: &UnitGroup, defender: &UnitGroup) -> usize {
        let mut dmg = attacker.effective_power();

        if defender.immunities.contains(&attacker.attack_dmg_type) {
            dmg = 0;
        } else if defender.weaknesses.contains(&attacker.attack_dmg_type) {
            dmg *= 2;
        }

        dmg
    }

    for (pos, group) in groups.iter().enumerate() {
        let best_enemy = groups
            .iter()
            .enumerate()
            .filter_map(|(other_pos, other)| {
                if other.army != group.army && attacks.iter().all(|(_, e_p)| *e_p != other_pos) {
                    Some((other, other_pos, calculate_attack_dmg(&group, &other)))
                } else {
                    None
                }
            })
            .max_by_key(|&(e, _, dmg)| (dmg, e.effective_power(), e.initiative))
            // This group may already be damaged by the time it gets to attack,
            // so the damage calculated in this phase may not be correct. We can
            // ignore it now.
            .map(|(_, enemy_pos, _)| enemy_pos);

        if let Some(enemy_pos) = best_enemy {
            attacks.push((pos, enemy_pos));
        }
    }

    attacks.sort_unstable_by_key(|(a_p, _)| Reverse(groups[*a_p].initiative));

    for (attacker_pos, defender_pos) in attacks {
        // We clone so we can get the &mut defender later
        let attacker = groups.get(attacker_pos).unwrap().clone();

        if attacker.num_units == 0 {
            // We can't remove it yet because we need to mantain the positions
            continue;
        }

        let defender = groups.get_mut(defender_pos).unwrap();

        let dmg = calculate_attack_dmg(&attacker, &defender);

        // This is usize divison, meaning it'll round down on its own.
        defender.num_units = defender.num_units.saturating_sub(dmg / defender.unit_hp);
    }

    groups.into_iter().filter(|g| g.num_units > 0).collect_vec()
}

fn parse_input(battle_info_str: &str) -> Result<Vec<UnitGroup>, anyhow::Error> {
    let army_lines_iter = battle_info_str
        .split("\n\n")
        .map(|army_str| army_str.lines());

    let mut groups = Vec::new();

    // This regex is desgined to match one-line group strings like:
    // "3 units each with 5 hit points (immune to cold, radiation; weak to
    //     slashing) with an attack that does 2 slashing damage at initiative 3"
    // The paranthetical immunities and weaknesses can either not exist at all,
    // only have one of the attributes, or have them in a different order:
    // (immune to cold)
    // (weak to radiation)
    // (weak to slashing; immune to cold)
    // This is what introduces most of the complexity of this regex. The rest of
    // it is pretty straightforward mostly literal matching.
    // We're compiling it here inside of inside the loop for effeciency.
    let group_re: Regex = Regex::new(
        r"(?ix)
            # Matches unit count and hp
            (?P<num_units>\d+) \s+ units* \s+ each \s+ with \s+ (?P<hp>\d+) \s+ hit \s+ points*
            # Matches the immunities and weaknesses
            \s*\(*(?:(?:immune \s+ to \s+ (?P<immunities>[^;\)]+));*\s*|(?:weak \s+ to \s+ (?P<weaknesses>[^;\)]+));*\s*)*\)*
            # Matches the damage attributes
            \s* with \s+ an \s+ attack \s+ that \s+ does \s+ (?P<dmg>\d+) \s+ (?P<dmg_type>\S+) \s+ damage \s+
            # Matches the initiative
            at \s+ initiative \s+ (?P<initiative>\d+)"
    ).unwrap(); // This would only panic if the regex itself is wrong

    for mut army_lines in army_lines_iter {
        let army_name = army_lines
            .next()
            .ok_or(anyhow!("Army is empty"))?
            .trim()
            .trim_matches(':')
            .to_string();

        for group_str in army_lines {
            let group_caps = group_re
                .captures(group_str)
                .ok_or(anyhow!("Group string not in expected format"))?;

            groups.push(UnitGroup {
                army: army_name.clone(),
                num_units: group_caps
                    .name("num_units")
                    .ok_or(anyhow!("Unit count not found in the expected format"))?
                    .as_str()
                    .parse()?,
                unit_hp: group_caps
                    .name("hp")
                    .ok_or(anyhow!("Unit hp not found in the expected format"))?
                    .as_str()
                    .parse()?,
                immunities: group_caps.name("immunities").map_or_else(
                    || HashSet::new(),
                    |imm_match| {
                        imm_match
                            .as_str()
                            .split(", ")
                            .map(|s| s.trim().to_string())
                            .collect()
                    },
                ),
                weaknesses: group_caps.name("weaknesses").map_or_else(
                    || HashSet::new(),
                    |weak_match| {
                        weak_match
                            .as_str()
                            .split(", ")
                            .map(|s| s.trim().to_string())
                            .collect()
                    },
                ),
                attack_dmg: group_caps
                    .name("dmg")
                    .ok_or(anyhow!("Attack damage not found in the expected format"))?
                    .as_str()
                    .parse()?,
                attack_dmg_type: group_caps
                    .name("dmg_type")
                    .ok_or(anyhow!(
                        "Attack damage type not found in the expected format"
                    ))?
                    .as_str()
                    .to_string(),
                initiative: group_caps
                    .name("initiative")
                    .ok_or(anyhow!("Initiative not found in the expected format"))?
                    .as_str()
                    .parse()?,
            });
        }
    }

    Ok(groups)
}

#[derive(Eq, PartialEq, Clone)]
struct UnitGroup {
    army: String,
    num_units: usize,
    unit_hp: usize,
    immunities: HashSet<String>,
    weaknesses: HashSet<String>,
    attack_dmg: usize,
    attack_dmg_type: String,
    initiative: usize,
}

impl UnitGroup {
    fn effective_power(&self) -> usize {
        self.num_units * self.attack_dmg
    }
}

impl fmt::Debug for UnitGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} {} hp (i: {:?}, w: {:?}) a {} {} {} init, ep: {}",
            self.army,
            self.num_units,
            self.unit_hp,
            self.immunities,
            self.weaknesses,
            self.attack_dmg,
            self.attack_dmg_type,
            self.initiative,
            self.effective_power()
        )
    }
}
