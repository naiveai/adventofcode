use hashbrown::HashMap;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use unit::*;

pub fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();

    let input_filename = if args.len() == 2 {
        &args[1]
    } else {
        "input.txt"
    };

    let string_grid = fs::read_to_string(input_filename)?;

    let mut combat_grid = parse_input(&string_grid)?;
    let mut full_rounds: usize = 0;

    println!("Start");
    print!("{}", combat_grid);
    println!("\n");

    while combat_grid.tick() {
        full_rounds += 1;
        println!("\n");
        println!("Round {}", full_rounds);
        print!("{}", combat_grid);
        println!("\n");
    }

    println!("Final");
    print!("{}", combat_grid);
    println!("\n");

    println!(
        "Outcome: {}",
        full_rounds * combat_grid.units.values().map(|u| u.hp).sum::<usize>()
    );

    Ok(())
}

pub fn parse_input(string_grid: &str) -> Result<CombatGrid, String> {
    let mut grid = HashMap::new();
    let mut units = HashMap::new();
    let mut dimensions = (0, 0);

    for (y, row) in string_grid.lines().enumerate() {
        dimensions.1 += 1;

        for (x, character) in row.chars().enumerate() {
            dimensions.0 += 1;

            let current_location = Location { x, y };

            grid.insert(
                current_location,
                match character {
                    '#' => Environment::Wall,
                    '.' => Environment::Open,
                    'G' | 'E' => {
                        units.insert(
                            current_location,
                            Unit {
                                team: if character == 'G' {
                                    UnitTeam::Goblin
                                } else {
                                    UnitTeam::Elf
                                },
                                location: current_location,
                                hp: 200,
                                attack_power: 3,
                            },
                        );

                        Environment::Open
                    }
                    _ => {
                        return Err(format!("Invalid input character: {}", character));
                    }
                },
            );
        }
    }

    dimensions.0 /= dimensions.1;

    Ok(CombatGrid {
        grid,
        units,
        dimensions,
    })
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub struct Location {
    x: usize,
    y: usize,
}

impl Location {
    fn adjacent(&self) -> [Self; 4] {
        [
            Location {
                x: self.x,
                y: self.y - 1,
            },
            Location {
                x: self.x,
                y: self.y + 1,
            },
            Location {
                x: self.x - 1,
                y: self.y,
            },
            Location {
                x: self.x + 1,
                y: self.y,
            },
        ]
    }
}

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Ord for Location {
    fn cmp(&self, other: &Self) -> Ordering {
        self.y.cmp(&other.y).then(self.x.cmp(&other.x))
    }
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct CombatGrid {
    pub grid: HashMap<Location, Environment>,
    pub units: HashMap<Location, Unit>,
    pub dimensions: (usize, usize),
}

impl fmt::Display for CombatGrid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.dimensions.1 {
            let mut row_units = Vec::new();

            for x in 0..self.dimensions.0 {
                let location = Location { x, y };

                if let Some(unit) = self.units.get(&location) {
                    write!(f, "{:?}", unit.team)?;
                    row_units.push(unit);
                } else if let Some(env) = self.grid.get(&location) {
                    write!(f, "{:?}", env)?;
                }
            }

            write!(f, "\t")?;

            for unit in row_units {
                write!(f, " [{:?}] ", unit)?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

impl CombatGrid {
    pub fn tick(&mut self) -> bool {
        let mut unit_locations = self.units.keys().cloned().collect::<Vec<_>>();
        unit_locations.sort_unstable();

        for unit_location in unit_locations.iter() {
            // This unit may have since died by the hands of another
            // by the time we have gotten to it, so check if it's still there.
            let unit = match self.units.get(unit_location) {
                Some(unit) => unit.clone(),
                None => continue,
            };

            let enemy_units = self
                .units
                .iter()
                .filter(|(_, u)| u.is_enemy(&unit))
                .map(|(l, u)| (*l, u.clone()))
                .collect::<HashMap<_, _>>();

            if enemy_units.is_empty() {
                return false; // Combat has ended, one team has won.
            }

            if let Some(attacked_unit_location) = unit.maybe_attack(&enemy_units) {
                self.attack_unit(unit_location, &attacked_unit_location);
                continue;
            }

            if let Some(move_location) = unit.maybe_move(&enemy_units, |l| self.is_open_fn(l)) {
                // Get the new Unit with the updated location. The old reference is stale
                // otherwise, leading to attack behaviour based on the old location, which never
                // actually works out, because the only reason any unit moves is because its
                // old location is not adjacent to any enemy unit.
                let unit = self.move_unit(unit_location, &move_location);

                if let Some(attacked_unit_location) = unit.maybe_attack(&enemy_units) {
                    self.attack_unit(&move_location, &attacked_unit_location);
                }
            }
        }

        true
    }

    fn attack_unit(&mut self, current_unit_location: &Location, attacked_unit_location: &Location) {
        let current_unit = &self.units[current_unit_location].clone();
        let mut attacked_unit = self.units.get_mut(attacked_unit_location).unwrap();

        // This protects against overflows in the usize
        attacked_unit.hp = attacked_unit.hp.saturating_sub(current_unit.attack_power);

        if attacked_unit.is_dead() {
            self.units.remove(attacked_unit_location);
        }
    }

    fn move_unit(&mut self, current_unit_location: &Location, new_location: &Location) -> Unit {
        let new_location = *new_location;
        let mut current_unit = self.units.remove(current_unit_location).unwrap();

        current_unit.location = new_location;
        self.units.insert(new_location, current_unit.clone());

        current_unit
    }

    fn is_open_fn(&self, location: &Location) -> bool {
        if self.units.contains_key(location) {
            false
        } else if let Some(env) = self.grid.get(location) {
            env == &Environment::Open
        } else {
            false
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum Environment {
    Wall,
    Open,
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Wall => '#',
                Self::Open => '.',
            }
        )
    }
}

mod unit {
    use super::*;

    #[derive(Eq, PartialEq, Copy, Clone)]
    pub enum UnitTeam {
        Goblin,
        Elf,
    }

    impl fmt::Debug for UnitTeam {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", if self == &Self::Elf { 'E' } else { 'G' })
        }
    }

    #[derive(Eq, PartialEq, Clone)]
    pub struct Unit {
        pub team: UnitTeam,
        pub location: Location,
        pub hp: usize,
        pub attack_power: usize,
    }

    impl fmt::Debug for Unit {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}({}) @ {:?}", self.team, self.hp, self.location)
        }
    }

    impl Unit {
        pub fn is_enemy(&self, other: &Self) -> bool {
            self.team != other.team
        }

        pub fn is_dead(&self) -> bool {
            self.hp == 0
        }

        pub fn maybe_attack(&self, enemy_units: &HashMap<Location, Unit>) -> Option<Location> {
            let mut adjacent_enemy_units = enemy_units
                .values()
                .filter(|u| self.location.adjacent().contains(&u.location))
                .collect::<Vec<_>>();

            adjacent_enemy_units.sort_unstable_by_key(|unit| (unit.hp, unit.location));

            adjacent_enemy_units.reverse();
            adjacent_enemy_units.pop().map(|u| u.location)
        }

        pub fn maybe_move(
            &self,
            enemy_units: &HashMap<Location, Unit>,
            is_open_fn: impl Fn(&Location) -> bool,
        ) -> Option<Location> {
            let mut frontier = self
                .location
                .adjacent()
                .iter()
                .cloned()
                .filter(&is_open_fn)
                .map(|l| {
                    Reverse(SearchNode {
                        distance: 1,
                        current_location: l,
                        starting_location: l,
                    })
                })
                .collect::<BinaryHeap<_>>();

            let mut explored = Vec::new();

            while let Some(Reverse(next)) = frontier.pop() {
                for next_adjacent in next.current_location.adjacent().iter().cloned() {
                    if explored.contains(&next_adjacent) {
                        continue;
                    }

                    if !is_open_fn(&next_adjacent) {
                        if enemy_units.contains_key(&next_adjacent) {
                            return Some(next.starting_location);
                        }

                        continue;
                    }

                    frontier.push(Reverse(SearchNode {
                        distance: next.distance + 1,
                        current_location: next_adjacent,
                        starting_location: next.starting_location,
                    }));

                    explored.push(next_adjacent);
                }
            }

            None
        }
    }

    // Private helper to make maybe_move easier to keep track of
    #[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
    struct SearchNode {
        distance: usize,
        current_location: Location,
        starting_location: Location,
    }
}
