use crate::graph::Graph;
use std::cmp;
use log::{info,warn,debug,error};
use std::collections::HashMap;

use super::CrosswordGrid;
use super::Location;

use super::Cell;

impl CrosswordGrid {
    pub fn expand_to_fit_cell(&mut self, location: Location) {
        while location.0 < self.top_left_cell_index.0 {
            self.add_empty_row(self.top_left_cell_index.0 - 1);
        }
        while location.0 > self.bottom_right_cell_index.0 {
            self.add_empty_row(self.bottom_right_cell_index.0 + 1);
        }
        while location.1 < self.top_left_cell_index.1 {
            self.add_empty_col(self.top_left_cell_index.1 - 1);
        }
        while location.1 > self.bottom_right_cell_index.1 {
            self.add_empty_col(self.bottom_right_cell_index.1 + 1);
        }
    }

    fn add_empty_row(&mut self, new_row: isize) {
        debug!("Adding new row at {}, top left is {:?}, bottom right is {:?}", new_row, self.top_left_cell_index, self.bottom_right_cell_index);
        let mut col = self.top_left_cell_index.1;
        while col <= self.bottom_right_cell_index.1 {
            let location = Location(new_row, col);
            self.cell_map.insert(location, Cell::empty(location));
            col += 1;
        }
        if new_row > self.bottom_right_cell_index.0 {
            self.bottom_right_cell_index = Location(new_row, self.bottom_right_cell_index.1);
        } else if new_row < self.top_left_cell_index.0 {
            self.top_left_cell_index = Location(new_row, self.top_left_cell_index.1);
        }
    }

    fn add_empty_col(&mut self, new_col: isize) {
        debug!("Adding new col at {}", new_col);
        let mut row = self.top_left_cell_index.0;
        while row <= self.bottom_right_cell_index.0 {
            let location = Location(row, new_col);
            self.cell_map.insert(location, Cell::empty(location));
            row += 1;
        }
        if new_col > self.bottom_right_cell_index.1 {
            self.bottom_right_cell_index = Location(self.bottom_right_cell_index.0, new_col);
        } else if new_col < self.top_left_cell_index.1 {
            self.top_left_cell_index = Location(self.top_left_cell_index.0, new_col);
        }
    }

    fn ensure_buffer_exists(&mut self) {
        if self.count_filled_cells_row(self.top_left_cell_index.0) > 0 {
            self.add_empty_row(self.top_left_cell_index.0 - 1);
        }
        if self.count_filled_cells_row(self.bottom_right_cell_index.0) > 0 {
            self.add_empty_row(self.bottom_right_cell_index.0 + 1);
        }
        if self.count_filled_cells_col(self.top_left_cell_index.1) > 0 {
            self.add_empty_col(self.top_left_cell_index.1 - 1);
        }
        if self.count_filled_cells_col(self.bottom_right_cell_index.1) > 0 {
            self.add_empty_col(self.bottom_right_cell_index.1 + 1);
        }
    }

    fn remove_row(&mut self, row: isize) {
        let mut col = self.top_left_cell_index.1;
        while col <= self.bottom_right_cell_index.1 {
            self.cell_map.remove(&Location(row, col));
            col += 1;
        }
        if row == self.bottom_right_cell_index.0 {
            self.bottom_right_cell_index = self.bottom_right_cell_index.relative_location(-1, 0);
        } else if row == self.top_left_cell_index.0 {
            self.top_left_cell_index = self.top_left_cell_index.relative_location(1, 0);
        }
    }

    fn remove_col(&mut self, col: isize) {
        let mut row = self.top_left_cell_index.0;
        while row <= self.bottom_right_cell_index.0 {
            self.cell_map.remove(&Location(row, col));
            row += 1;
        }
        if col == self.bottom_right_cell_index.1 {
            self.bottom_right_cell_index = self.bottom_right_cell_index.relative_location(0, -1);
        } else if col == self.top_left_cell_index.1 {
            self.top_left_cell_index = self.top_left_cell_index.relative_location(0, 1);
        }
    }

    fn remove_excess_empty(&mut self) {
        // Remove excess rows
        while self.count_filled_cells_row(self.top_left_cell_index.0 + 1) == 0 {
            self.remove_row(self.top_left_cell_index.0)
        }
        while self.count_filled_cells_row(self.bottom_right_cell_index.0 - 1) == 0 {
            self.remove_row(self.bottom_right_cell_index.0)
        }

        // Remove excess columns
        while self.count_filled_cells_col(self.top_left_cell_index.1 + 1) == 0 {
            self.remove_col(self.top_left_cell_index.1)
        }
        while self.count_filled_cells_col(self.bottom_right_cell_index.1 - 1) == 0 {
            self.remove_col(self.bottom_right_cell_index.1)
        }
    }

    fn count_filled_cells_row(&self, row: isize) -> usize {
        let mut col = self.top_left_cell_index.1;
        let mut filled_count: usize = 0;

        while col <= self.bottom_right_cell_index.1 {
            if self.cell_map.get(&Location(row, col)).unwrap().contains_letter() {
                filled_count += 1;
            }
            col += 1;
        }
        filled_count
    }

    fn count_filled_cells_col(&self, col: isize) -> usize {
        let mut row = self.top_left_cell_index.0;
        let mut filled_count: usize = 0;

        while row <= self.bottom_right_cell_index.0 {
            if self.cell_map.get(&Location(row, col)).unwrap().contains_letter() {
                filled_count += 1;
            }
            row += 1;
        }
        filled_count
    }

    /// Trim the grid so that there is exactly one row and column of empty
    /// cells on either side of the grid
    pub fn fit_to_size(&mut self) {
        self.check_valid();

        // First make sure we've got at least one buffer row and buffer column
        self.ensure_buffer_exists();

        // Then check we don't have too many empty rows or columns
        self.remove_excess_empty();
    }
}