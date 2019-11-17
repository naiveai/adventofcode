use std::{env, fs, fmt};
use std::error::Error;

pub fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();

    let input_filename = match args.len() {
        2 => &args[1],
        _ => "input.txt"
    };

    let grid_serial_number: usize =
        fs::read_to_string(input_filename)?.trim().parse()?;

    let grid = construct_grid(grid_serial_number, (300, 300));

    let example = vec![
        vec![1, 2, 3, 4, 6],
        vec![5, 3, 8, 1, ],
        vec![4, 6, 7, 5, 5],
        vec![2, 4, 8, 9, 4],
    ];

    println!("{:?}", compute_summed_area_table(&example).unwrap());

    // let summed_area_table = compute_summed_area_table(grid);

    Ok(())
}

fn construct_grid(
    grid_serial_number: usize,
    grid_size: (usize, usize)
) -> Vec<Vec<isize>> {
    let power_level = |x: usize, y: usize| -> isize {
        let rack_id = x + 10;
        let mut power_level = rack_id * y + grid_serial_number;
        power_level *= rack_id;

        (((power_level / 100) % 10) as isize - 5)
    };

    (1..=grid_size.1)
    .map(|yi|
        (1..=grid_size.0)
        .map(|xi| power_level(xi, yi))
        .collect()
    )
    .collect()
}

fn compute_summed_area_table(grid: &Vec<Vec<isize>>) -> Result<Vec<Vec<isize>>, NonRectError> {
    let mut summed_area_table =
        vec![Vec::with_capacity(grid[0].len()); grid.len()];

    for (yi, row) in grid.iter().enumerate() {
        for (xi, &value) in row.iter().enumerate() {
            // The value of the summed-area table at (x, y) is simply (where I
            // provides previous values in the table, and i provides values in
            // the original grid):
            //
            // I(x, y) = i(x, y) + I(x - 1, y) + I(x, y - 1) - I(x - 1, y - 1)
            //
            // If any of these values do not exist, they are replaced with 0.

            let (prev_row, prev_column_idx) = (
                yi.checked_sub(1).and_then(|i| summed_area_table.get(i)),
                xi.checked_sub(1)
            );

            let summed_values =
                value +
                // I(x, y - 1)
                match prev_row {
                    None => &0,
                    Some(prev_row_vec) => match prev_row_vec.get(xi) {
                        Some(v) => v,
                        // However, if I(x, y - 1) does not exist even
                        // though y - 1 exists, we know that we have
                        // an x-index that's not accessible on the
                        // previous row, meaning the grid we got is
                        // not actually rectangular, so we return the
                        // error there.
                        None => return Err(NonRectError { xi, yi })
                    }
                } +
                // I(x - 1, y)
                (prev_column_idx
                     .and_then(|i| summed_area_table[yi].get(i))
                     .unwrap_or(&0)) -
                // I(x - 1, y - 1)
                (prev_row
                     .map_or(&0, |r| {
                         prev_column_idx
                             .and_then(|i| r.get(i))
                             .unwrap_or(&0)
                     }));

            summed_area_table[yi].push(summed_values);
        }
    }

    Ok(summed_area_table)
}

#[derive(Debug, Clone)]
struct NonRectError {
    xi: usize,
    yi: usize
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
