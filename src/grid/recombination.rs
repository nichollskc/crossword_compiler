use log::debug;
use std::cmp;

use super::CrosswordGrid;
use super::Location;

impl CrosswordGrid {
    pub fn merge_with_grid(&mut self, other: &CrosswordGrid, row_shift: isize, col_shift: isize) {
        self.grow_to_fit_merge(other, row_shift, col_shift);

        for (word_id, other_word) in other.word_map.iter() {
            if let Some((start_location, _, direction)) = other_word.get_location() {
                let this_word = self.word_map.get(word_id).unwrap();
                assert!(!this_word.is_placed());

                let shifted_location = start_location.relative_location(row_shift, col_shift);
                let success = self.try_place_word_in_cell(shifted_location, *word_id, 0, direction, true);
                assert!(success);
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

        self.expand_to_fit_cell(new_top_left);
        self.expand_to_fit_cell(new_bottom_right);
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use super::super::CrosswordGridBuilder;

    #[test]
    fn test_merge() {
        crate::logging::init_logger(true);
        let mut grid1 = CrosswordGridBuilder::new().from_file("tests/resources/everyman_starter.txt");
        let mut grid2 = CrosswordGridBuilder::new().from_file("tests/resources/everyman_compatible.txt");
        println!("{:#?}", grid1);
        println!("{:#?}", grid2);
        grid2.update_word_id(1, 101);
        grid2.update_word_id(5, 105);
        grid2.update_word_id(9, 109);
        println!("{:#?}", grid2);
        
        grid1.add_unplaced_word_at_id("RUSTY", "", 101, None); 
        grid1.add_unplaced_word_at_id("SIXTY", "", 105, None);
        grid1.add_unplaced_word_at_id("TROUT", "", 109, None);
        grid1.merge_with_grid(&grid2, -2, 2);
        println!("{}", grid1.to_string());
    }

}
