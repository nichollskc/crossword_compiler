use log::{info,warn,debug,error};
use std::collections::HashSet;

use super::CrosswordGrid;
use super::Location;

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
            if let Some((start_location, end_location, across)) = word.get_location() {
                let mut black_cells: Vec<Location> = vec![];
                black_cells.push(start_location.relative_location_directed(-1, across));
                black_cells.push(end_location.relative_location_directed(1, across));

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

    /// Check if the neighbouring cell has an across word (or down if across=false)
    /// I.e. look for the cell parallel to this one in the direction indicated
    ///     (e.g. move of (-1, 0) - the cell above if we are thinking of adding an across word
    ///           corresponds to move_by=-1, across=true)
    /// and check if it has an across word id (or down if across=false)
    fn get_adjacent_word_id(&self, location: &Location, move_by: isize, across: bool) -> Option<usize> {
        let neighbour_location = location.relative_location_directed(move_by, !across);
        let cell = self.cell_map.get(&neighbour_location).unwrap();
        if across {
            cell.get_across_word_id()
        } else {
            cell.get_down_word_id()
        }
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

    fn cell_is_open(&self, location: Location, across: bool) -> bool {
        if across {
            self.cell_is_open_across(location)
        } else {
            self.cell_is_open_down(location)
        }
    }

    pub fn try_place_word_in_cell(&mut self,
                                  location: Location,
                                  word_id: usize,
                                  index_in_word: usize,
                                  across: bool) -> bool {
        self.fit_to_size();
        self.fill_black_cells();

        let mut success = true;
        let mut start_location = location;
        let word = self.word_map.get(&word_id).unwrap().clone();
        info!("Attempting to add word to location: {:?} across: {} index: {} word: {:?}",
               location, across, index_in_word, word);
        assert!(!word.is_placed());
        if self.cell_is_open(location, across) {
            let cells_before_root = - (index_in_word as isize);
            let cells_after_root = (word.word_text.len() as isize) - (index_in_word as isize + 1);
            start_location = location.relative_location_directed(cells_before_root, across);
            let end_location: Location = location.relative_location_directed(cells_after_root, across);
            self.expand_to_fit_cell(start_location.relative_location_directed(-1, across));
            self.expand_to_fit_cell(end_location.relative_location_directed(1, across));

            let mut updated_locations: Vec<Location> = vec![];
            let mut adjacent_words: HashSet<usize> = HashSet::new();

            let before_start = start_location.relative_location_directed(-1, across);
            success = !self.cell_map.get(&before_start).unwrap().contains_letter();
            if !success {
                debug!("Cell before word not empty, failure! {:?}", before_start);
            } else {
                let after_end = end_location.relative_location_directed(1, across);
                success = !self.cell_map.get(&after_end).unwrap().contains_letter();
                if !success {
                    debug!("Cell after word not empty, failure! {:?}", after_end);
                }
            }

            let mut working_location = start_location.clone();
            for letter in word.word_text.chars() {
                if success {
                    debug!("Trying to add letter {} to cell location {:?}", letter, working_location);
                    // (1) Check we don't border a parallel word with more than 1 cell
                    // Fail if we are adjacent to a word we have already found ourselves
                    // adjacent to - keep track of which we've already visited using HashSet
                    if let Some(neighbour_id) = self.get_adjacent_word_id(&working_location,
                                                                          -1,
                                                                          across) {
                        success = adjacent_words.insert(neighbour_id);
                        debug!("Checking neighbouring word {}, ok: {}", neighbour_id, success);
                    }
                }
                if success {
                    if let Some(neighbour_id) = self.get_adjacent_word_id(&working_location,
                                                                          1,
                                                                          across) {
                        success = adjacent_words.insert(neighbour_id);
                        debug!("Checking neighbouring word {}, ok: {}", neighbour_id, success);
                    }
                }

                // (2) Check cell empty or matches letter
                if success {
                    let cell = self.cell_map.get_mut(&working_location).unwrap();
                    success = cell.add_word(word_id, letter, across);
                    updated_locations.push(working_location);
                    working_location = working_location.relative_location_directed(1, across);
                }
            }

            // If we have failed, undo anything we did i.e. remove word from cells
            if !success {
                for updated_location in updated_locations {
                    let cell = self.cell_map.get_mut(&updated_location).unwrap();
                    cell.remove_word(word_id);
                }
            }
        } else {
            success = false;
        }

        if success {
            self.word_map.insert(word_id, Word::new(&word.word_text, start_location, across));
        } else {
            assert!(!word.is_placed());
        }
        let word = self.word_map.get(&word_id).unwrap().clone();
        debug!("After possibly adding {:?}", word);
        self.fit_to_size();
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
    fn add_word_to_grid() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGrid::new_single_word("ALPHA");
        grid.fit_to_size();
        grid.fill_black_cells();
        grid.check_valid();

        let arrival_word_id = grid.add_unplaced_word("ARRIVAL");
        let bear_word_id = grid.add_unplaced_word("BEARER");
        let innards_word_id = grid.add_unplaced_word("INNARDS");
        let cup_word_id = grid.add_unplaced_word("CUP");
        let cap_word_id = grid.add_unplaced_word("CAP");
        grid.check_valid();
        debug!("{:#?}", grid);

        assert!(grid.try_place_word_in_cell(Location(0, 0), arrival_word_id, 0, false));
        grid.check_valid();
        assert!(grid.try_place_word_in_cell(Location(0, 4), bear_word_id, 2, false));
        grid.check_valid();
        assert!(grid.try_place_word_in_cell(Location(0, 2), cup_word_id, 2, false));
        grid.check_valid();

        let before_failure = grid.to_string();
        assert!(!grid.try_place_word_in_cell(Location(0, 3), innards_word_id, 1, false));
        grid.check_valid();
        assert_eq!(before_failure, grid.to_string());

        assert!(!grid.try_place_word_in_cell(Location(-2, 2), cap_word_id, 0, true));
        grid.check_valid();
        assert_eq!(before_failure, grid.to_string());
        info!("{}", grid.to_string());

        debug!("{:#?}", grid);
        assert!(grid.try_place_word_in_cell(Location(3, 0), innards_word_id, 0, true));
        grid.check_valid();

        let mut from_file = CrosswordGridBuilder::new().from_file("tests/resources/built_up.txt");
        from_file.fit_to_size();
        debug!("{}", grid.to_string());
        assert_eq!(from_file.to_string(), grid.to_string());

        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/bear_button.txt");
        let button_word_id = grid.add_unplaced_word("BUTTON");
        grid.check_valid();
        assert!(!grid.try_place_word_in_cell(Location(3, 5), button_word_id, 2, true));
    }
}
