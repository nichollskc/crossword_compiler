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

#[derive(Debug)]
struct CrosswordGridMatrix {
    matrix: Array2<u8>,
    row_shift: isize,
    col_shift: isize,
}

impl CrosswordGridMatrix {
    pub fn empty(nrows: usize, ncols: usize, row_shift: isize, col_shift: isize) -> Self {
        CrosswordGridMatrix {
            matrix: Array::zeros((nrows, ncols)),
            row_shift,
            col_shift,
        }
    }

    pub fn set_coord(&mut self, row: isize, col: isize, value: u8) {
        self.matrix[[coord_isize_to_usize(row, self.row_shift),
                     coord_isize_to_usize(col, self.col_shift)]] = value;
    }

    fn compatible_with_matrix(&self, other: CrosswordGridMatrix) -> bool {
        let nonempty_cells_shared: Array2<u8> = &self.matrix * &other.matrix;
        let cells_mismatch: Array2<u8> = &self.matrix - &other.matrix;

        (nonempty_cells_shared * cells_mismatch).sum() > 0
    }
}

impl CrosswordGrid {
    fn to_matrix(&self) -> CrosswordGridMatrix {
        let mut row: isize = self.top_left_cell_index.0 + 1;
        let mut col: isize = self.top_left_cell_index.1 + 1;

        let (nrows, ncols) = self.get_grid_dimensions();
        let mut matrix = CrosswordGridMatrix::empty(nrows, ncols, -row, -col);

        while row < self.bottom_right_cell_index.0 {
            while col < self.bottom_right_cell_index.1 {
                let cell = self.cell_map.get(&Location(row, col)).unwrap();
                matrix.set_coord(row, col, cell_to_u8(cell));
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

        let grid = CrosswordGrid::new_single_word("ALPHA");
        println!("{:#?}", grid.to_matrix());

        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/everyman_starter.txt");
        grid.add_unplaced_word("PROBONO", "", None);
        grid.add_unplaced_word("PASTURE", "", None);
        grid.add_unplaced_word("VETO", "", None);
        grid.add_unplaced_word("EROS", "", None);

        let mut success = true;
        while success {
            success = grid.place_random_word(13);
        }
        println!("{:#?}", grid.to_matrix());
    }
}
