use log::{info,debug};

use ndarray::{Array,Array2};

use super::CrosswordGrid;
use super::Cell;
use super::Location;
use super::VALID_ANSWERCHARS;

fn coord_isize_to_usize(value: isize, shift: isize) -> usize {
    (value + shift) as usize
}

fn cell_to_u8(cell: &Cell) -> u8 {
    if cell.is_empty() {
        0
    } else if cell.is_black() {
        1
    } else {
        let c = cell.to_char();
        let c_index = VALID_ANSWERCHARS.find(c).unwrap();
        (c_index as u8) + 2
    }
}

impl CrosswordGrid {
    pub fn to_matrix(&self) -> Array2<u8> {
        let mut matrix: Array2<u8> = Array::zeros(self.get_grid_dimensions());
        let mut row: isize = self.top_left_cell_index.0 + 1;
        let mut col: isize = self.top_left_cell_index.1 + 1;

        while row < self.bottom_right_cell_index.0 {
            while col < self.bottom_right_cell_index.1 {
                let cell = self.cell_map.get(&Location(row, col)).unwrap();
                matrix[[coord_isize_to_usize(row, - self.top_left_cell_index.0 - 1),
                        coord_isize_to_usize(col, - self.top_left_cell_index.1 - 1)]] = cell_to_u8(cell);
                col += 1;
            }
            col = self.top_left_cell_index.1 + 1;
            row += 1;
        }
        matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::CrosswordGridBuilder;

    #[test]
    fn test_to_matrix() {
        crate::logging::init_logger(true);
        let grid = CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
        println!("{:#?}", grid.to_matrix());

        let grid = CrosswordGridBuilder::new().from_file("tests/resources/everyman_starter.txt");
        println!("{:#?}", grid.to_matrix());
    }
}
