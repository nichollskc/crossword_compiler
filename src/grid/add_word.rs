use log::{info,trace,debug};
use std::collections::HashSet;
use std::iter::FromIterator;

use super::CrosswordGrid;
use super::Location;
use super::Direction;

use super::Word;
use super::CrosswordError;

impl CrosswordGrid {
    fn get_expected_black_cells(&self) -> Vec<Location> {
        let mut black_cells: Vec<Location> = vec![];
        for word in self.word_map.values() {
            if let Some((start_location, end_location, direction)) = word.get_location() {
                black_cells.push(start_location.relative_location_directed(-1, direction));
                black_cells.push(end_location.relative_location_directed(1, direction));
            }
        }
        black_cells
    }

    pub fn black_cells_valid(&self) -> bool {
        let black_cells_set: HashSet<Location> = HashSet::from_iter(self.get_expected_black_cells().iter().cloned());
        let mut valid = true;

        for (location, cell) in self.cell_map.iter() {
            if cell.is_black() && !black_cells_set.contains(location) {
                valid = false;
            }
        }

        for location in black_cells_set {
            if !self.cell_map.get(&location).unwrap().is_black() {
                valid = false;
            }
        }
        valid
    }

    pub fn fill_black_cells(&mut self) {
        // Clear black cells before starting
        for (_location, cell) in self.cell_map.iter_mut() {
            if cell.is_black() {
                cell.set_empty();
            }
        }

        let black_cells = self.get_expected_black_cells();
        for cell_location in black_cells {
            if let Some(cell) = self.cell_map.get_mut(&cell_location) {
                cell.set_black();
            } else {
                panic!("Cell doesn't exist! {:#?}\n{:#?}", cell_location, self);
            }
        }
    }

    fn get_word_id(&self, location: &Location, word_direction: Direction) -> Option<usize> {
        let cell = self.cell_map.get(&location).unwrap();
        debug!("Looking at adjacent cell {:?}", cell);
        match word_direction {
            Direction::Across => cell.get_across_word_id(),
            Direction::Down => cell.get_down_word_id(),
        }
    }

    // Checks whether two adjacent cells are compatible i.e. if direction is across
    // then checks the horizontally adjacent neighbour is valid i.e. if both this cell and its
    // neighbour are non-empty, are they part of the same across word?
    //
    // Can return NodeNotFound (probably worth a warn, but technically the nodes are compatible,
    // so we return OK here)
    // AdjacentCellsNoLinkWord is an error that can slip through the matrix checker method
    //      (there is no simple matrix-based check for this!) but is unacceptable.
    // AdjacentCellsMismatchedLinkWord is also unacceptable, and should have been avoided by the
    //      matrix checker
    fn check_adjacent_cells_compatible(&self, location: &Location, move_by: isize, direction: Direction) -> Result<(), CrosswordError> {
        let neighbour_location = location.relative_location_directed(move_by, direction);

        // Fetch the cells. This can only fail if the locations are invalid, in which case we'll
        // get a NodeNotFound error. The caller can decide if this is an issue or not.
        // If either node doesn't exist, they are trivially compatible.
        let cell = self.get_cell(location)?;
        let neighbour = self.get_cell(&neighbour_location)?;
        if cell.contains_letter() && neighbour.contains_letter() {
            let cell_word = cell.get_word_id(direction);
            let neighbour_word = neighbour.get_word_id(direction);
            // Three ways to fail - either one of the cells has no across [down] word_id
            // or they do both have an across [down] word_id but it's different
            if cell_word.is_none() || neighbour_word.is_none() {
                Err(CrosswordError::AdjacentCellsNoLinkWord(*location,
                                                            neighbour_location))
            } else if cell_word != neighbour_word {
                // This should have been caught by adding black cells at the end/start of each word
                Err(CrosswordError::AdjacentCellsMismatchedLinkWord(*location,
                                                                    neighbour_location,
                                                                    cell_word.expect("Checked not none previously"),
                                                                    neighbour_word.expect("Checked not none previously")))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn neighbouring_cells_empty(&self, location: Location, neighbour_moves: Vec<(isize, isize)>) -> bool {
        let mut result = false;
        for relative_move in neighbour_moves {
            if self.cell_map.get(&location.relative_location(relative_move.0, relative_move.1)).unwrap().is_empty() {
                result = true;
            }
        }
        result
    }

    fn check_cells_at_ends_free_for_word(&mut self,
                                         location: Location,
                                         word: &Word,
                                         index_in_word: usize,
                                         word_direction: Direction) -> Result<Location, CrosswordError> {
        let cells_before_root = - (index_in_word as isize);
        let start_location = location.relative_location_directed(cells_before_root, word_direction);
        let before_start = start_location.relative_location_directed(-1, word_direction);
        self.expand_to_fit_cell(before_start);
        if self.get_cell(&before_start)?.contains_letter() {
            Err(CrosswordError::NonEmptyWordBoundary(before_start, start_location))?;
        }

        let cells_after_root = (word.word_text.len() as isize) - (index_in_word as isize + 1);
        let end_location: Location = location.relative_location_directed(cells_after_root, word_direction);
        let after_end = end_location.relative_location_directed(1, word_direction);
        self.expand_to_fit_cell(after_end);
        if self.get_cell(&after_end)?.contains_letter() {
            Err(CrosswordError::NonEmptyWordBoundary(after_end, end_location))?;
        }

        Ok(start_location)
    }


    fn place_letter(&mut self,
                    letter: char,
                    word_id: usize,
                    working_location: &Location,
                    word_direction: Direction) -> Result<(), CrosswordError> {
        debug!("Trying to add letter {} to cell location {:?}", letter, working_location);
        let mut cell = self.get_cell_mut(&working_location)?;
        let result = cell.add_word(word_id, letter, word_direction);
        debug!("Success adding letter: {:?}", result);
        result
    }

    fn try_place_letter(&mut self,
                        letter: char,
                        word_id: usize,
                        working_location: &Location,
                        word_direction: Direction) -> Result<(), CrosswordError> {
        // Check if the adjacent cell contains a letter but does not share a word_id with
        // the current cell (if we are placing an across word, an adjacent filled cell should
        // share down word id and vice versa).
        self.check_adjacent_cells_compatible(&working_location, -1, word_direction.rotate())?;
        self.check_adjacent_cells_compatible(&working_location, 1, word_direction.rotate())?;

        Ok(())
    }

    pub fn try_place_word_in_cell(&mut self,
                                  location: Location,
                                  word_id: usize,
                                  index_in_word: usize,
                                  word_direction: Direction) -> Result<(), CrosswordError> {
        self.fill_black_cells();

        let word = self.get_word(word_id)?;
        debug!("Attempting to add word to location: {:?} word_direction: {:?} index: {} word: {:?}",
               location, word_direction, index_in_word, word);
        if let Some(existing_location) = word.get_location() {
            Err(CrosswordError::WordAlreadyPlaced(word_id,
                                                  word.word_text.clone(),
                                                  existing_location.0))?;
        }

        if !word.allowed_in_direction(word_direction) {
            // If the word requires to be placed in the opposite configuration, fail automatically
            debug!("Failed since direction {:?} is not what word requires: {:?}", word_direction, word);
            Err(CrosswordError::InvalidWordDirection(word_id,
                                                     word.word_text.clone(),
                                                     word_direction))?;
        } else {
            self.place_word_in_cell(location, word_id, index_in_word, word_direction)?;
        }

        let updated_word = self.get_word(word_id)?.clone();
        Ok(())
    }

    pub fn place_word_in_cell(&mut self,
                              location: Location,
                              word_id: usize,
                              index_in_word: usize,
                              word_direction: Direction) -> Result<(), CrosswordError> {
        let mut word = self.get_word(word_id)?.clone();

        // Check that the spaces at either end of the word are free, and calculate the
        // first cell where we should start placing letters
        let start_location = self.check_cells_at_ends_free_for_word(location, &word, index_in_word, word_direction)?;
        let mut result = Ok(());

        let mut updated_locations: Vec<Location> = vec![];

        let mut working_location = start_location.clone();
        for letter in word.word_text.chars() {
            if result.is_ok() {
                result = self.place_letter(letter, word_id, &working_location, word_direction);

                updated_locations.push(working_location);
                working_location = working_location.relative_location_directed(1, word_direction);
            }
        }

        // If we have succeeded, update the location. Else, we failed, undo anything we did i.e. remove word from cells
        if result.is_ok() {
            word.update_location(start_location, word_direction);
            self.word_map.insert(word_id, word);
        } else {
            for updated_location in updated_locations {
                let cell = self.cell_map.get_mut(&updated_location).unwrap();
                cell.remove_word(word_id);
            }
            self.fit_to_size();
        }
        result
    }

    fn check_adjacent_cell_matches(&self,
                                   location: &Location,
                                   move_by: isize,
                                   direction: Direction) -> Result<(), CrosswordError> {
        let cell = self.get_cell(location).unwrap();
        if cell.contains_letter() {
            self.check_adjacent_cells_compatible(location, move_by, direction)
        } else {
            Ok(())
        }
    }

    pub fn check_word_placement_valid(&self) -> Result<(), CrosswordError> {
        info!("Checking word placement valid for grid\n{}", self.to_string());
        // Each cell with a word_id should only be adjacent to another cell with
        // a word_id if the IDs match
        for location in self.cell_map.keys() {
            debug!("Checking location {:?}", location);
            self.check_adjacent_cell_matches(location, -1, Direction::Across)?;
            self.check_adjacent_cell_matches(location,  1, Direction::Across)?;
            self.check_adjacent_cell_matches(location, -1, Direction::Down)?;
            self.check_adjacent_cell_matches(location,  1, Direction::Down)?;
        }
        Ok(())
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

        for i in -1..6 {
            assert!(!grid.cell_is_open(Location(-1, i), Direction::Across, false), "Cell (-1, {}) should not be open across", i);
            assert!(!grid.cell_is_open(Location(-1, i), Direction::Down, false), "Cell (-1, {}) should not be open down", i);
            assert!(!grid.cell_is_open(Location(1, i), Direction::Across, false), "Cell (1, {}) should not be open across", i);
            assert!(!grid.cell_is_open(Location(1, i), Direction::Down, false), "Cell (1, {}) should not be open down", i);
            assert!(!grid.cell_is_open(Location(0, i), Direction::Across, false), "Cell (0, {}) should not be open across", i);
        }

        for i in 0..5 {
            assert!(grid.cell_is_open_down(Location(0, i)), "Cell (0, {}) should be open down", i);
        }

        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
        grid.fit_to_size();
        grid.fill_black_cells();
        debug!("{:#?}", grid);

        assert!(grid.cell_is_open_down(Location(2, 0)));
        assert!(!grid.cell_is_open_down(Location(2, 1)));
        assert!(!grid.cell_is_open_down(Location(2, 2)));
        assert!(grid.cell_is_open_down(Location(2, 3)));

        assert!(!grid.cell_is_open(Location(3, 0), Direction::Across, false));
        assert!(grid.cell_is_open(Location(3, 1), Direction::Across, false));
        assert!(!grid.cell_is_open(Location(3, 2), Direction::Across, false));
        assert!(!grid.cell_is_open(Location(3, 3), Direction::Across, false));
        assert!(grid.cell_is_open(Location(3, 5), Direction::Across, false));
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

        assert!(grid.try_place_word_in_cell_connected(Location(0, 0), arrival_word_id, 0, Direction::Down));
        grid.check_valid();
        assert!(grid.try_place_word_in_cell_connected(Location(0, 4), bear_word_id, 2, Direction::Down));
        grid.check_valid();
        assert!(grid.try_place_word_in_cell_connected(Location(0, 2), cup_word_id, 2, Direction::Down));
        grid.check_valid();

        let before_failure = grid.to_string();
        assert!(!grid.try_place_word_in_cell_connected(Location(0, 3), innards_word_id, 1, Direction::Down));
        grid.check_valid();
        assert_eq!(before_failure, grid.to_string());

        assert!(!grid.try_place_word_in_cell_connected(Location(-2, 2), cap_word_id, 0, Direction::Across));
        grid.check_valid();
        assert_eq!(before_failure, grid.to_string());
        debug!("{}", grid.to_string());

        debug!("{:#?}", grid);
        assert!(grid.try_place_word_in_cell_connected(Location(3, 0), innards_word_id, 0, Direction::Across));
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
        assert!(!grid.try_place_word_in_cell_connected(Location(3, 5), button_word_id, 2, Direction::Across));
    }
}
