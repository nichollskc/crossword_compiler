use log::{info,warn,debug,error};

use super::CrosswordGrid;

impl CrosswordGrid {
    pub fn count_all_words(&self) -> usize {
        self.word_map.len()
    }

    pub fn count_placed_words(&self) -> usize {
        self.word_map.values().filter(|w| w.is_placed()).count()
    }

    pub fn count_intersections(&self) -> usize {
        let mut intersections: usize = 0;
        for cell in self.cell_map.values() {
            if cell.is_intersection() {
                intersections += 1
            }
        }
        intersections
    }

    pub fn get_grid_dimensions(&self) -> (usize, usize) {
        let nrows: usize = (self.bottom_right_cell_index.0 - self.top_left_cell_index.0 - 1) as usize;
        let ncols: usize = (self.bottom_right_cell_index.1 - self.top_left_cell_index.1 - 1) as usize;
        (nrows, ncols)
    }

    pub fn count_filled_cells(&self) -> usize {
        self.cell_map.values().filter(|c| c.contains_letter()).count()
    }

    pub fn count_empty_cells(&self) -> usize {
        let (nrows, ncols) = self.get_grid_dimensions();
        nrows * ncols - self.count_filled_cells()
    }
}
