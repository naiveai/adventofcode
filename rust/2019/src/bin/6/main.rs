use anyhow::anyhow;
use clap::{App, Arg};
use itertools::Itertools;
use multimap::MultiMap;
use std::{collections::HashMap, fs, hash::Hash, mem};

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2019-6")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let orbits_str = fs::read_to_string(input_filename)?.replace("\r\n", "\n");
    let orbits = parse_input(&orbits_str)?;

    let mut depths = HashMap::with_capacity(orbits.len());
    let mut euler_walk = Vec::with_capacity(orbits.len());

    depth_first_traversal(&orbits, &mut depths, &mut euler_walk, &"COM".to_owned(), 0);

    println!("Total number of orbits: {}", depths.values().sum::<usize>());

    println!(
        "Shortest path between us and Santa is {} orbital transfers long",
        find_path_length(&depths, &euler_walk, &"YOU".to_owned(), &"SAN".to_owned())
            .map(|e| e.saturating_sub(2)) // Skip the starting and destination
            .ok_or_else(|| anyhow!("Couldn't find a path between us and Santa"))?,
    );

    Ok(())
}

// GeeksForGeeks comes in clutch, unexpectedly!
// https://www.geeksforgeeks.org/lca-n-ary-tree-constant-query-o1/
fn find_path_length<T: Eq + Hash>(
    depths: &HashMap<T, usize>,
    euler_walk: &[T],
    start: &T,
    destination: &T,
) -> Option<usize> {
    let (mut start_pos, mut end_pos) = euler_walk
        .iter()
        .positions(|e| e == start || e == destination)
        .collect_tuple()?;

    if start_pos > end_pos {
        mem::swap(&mut start_pos, &mut end_pos);
    }

    let lowest_common_ancestor_depth = euler_walk[start_pos..end_pos]
        .iter()
        // Skip the starting element. If we added 1 to the start_pos
        // we could end up panicking from an invalid index.
        .skip(1)
        .map(|e| depths.get(e).unwrap())
        .min()?;

    Some((depths[start] + depths[destination]) - (lowest_common_ancestor_depth * 2))
}

fn depth_first_traversal<T: Eq + Hash + Clone>(
    elements: &MultiMap<T, T>,
    depths: &mut HashMap<T, usize>,
    euler_walk: &mut Vec<T>,
    root: &T,
    depth: usize,
) {
    euler_walk.push(root.clone());
    depths.insert(root.clone(), depth);

    if let Some(children) = elements.get_vec(root) {
        for child in children {
            depth_first_traversal(elements, depths, euler_walk, child, depth + 1);
            euler_walk.push(root.to_owned());
        }
    }
}

fn parse_input(orbits_str: &str) -> Result<MultiMap<String, String>, anyhow::Error> {
    orbits_str
        .lines()
        .map(|orbit| {
            orbit
                .split(')')
                .map(|s| s.to_string())
                .collect_tuple()
                .ok_or(anyhow!("Found an invalid orbit: {}", orbit))
        })
        .try_collect()
}
