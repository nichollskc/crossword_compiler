use log::{info,debug};

use super::CrosswordGrid;
use super::Location;
use super::Direction;

use super::Word;

impl CrosswordGrid {
    fn fill_black_cells(&mut self) {
        // Clear black cells before starting
        for (_location, cell) in self.cell_map.iter_mut() {
            if cell.is_black() {
                cell.set_empty();
            }
        }

        for word in self.word_map.values() {
            if let Some((start_location, end_location, direction)) = word.get_location() {
                let mut black_cells: Vec<Location> = vec![];
                black_cells.push(start_location.relative_location_directed(-1, direction));
                black_cells.push(end_location.relative_location_directed(1, direction));

                for cell_location in black_cells {
                    if let Some(cell) = self.cell_map.get_mut(&cell_location) {
                        cell.set_black();
                    } else {
                        panic!("Cell doesn't exist! {:#?}, {:#?}", cell_location, word);
                    }
                }
            }
        }
    }

    fn get_adjacent_word_id(&self, location: &Location, move_by: isize, move_direction: Direction, word_direction: Direction) -> Option<usize> {
        let neighbour_location = location.relative_location_directed(move_by, move_direction);
        let cell = self.cell_map.get(&neighbour_location).unwrap();
        debug!("Looking at adjacent cell {:?}", cell);
        match word_direction {
            Direction::Across => cell.get_across_word_id(),
            Direction::Down => cell.get_down_word_id(),
        }
    }

    fn adjacent_word_ids_mismatch(&self, location: &Location, move_by: isize, word_direction: Direction, expected_perp_id: Option<usize>) -> bool {
        let found_perp_id: Option<usize> = self.get_adjacent_word_id(location, move_by, word_direction.rotate(), word_direction.rotate());
        let found_parallel_id: Option<usize> = self.get_adjacent_word_id(location, move_by, word_direction.rotate(), word_direction);
        let invalid: bool;

        if found_parallel_id.is_none() {
            // If the adjacent cell has no parallel word, this is never a problem
            invalid = false;
        } else {
            // This is only allowed if these two cells share a perpendicular word id
            if found_perp_id.is_none() {
                // So if the adjacent cell doesn't even have a perpendicular word ID, this is an
                // automatic failure
                invalid = true;
            } else {
                // If the current cell hasn't got a perpendicular word id, this means we have a
                // flaw in our logic, since the current cell should have been marked as Black (end
                // of word) and so we shouldn't be in this function
                assert!(expected_perp_id.is_some(),
                        "Adjacent cell should only belong to a word_id if this cell also belongs to that word");

                // So there is a parallel word in the adjacent cell and a perpendicular word.
                // Final check is whether the two IDs match. If not, this is a flaw in the logic,
                // so panic! If we don't panic in the assert, the two are equal so the word ids do
                // not cause mismatch
                assert_eq!(expected_perp_id, found_perp_id,
                        "Adjacent cells have words in the same direction which are different! E.g. BEARBUTTON - two words without a Black space to separate them.");
                invalid = false
            }
        }
        invalid
    }

    fn neighbouring_cells_empty(&self, location: Location, neighbour_moves: Vec<(isize, isize)>) -> bool {
        if !self.cell_map.get(&location).unwrap().contains_letter() {
            // If the cell is empty, it cannot be added to - it is not an open cell
            false
        } else {
            let mut result = false;
            for relative_move in neighbour_moves {
                if self.cell_map.get(&location.relative_location(relative_move.0, relative_move.1)).unwrap().is_empty() {
                    result = true;
                }
            }
            result
        }
    }

    fn cell_is_open_across(&self, location: Location) -> bool {
        // If there is already an across word for this cell, can't place another across word here
        if self.cell_map.get(&location).unwrap().get_across_word_id().is_some() {
            false
        } else {
            let across_relative_moves: Vec<(isize, isize)> = vec![(0, -1), (0, 1)];
            self.neighbouring_cells_empty(location, across_relative_moves)
        }
    }

    fn cell_is_open_down(&self, location: Location) -> bool {
        // If there is already an down word for this cell, can't place another down word here
        if self.cell_map.get(&location).unwrap().get_down_word_id().is_some() {
            false
        } else {
            let down_relative_moves: Vec<(isize, isize)> = vec![(-1, 0), (1, 0)];
            self.neighbouring_cells_empty(location, down_relative_moves)
        }
    }

    fn cell_is_open(&self, location: Location, direction: Direction) -> bool {
        match direction {
            Direction::Across => self.cell_is_open_across(location),
            Direction::Down => self.cell_is_open_down(location),
        }
    }

    fn check_cells_at_ends_free_for_word(&mut self,
                                         location: Location,
                                         word: &Word,
                                         index_in_word: usize,
                                         word_direction: Direction) -> (bool, Location) {
        let mut success: bool;
        let cells_before_root = - (index_in_word as isize);
        let cells_after_root = (word.word_text.len() as isize) - (index_in_word as isize + 1);
        let start_location = location.relative_location_directed(cells_before_root, word_direction);
        let end_location: Location = location.relative_location_directed(cells_after_root, word_direction);
        self.expand_to_fit_cell(start_location.relative_location_directed(-1, word_direction));
        self.expand_to_fit_cell(end_location.relative_location_directed(1, word_direction));

        let before_start = start_location.relative_location_directed(-1, word_direction);
        success = !self.cell_map.get(&before_start).unwrap().contains_letter();
        if !success {
            debug!("Cell before word not empty, failure! {:?}", before_start);
        } else {
            let after_end = end_location.relative_location_directed(1, word_direction);
            success = !self.cell_map.get(&after_end).unwrap().contains_letter();
            if !success {
                debug!("Cell after word not empty, failure! {:?}", after_end);
            }
        }
        (success, start_location)
    }

    fn try_place_letter(&mut self,
                        letter: char,
                        word_id: usize,
                        working_location: &Location,
                        word_direction: Direction) -> bool {
        debug!("Trying to add letter {} to cell location {:?}", letter, working_location);
        let cell = self.cell_map.get_mut(&working_location).unwrap();
        let mut success = cell.add_word(word_id, letter, word_direction);
        debug!("Success adding letter: {}", success);

        let perpendicular_word_id: Option<usize> = match word_direction.rotate() {
            Direction::Across => cell.get_across_word_id(),
            Direction::Down => cell.get_down_word_id(),
        };
        debug!("Cell has perpendicular id {:?}", perpendicular_word_id);

        // Check if the adjacent cell contains a letter but does not share a word_id with
        // the current cell (if we are placing an across word, an adjacent filled cell should
        // share down word id and vice versa).
        if success {
            success = !self.adjacent_word_ids_mismatch(&working_location, -1, word_direction, perpendicular_word_id);
            debug!("Checked adjacent cell empty or matches perpendicular word: {}", success);
        }
        if success {
            success = !self.adjacent_word_ids_mismatch(&working_location, 1, word_direction, perpendicular_word_id);
            debug!("Checked adjacent cell empty or matches perpendicular word: {}", success);
        }

        success
    }

    pub fn try_place_word_in_cell(&mut self,
                                  location: Location,
                                  word_id: usize,
                                  index_in_word: usize,
                                  word_direction: Direction) -> bool {
        self.fill_black_cells();

        let mut success: bool;
        let mut word = self.word_map.get(&word_id).unwrap().clone();
        debug!("Attempting to add word to location: {:?} word_direction: {:?} index: {} word: {:?}",
               location, word_direction, index_in_word, word);
        assert!(!word.is_placed());
        if !word.allowed_in_direction(word_direction) {
            // If the word requires to be placed in the opposite configuration, fail automatically
            success = false;
        } else if self.cell_is_open(location, word_direction) {
            // Check that the spaces at either end of the word are free, and calculate the
            // first cell where we should start placing letters
            let (ends_free, start_location) = self.check_cells_at_ends_free_for_word(location, &word, index_in_word, word_direction);
            success = ends_free;

            let mut updated_locations: Vec<Location> = vec![];

            let mut working_location = start_location.clone();
            for letter in word.word_text.chars() {
                if success {
                    success = self.try_place_letter(letter, word_id, &working_location, word_direction);

                    updated_locations.push(working_location);
                    working_location = working_location.relative_location_directed(1, word_direction);
                }
            }

            // If we have succeeded, update the location. Else, we failed, undo anything we did i.e. remove word from cells
            if success {
                word.update_location(start_location, word_direction);
                self.word_map.insert(word_id, word);
            } else {
                for updated_location in updated_locations {
                    let cell = self.cell_map.get_mut(&updated_location).unwrap();
                    cell.remove_word(word_id);
                }
            }
            self.fit_to_size();
        } else {
            success = false;
        }

        let updated_word = self.word_map.get(&word_id).unwrap().clone();
        if !success {
            assert!(!updated_word.is_placed());
        }
        debug!("After possibly adding {:?}", updated_word);
        success
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::CrosswordGridBuilder;

    #[test]
    fn test_open_cells() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGrid::new_single_word("ALPHA");
        grid.fit_to_size();
        grid.fill_black_cells();
        debug!("{:#?}", grid);

        for i in -1..5 {
            assert!(!grid.cell_is_open_across(Location(0, i)), "Cell (0, {}) should not be open across", i);
            assert!(!grid.cell_is_open_across(Location(-1, i)), "Cell (0, {}) should not be open across", i);
            assert!(!grid.cell_is_open_across(Location(1, i)), "Cell (0, {}) should not be open across", i);
            assert!(!grid.cell_is_open_down(Location(-1, i)), "Cell (0, {}) should not be open down", i);
            assert!(!grid.cell_is_open_down(Location(1, i)), "Cell (0, {}) should not be open down", i);
        }

        for i in 0..4 {
            assert!(grid.cell_is_open_down(Location(0, i)), "Cell (0, {}) should be open down", i);
        }

        assert!(!grid.cell_is_open_down(Location(0, -1)));
        assert!(!grid.cell_is_open_down(Location(0, 5)));

        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
        grid.fit_to_size();
        grid.fill_black_cells();
        debug!("{:#?}", grid);

        assert!(grid.cell_is_open_down(Location(2, 0)));
        assert!(!grid.cell_is_open_down(Location(2, 1)));
        assert!(!grid.cell_is_open_down(Location(2, 2)));
        assert!(grid.cell_is_open_down(Location(2, 3)));

        assert!(!grid.cell_is_open_across(Location(3, 0)));
        assert!(grid.cell_is_open_across(Location(3, 1)));
        assert!(!grid.cell_is_open_across(Location(3, 2)));
        assert!(!grid.cell_is_open_across(Location(3, 3)));
        assert!(grid.cell_is_open_across(Location(3, 5)));
    }

    #[test]
    fn test_fill_black_cells() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGrid::new_single_word("ALPHA");
        debug!("{:#?}", grid);
        grid.fit_to_size();
        debug!("{:#?}", grid);
        grid.fill_black_cells();

        assert_eq!(grid.cell_map.values().filter(|&x| x.is_black()).count(), 2);

        assert!(grid.cell_map.get(&Location(0, -1)).unwrap().is_black());
        assert!(grid.cell_map.get(&Location(0, 5)).unwrap().is_black());

        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
        grid.fit_to_size();
        grid.fill_black_cells();
        assert_eq!(grid.cell_map.values().filter(|&x| x.is_black()).count(), 18);
    }

    #[test]
    fn add_word_to_grid_basic() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGrid::new_single_word("ALPHA");
        grid.fit_to_size();
        grid.fill_black_cells();
        grid.check_valid();

        let arrival_word_id = grid.add_unplaced_word("ARRIVAL", "", None);
        let bear_word_id = grid.add_unplaced_word("BEARER", "", None);
        let innards_word_id = grid.add_unplaced_word("INNARDS", "", None);
        let cup_word_id = grid.add_unplaced_word("CUP", "", None);
        let cap_word_id = grid.add_unplaced_word("CAP", "", None);
        grid.check_valid();
        debug!("{:#?}", grid);

        assert!(grid.try_place_word_in_cell(Location(0, 0), arrival_word_id, 0, Direction::Down));
        grid.check_valid();
        assert!(grid.try_place_word_in_cell(Location(0, 4), bear_word_id, 2, Direction::Down));
        grid.check_valid();
        assert!(grid.try_place_word_in_cell(Location(0, 2), cup_word_id, 2, Direction::Down));
        grid.check_valid();

        let before_failure = grid.to_string();
        assert!(!grid.try_place_word_in_cell(Location(0, 3), innards_word_id, 1, Direction::Down));
        grid.check_valid();
        assert_eq!(before_failure, grid.to_string());

        assert!(!grid.try_place_word_in_cell(Location(-2, 2), cap_word_id, 0, Direction::Across));
        grid.check_valid();
        assert_eq!(before_failure, grid.to_string());
        info!("{}", grid.to_string());

        debug!("{:#?}", grid);
        assert!(grid.try_place_word_in_cell(Location(3, 0), innards_word_id, 0, Direction::Across));
        grid.check_valid();

        let mut from_file = CrosswordGridBuilder::new().from_file("tests/resources/built_up.txt");
        from_file.fit_to_size();
        debug!("{}", grid.to_string());
        assert_eq!(from_file.to_string(), grid.to_string());
    }

    #[test]
    fn add_word_to_grid_adjacent() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/bear_button.txt");
        let button_word_id = grid.add_unplaced_word("BUTTON", "", None);
        grid.check_valid();
        assert!(!grid.try_place_word_in_cell(Location(3, 5), button_word_id, 2, Direction::Across));
    }
}
