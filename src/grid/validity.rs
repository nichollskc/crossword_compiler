use log::{info,trace,debug};
use std::collections::HashSet;
use std::iter::FromIterator;

use super::CrosswordGrid;
use super::Location;
use super::Direction;

use super::CrosswordError;

impl CrosswordGrid {
    /// Check that all word placements are valid i.e. that there are no adjacent cells
    /// which don't belong to the same word and that at the start and end of each word
    /// there is an empty cell.
    pub fn check_all_word_placement_valid(&self) -> Result<(), CrosswordError> {
        info!("Checking word placement valid for grid\n{}", self.to_string());
        // Each cell with a word_id should only be adjacent to another cell with
        // a word_id if the IDs match
        for location in self.cell_map.keys() {
            debug!("Checking location {:?}", location);
            self.check_all_neighbours_compatible(location)?;
        }
        Ok(())
    }

    /// Check that the placement of a word is valid i.e. that the cells before and after the word
    /// are empty and that there are no cells adjacent to the word which do not share a word ID.
    pub fn check_all_cells_in_word_valid(&self, word_id: usize) -> Result<(), CrosswordError> {
        let word = self.get_word(word_id)?;

        if let Some((start_location, end_location, direction)) = word.get_location() {
            let before_start = start_location.relative_location_directed(-1, direction);
            if self.get_cell(&before_start)?.contains_letter() {
                Err(CrosswordError::NonEmptyWordBoundary(before_start, start_location))?;
            }
            let after_end = end_location.relative_location_directed(1, direction);
            if self.get_cell(&after_end)?.contains_letter() {
                Err(CrosswordError::NonEmptyWordBoundary(after_end, end_location))?;
            }

            let mut working_location = start_location.clone();
            for _i in 0..word.len() {
                self.check_all_neighbours_compatible(&working_location)?;
                working_location = working_location.relative_location_directed(1, direction);
            }
        }
        Ok(())
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
        trace!("Checking whether cell at {:?} is compatible with cell at {:?}", location, neighbour_location);

        if cell.contains_letter() && neighbour.contains_letter() {
            let cell_word = cell.get_word_id(direction);
            let neighbour_word = neighbour.get_word_id(direction);
            debug!("Both cells contain a letter. This cell is in word {:?}, neighbour is in word {:?}", cell_word, neighbour_word);
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

    fn check_all_neighbours_compatible(&self, location: &Location) -> Result<(), CrosswordError> {
        let cell = self.get_cell(location).unwrap();
        if cell.contains_letter() {
            self.check_adjacent_cells_compatible(location, -1, Direction::Across)?;
            self.check_adjacent_cells_compatible(location,  1, Direction::Across)?;
            self.check_adjacent_cells_compatible(location, -1, Direction::Down)?;
            self.check_adjacent_cells_compatible(location,  1, Direction::Down)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::CrosswordGridBuilder;
    use super::super::CellError;

    #[test]
    fn test_valid_word_placement() -> Result<(), CrosswordError> {
        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/bear_button.txt");
        let bat_id = grid.add_unplaced_word("BAT", "", None);
        println!("{}", grid.to_string_with_coords());
        let result: Result<(), CrosswordError> = Err(CrosswordError::CellError(Location(2, 3),
                                                                               CellError::FillBlack));
        assert_matches!(grid.no_check_place_word_in_cell(Location(2, 5),
                                                         bat_id,
                                                         1,
                                                         Direction::Across),
                        result);
        grid.get_cell_mut(&Location(2, 4))?.set_empty();
        println!("Cell {:?}", grid.get_cell(&Location(2, 4)).unwrap());
        grid.no_check_place_word_in_cell(Location(2, 5),
                                         bat_id,
                                         1,
                                         Direction::Across);
        println!("{}", grid.to_string_with_coords());
        assert_matches!(grid.check_adjacent_cells_compatible(&Location(2, 4), -1, Direction::Across),
                        Err(CrosswordError::AdjacentCellsMismatchedLinkWord(Location(2, 4),
                                                                            Location(2, 3),
                                                                            bat_id,
                                                                            _)));

        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/bear_button.txt");
        let bet_id = grid.add_unplaced_word("BET", "", None);
        grid.no_check_place_word_in_cell(Location(0, 6),
                                         bet_id,
                                         1,
                                         Direction::Down);
        println!("{}", grid.to_string_with_coords());
        assert_eq!(Err(CrosswordError::AdjacentCellsNoLinkWord(Location(1, 6),
                                                               Location(1, 5))),
                   grid.check_all_cells_in_word_valid(bet_id));
        Ok(())
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
}
