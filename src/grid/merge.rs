use log::debug;
use std::cmp;

use super::CrosswordGrid;
use super::Location;

impl CrosswordGrid {
    fn words_placed_compatible(&self, other: &CrosswordGrid) -> bool {
        let mut compatible = true;
        for (word_id, word) in self.word_map.iter() {
            if word.is_placed() && other.word_map.get(word_id).unwrap().is_placed() {
                compatible = false;
            }
        }
        compatible
    }

    pub fn try_merge_with_grid(&mut self, other: &CrosswordGrid) -> bool {
        // First check if the word lists are compatible i.e. that they don't share any placed words
        let mut success = self.words_placed_compatible(other);
        if success {
            // Then look to see if there is a way for the grids to fit together
            let configuration = self.find_best_compatible_configuration_for_merge(other);
            if let Some((row_shift, col_shift)) = configuration {
                self.merge_with_grid(other, row_shift, col_shift);
            } else {
                // If no valid configuration, this is a failure
                success = false;
            }
        }
        success
    }

    pub fn merge_with_grid(&mut self, other: &CrosswordGrid, row_shift: isize, col_shift: isize) {
        assert!(other.black_cells_valid());
        self.grow_to_fit_merge(other, row_shift, col_shift);
        self.fill_black_cells();

        for (word_id, other_word) in other.word_map.iter() {
            if let Some((start_location, _, direction)) = other_word.get_location() {
                debug!("Attempting to add word {:?} to grid\n{:?}", word_id, self);
                let this_word = self.word_map.get(word_id).unwrap();
                assert!(!this_word.is_placed());

                let shifted_location = start_location.relative_location(row_shift, col_shift);
                let success = self.try_place_word_in_cell(shifted_location, *word_id, 0, direction, true);
                assert!(success, "Failed to place word {} in location {:?}. Other word: {:?}", word_id, shifted_location, other_word);
            }
        }
        self.check_valid();
    }

    fn grow_to_fit_merge(&mut self, other: &CrosswordGrid, row_shift: isize, col_shift: isize) {
        let min_row = cmp::min(other.top_left_cell_index.0 + row_shift,
                               self.top_left_cell_index.0);
        let min_col = cmp::min(other.top_left_cell_index.1 + col_shift,
                               self.top_left_cell_index.1);
        let max_row = cmp::max(other.bottom_right_cell_index.0 + row_shift,
                               self.bottom_right_cell_index.0);
        let max_col = cmp::max(other.bottom_right_cell_index.1 + col_shift,
                               self.bottom_right_cell_index.1);
        let new_top_left = Location(min_row, min_col);
        let new_bottom_right = Location(max_row, max_col);
        debug!("Expanding to fit to cells {:?} {:?}", new_top_left, new_bottom_right);

        self.expand_to_fit_cell(new_top_left);
        self.expand_to_fit_cell(new_bottom_right);
        debug!("{:?}", self);
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use super::super::CrosswordGridBuilder;
    use super::super::Direction;

    fn setup_merge() -> (CrosswordGrid, CrosswordGrid) {
        let mut grid1 = CrosswordGridBuilder::new().from_file("tests/resources/everyman_starter.txt");
        let mut grid2 = CrosswordGrid::new_single_word("SIXTY");
        grid2.update_word_id(0, 100);
        let rusty_id = grid2.add_unplaced_word("RUSTY", "", None);
        grid2.update_word_id(rusty_id, 101);
        let trout_id = grid2.add_unplaced_word("TROUT", "", None);
        grid2.update_word_id(trout_id, 102);

        println!("{:#?}", grid2);
        assert!(grid2.try_place_word_in_cell_connected(Location(0, 4), 101, 4, Direction::Down));
        assert!(grid2.try_place_word_in_cell_connected(Location(0, 3), 102, 0, Direction::Down));
        
        grid1.add_unplaced_word_at_id("SIXTY", "", 100, None);
        grid1.add_unplaced_word_at_id("RUSTY", "", 101, None); 
        grid1.add_unplaced_word_at_id("TROUT", "", 102, None);
        println!("{:#?}", grid1);
        println!("{:#?}", grid2);

        grid2.fit_to_size();
        grid2.fill_black_cells();
        (grid1, grid2)
    }

    #[test]
    fn test_merge() {
        crate::logging::init_logger(true);
        let (mut grid1, grid2) = setup_merge();
        grid1.merge_with_grid(&grid2, 2, 2);
        println!("{}", grid1.to_string());
    }

    #[test]
    #[should_panic]
    fn test_disconnected() {
        crate::logging::init_logger(true);
        let (mut grid1, grid2) = setup_merge();
        grid1.merge_with_grid(&grid2, -10, -10);
    }

    #[test]
    #[should_panic]
    fn test_mismatch() {
        crate::logging::init_logger(true);
        let (mut grid1, grid2) = setup_merge();
        grid1.merge_with_grid(&grid2, 3, 0);
    }

}
