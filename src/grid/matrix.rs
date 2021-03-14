use log::{info,debug};
use std::cmp;

use ndarray::{Array,ArrayView,Array2};

use super::CrosswordGrid;
use super::Cell;
use super::Location;
use super::VALID_ANSWERCHARS;

fn coord_isize_to_usize(value: isize, shift: isize) -> usize {
    (value + shift) as usize
}

fn cell_to_i16(cell: &Cell) -> i16 {
    if cell.is_empty() {
        0
    } else if cell.is_black() {
        1
    } else {
        let c = cell.to_char();
        let c_index = VALID_ANSWERCHARS.find(c).unwrap();
        (c_index as i16) + 2
    }
}

#[derive(Debug)]
struct CrosswordGridMatrix {
    matrix: Array2<i16>,
    row_shift: isize,
    col_shift: isize,
    nrows: usize,
    ncols: usize,
}

impl CrosswordGridMatrix {
    pub fn empty(nrows: usize, ncols: usize, row_shift: isize, col_shift: isize) -> Self {
        CrosswordGridMatrix {
            matrix: Array::zeros((nrows, ncols)),
            row_shift,
            col_shift,
            nrows: nrows,
            ncols: ncols,
        }
    }

    pub fn set_coord(&mut self, row: isize, col: isize, value: i16) {
        self.matrix[[coord_isize_to_usize(row, self.row_shift),
                     coord_isize_to_usize(col, self.col_shift)]] = value;
    }

    pub fn padded_to_size(&self, nrows: usize, ncols: usize) -> Self {
        let mut matrix = Array::zeros((nrows, ncols));
        let mut used_slice = matrix.slice_mut(s![..self.nrows, ..self.ncols]);
        used_slice += &ArrayView::from(&self.matrix);

        CrosswordGridMatrix {
            matrix,
            row_shift: self.row_shift,
            col_shift: self.col_shift,
            nrows,
            ncols,
        }
    }

    pub fn shifted(&self, extra_rows: usize, extra_cols: usize) -> Self {
        let nrows: usize = self.nrows + extra_rows;
        let ncols: usize = self.ncols + extra_cols;
        let mut matrix = Array::zeros((nrows, ncols));
        let mut used_slice = matrix.slice_mut(s![extra_rows..nrows, extra_cols..ncols]);
        used_slice += &ArrayView::from(&self.matrix);

        CrosswordGridMatrix {
            matrix,
            row_shift: self.row_shift + extra_rows as isize,
            col_shift: self.col_shift + extra_cols as isize,
            nrows,
            ncols,
        }
    }

    pub fn compatible_with_matrix(&self, other: &CrosswordGridMatrix, other_row_shift: isize, other_col_shift: isize) -> bool {
        let other_shifted = other.shifted(other_row_shift as usize,
                                          other_col_shift as usize);

        let max_rows = cmp::max(self.nrows, other_shifted.nrows);
        let max_cols = cmp::max(self.ncols, other_shifted.ncols);

        let padded1 = self.padded_to_size(max_rows, max_cols);
        let padded2 = other_shifted.padded_to_size(max_rows, max_cols);

        debug!("{:#?}\n{:#?}", padded1, padded2);
        let nonempty_cells_shared: Array2<i16> = &padded1.matrix * &padded2.matrix;
        let cells_mismatch: Array2<i16> = &padded1.matrix - &padded2.matrix;
        debug!("Cells shared: {:#?}", nonempty_cells_shared);
        debug!("Cells mismatched: {:#?}", cells_mismatch);

        let cells_shared_and_mismatched = nonempty_cells_shared * cells_mismatch;
        debug!("Cells shared and mismatched: {:#?}", cells_shared_and_mismatched);

        cells_shared_and_mismatched.sum() == 0
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
                matrix.set_coord(row, col, cell_to_i16(cell));
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
    use super::super::Word;
    use super::super::Direction;

    use std::collections::HashMap;

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

    #[test]
    fn test_matrix_compatible() {
        crate::logging::init_logger(true);
        let mut word_map: HashMap<usize, Word> = HashMap::new();
        word_map.insert(0, Word::new_parsed("BEE").unwrap());
        word_map.insert(1, Word::new_parsed("BEAR").unwrap());
        let bee_grid = CrosswordGrid::new_from_wordmap_single_placed(0, Direction::Across, word_map.clone());
        let bear_grid = CrosswordGrid::new_from_wordmap_single_placed(1, Direction::Down, word_map);
        println!("{:#?}", bee_grid.to_matrix());
        println!("{:#?}", bear_grid.to_matrix());

        assert!(bee_grid.to_matrix().compatible_with_matrix(&bear_grid.to_matrix(), 0, 0));
        assert!(!bee_grid.to_matrix().compatible_with_matrix(&bear_grid.to_matrix(), 0, 1));
    }
}
