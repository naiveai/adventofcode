#![feature(default_free_fn)]

use binary_heap_plus::*;
use cached::proc_macro::cached;
use itertools::Itertools;
use std::{
    cmp::{max, min, Reverse},
    collections::HashSet,
    default::default,
    env,
    error::Error,
    fmt, fs,
    hash::{Hash, Hasher},
    rc::Rc,
};

pub fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect_vec();

    let input_filename = if args.len() == 2 {
        &args[1]
    } else {
        "input.txt"
    };

    let cave_info_str = fs::read_to_string(input_filename)?;

    let (depth, target) = parse_input(&cave_info_str)?;

    let result = cave_search(depth, target).expect("No path found");

    println!("Minimum time to target: {}", result.path_cost);

    Ok(())
}

fn cave_search(depth: usize, target: Location) -> Option<CaveNode> {
    const MOVE_COST: usize = 1;
    const SWITCH_COST: usize = 7;

    use Tool::*;

    // path_cost and prev don't matter here
    let goal = CaveNode {
        location: target,
        tool: Some(Torch),
        ..default()
    };

    let mut frontier = BinaryHeap::from_vec_cmp(
        vec![CaveNode {
            tool: Some(Torch),
            ..default()
        }],
        KeyComparator(|n: &CaveNode| {
            Reverse(
                n.path_cost
                    + n.location.manhattan_distance(&target) * MOVE_COST
                    + if n.tool != goal.tool { SWITCH_COST } else { 0 },
            )
        }),
    );

    fn possible_tools(region: Region) -> Vec<Option<Tool>> {
        match region {
            Region::Rocky => vec![Some(Torch), Some(ClimbingGear)],
            Region::Wet => vec![Some(ClimbingGear), None],
            Region::Narrow => vec![Some(Torch), None],
        }
    }

    let expand = |node: &CaveNode| -> Vec<CaveNode> {
        let mut expanded = vec![];

        // Add all possibilities for switching tools
        let node_region = get_region_type(calculate_erosion_level(node.location, depth, target));

        for other_tool in possible_tools(node_region) {
            if other_tool != node.tool {
                expanded.push(CaveNode {
                    location: node.location,
                    tool: other_tool,
                    path_cost: node.path_cost + SWITCH_COST,
                    prev: Some(Rc::new(node.clone())),
                });
            }
        }

        // Add all possibilities for moving to an adjacent region
        for adj in node.location.adjacent() {
            let adj_region = get_region_type(calculate_erosion_level(adj, depth, target));

            if possible_tools(adj_region).contains(&node.tool) {
                expanded.push(CaveNode {
                    location: adj,
                    tool: node.tool,
                    path_cost: node.path_cost + MOVE_COST,
                    prev: Some(Rc::new(node.clone())),
                });
            }
        }

        expanded
    };

    let mut explored = HashSet::new();

    while let Some(current) = frontier.pop() {
        if explored.contains(&current) {
            continue;
        }

        if &current == &goal {
            return Some(current);
        }

        for next in expand(&current) {
            frontier.push(next);
        }

        explored.insert(current);
    }

    None
}

#[derive(Clone, Default)]
struct CaveNode {
    location: Location,
    tool: Option<Tool>,
    path_cost: usize,
    // We use Rc as opposed to Box here because it can be cloned really inexpensively,
    // because its clone points to the same heap allocation
    prev: Option<Rc<CaveNode>>,
}

impl PartialEq for CaveNode {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location && self.tool == other.tool
    }
}

impl Eq for CaveNode {}

impl Hash for CaveNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.location.hash(state);
        self.tool.hash(state);
    }
}

impl fmt::Debug for CaveNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("")
            .field(&self.location)
            .field(&self.tool)
            .field(&self.path_cost)
            .finish()
    }
}

#[cached]
fn calculate_erosion_level(location: Location, depth: usize, target: Location) -> usize {
    let geologic_index = match location {
        Location { x: 0, y: 0 } => 0,
        Location { x, y } if x == target.x && y == target.y => 0,
        Location { x, y: 0 } => x * 16807,
        Location { x: 0, y } => y * 48271,
        Location { x, y } => {
            calculate_erosion_level(Location { x: x - 1, y }, depth, target)
                * calculate_erosion_level(Location { x, y: y - 1 }, depth, target)
        }
    };

    (geologic_index + depth) % 20183
}

fn get_region_type(erosion_level: usize) -> Region {
    match erosion_level % 3 {
        0 => Region::Rocky,
        1 => Region::Wet,
        2 => Region::Narrow,
        // Mathematically impossible.
        _ => unsafe { std::hint::unreachable_unchecked() },
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Region {
    Rocky,
    Wet,
    Narrow,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
enum Tool {
    Torch,
    ClimbingGear,
}

fn parse_input(cave_info_str: &str) -> Result<(usize, Location), &str> {
    let cave_info_lines = cave_info_str.lines().collect_vec();
    let (depth_line, target_line) = (cave_info_lines[0], cave_info_lines[1]);

    let (depth_str, target_str) = (
        depth_line
            .strip_prefix("depth: ")
            .ok_or("Invalid depth line format")?,
        target_line
            .strip_prefix("target: ")
            .ok_or("Invalid target line format")?,
    );

    let (target_x_str, target_y_str) = target_str
        .split(",")
        .collect_tuple()
        .ok_or("Invalid target coordinate format")?;

    Ok((
        depth_str.parse().map_err(|_| "Depth is not a number")?,
        Location {
            x: target_x_str
                .parse()
                .map_err(|_| "Target X is not a number")?,
            y: target_y_str
                .parse()
                .map_err(|_| "Target Y is not a number")?,
        },
    ))
}

#[derive(Eq, PartialEq, Default, Hash, Copy, Clone)]
struct Location {
    x: usize,
    y: usize,
}

impl Location {
    fn manhattan_distance(&self, other: &Self) -> usize {
        (max(self.x, other.x) - min(self.x, other.x))
            + (max(self.y, other.y) - min(self.y, other.y))
    }

    fn adjacent(&self) -> Vec<Self> {
        let mut adjacent_locations = vec![
            Location {
                x: self.x,
                y: self.y + 1,
            },
            Location {
                x: self.x + 1,
                y: self.y,
            },
        ];

        if self.y > 0 {
            adjacent_locations.push(Location {
                x: self.x,
                y: self.y - 1,
            });
        }

        if self.x > 0 {
            adjacent_locations.push(Location {
                x: self.x - 1,
                y: self.y,
            });
        }

        adjacent_locations
    }
}

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("").field(&self.x).field(&self.y).finish()
    }
}
