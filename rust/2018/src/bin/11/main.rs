use std::error::Error;
use std::{env, fmt, fs};

const GRID_SIZE: (usize, usize) = (300, 300);

type Grid = Vec<Vec<isize>>;

pub fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();

    let input_filename = if args.len() == 2 {
        &args[1]
    } else {
        "input.txt"
    };

    let grid_serial_number: usize = fs::read_to_string(input_filename)?.trim().parse()?;

    // Technically, I could compute the grid along with the SAT, and
    // it might be faster since it would be one pass, but for the sake
    // of cleanliness and overall generality I decided to split them
    // both up, so that I could test them separately.
    let grid = construct_grid(grid_serial_number, GRID_SIZE);
    let summed_area_table = compute_summed_area_table(&grid)?;

    let mut grid_sums = vec![];

    for size in 1..=GRID_SIZE.0 {
        for yi in 0..GRID_SIZE.1 {
            for xi in 0..GRID_SIZE.0 {
                if xi.checked_sub(size).is_none() || yi.checked_sub(size).is_none() {
                    continue;
                }

                let square_sum = summed_area_table[yi][xi]
                    - summed_area_table[yi][xi - size]
                    - summed_area_table[yi - size][xi]
                    + summed_area_table[yi - size][xi - size];

                grid_sums.push((square_sum, (xi - size) + 2, (yi - size) + 2, size));
            }
        }
    }

    println!("{:?}", grid_sums.iter().max_by_key(|v| v.0).unwrap());

    Ok(())
}

fn construct_grid(grid_serial_number: usize, grid_size: (usize, usize)) -> Grid {
    let power_level = |x: usize, y: usize| -> isize {
        let rack_id = x + 10;
        let mut power_level = rack_id * y + grid_serial_number;
        power_level *= rack_id;

        ((power_level / 100) % 10) as isize - 5
    };

    (1..=grid_size.1)
        .map(|yi| (1..=grid_size.0).map(|xi| power_level(xi, yi)).collect())
        .collect()
}

fn compute_summed_area_table(grid: &Grid) -> Result<Grid, NonRectError> {
    // Asumming the grid is actually rectangular, we can assign all
    // the Vecs with the same row-length capacity to help optimize
    // with memory a teeny bit.
    let mut summed_area_table = vec![Vec::with_capacity(grid[0].len()); grid.len()];

    for (yi, row) in grid.iter().enumerate() {
        for (xi, &value) in row.iter().enumerate() {
            // The value of the summed-area table at (x, y) is simply (where I
            // provides previous values in the table, and i provides values in
            // the original grid):
            //
            // I(x, y) = i(x, y) + I(x - 1, y) + I(x, y - 1) - I(x - 1, y - 1)
            //
            // If any of these values do not exist, they are replaced with 0.

            // I(x, y - 1)
            let north = match yi {
                0 => &0,
                _ => {
                    // However, if this particular value doesn't exist, then we
                    // know that we have an x-index that's not accessible on a
                    // previous row. This means the grid were working with is
                    // actually non-rectangular, which means we should return an
                    // error here.
                    summed_area_table
                        .get(yi - 1)
                        .and_then(|row| row.get(xi))
                        .ok_or(NonRectError { xi, yi })?
                }
            };

            // I(x - 1, y)
            let west = match xi {
                0 => &0,
                _ => &summed_area_table[yi][xi - 1],
            };

            // I(x - 1, y - 1)
            let northwest = match (xi, yi) {
                (0, _) => &0,
                (_, 0) => &0,
                (_, _) => summed_area_table
                    .get(yi - 1)
                    .and_then(|row| row.get(xi - 1))
                    .unwrap_or(&0),
            };

            let summed_values = value + north + west - northwest;

            summed_area_table[yi].push(summed_values);
        }
    }

    Ok(summed_area_table)
}

#[derive(Debug, Clone)]
struct NonRectError {
    xi: usize,
    yi: usize,
}

impl fmt::Display for NonRectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "grid is not a rectangular 2d Vec: column {} is not valid on row {}, but it is on row {}",
            self.xi, self.yi - 1, self.yi
        )
    }
}

impl Error for NonRectError {}
