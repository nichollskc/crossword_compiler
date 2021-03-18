use log::debug;

use super::CrosswordGrid;
use super::Location;
use super::Direction;

use super::Word;
use super::CrosswordError;

impl CrosswordGrid {
    /// Place the word such that the letter at index index_in_word is placed in the
    /// given location, and the word is in the direction given.
    pub fn place_word_in_cell(&mut self,
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
            self.no_check_place_word_in_cell(location, word_id, index_in_word, word_direction)?;
        }

        match self.check_all_cells_in_word_valid(word_id) {
            Ok(()) => Ok(()),
            Err(error) => {
                self.unplace_word(word_id);
                Err(error)
            },
        }
    }

    /// Place the word such that the letter at index index_in_word is placed in the
    /// given location, and the word is in the direction given. This is a lower-level
    /// function than place_word_in_cell, and performs fewer checks.
    ///
    /// To avoid issues, call fill_black_cells before this function and
    /// check_all_cells_in_word_valid afterwards.
    pub(crate) fn no_check_place_word_in_cell(&mut self,
                                              location: Location,
                                              word_id: usize,
                                              index_in_word: usize,
                                              word_direction: Direction) -> Result<(), CrosswordError> {
        let mut word = self.get_word(word_id)?.clone();

        // Check that the spaces at either end of the word are free, and calculate the
        // first cell where we should start placing letters
        let start_location = self.expand_to_fit_word(location, &word, index_in_word, word_direction);
        let mut result = Ok(());

        let mut working_location = start_location.clone();
        for letter in word.word_text.chars() {
            if result.is_ok() {
                result = self.place_letter(letter, word_id, &working_location, word_direction);
                working_location = working_location.relative_location_directed(1, word_direction);
            }
        }

        // If we have succeeded, update the location. Else, we failed, undo anything we did i.e. remove word from cells
        if result.is_ok() {
            word.update_location(start_location, word_direction);
            self.word_map.insert(word_id, word);
        } else {
            self.unplace_word(word_id);
        }
        result
    }

    fn expand_to_fit_word(&mut self,
                          location: Location,
                          word: &Word,
                          index_in_word: usize,
                          word_direction: Direction) -> Location {
        let cells_before_root = - (index_in_word as isize);
        let start_location = location.relative_location_directed(cells_before_root, word_direction);
        let before_start = start_location.relative_location_directed(-1, word_direction);
        self.expand_to_fit_cell(before_start);

        let cells_after_root = (word.word_text.len() as isize) - (index_in_word as isize + 1);
        let end_location: Location = location.relative_location_directed(cells_after_root, word_direction);
        let after_end = end_location.relative_location_directed(1, word_direction);
        self.expand_to_fit_cell(after_end);

        start_location
    }

    fn place_letter(&mut self,
                    letter: char,
                    word_id: usize,
                    working_location: &Location,
                    word_direction: Direction) -> Result<(), CrosswordError> {
        debug!("Trying to add letter {} to cell location {:?}", letter, working_location);
        let cell = self.get_cell_mut(&working_location)?;
        let result = cell.add_word(word_id, letter, word_direction);
        debug!("Success adding letter: {:?}", result);
        match result {
            Ok(()) => Ok(()),
            Err(cell_error) => Err(CrosswordError::CellError(*working_location, cell_error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::CrosswordGridBuilder;
    use super::super::CellError;

    #[test]
    fn test_add_word_to_grid_basic() -> Result<(), CrosswordError> {
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

        grid.place_word_in_cell(Location(0, 0), arrival_word_id, 0, Direction::Down)?;
        grid.check_valid();
        grid.place_word_in_cell(Location(0, 4), bear_word_id, 2, Direction::Down)?;
        grid.check_valid();
        grid.place_word_in_cell(Location(0, 2), cup_word_id, 2, Direction::Down)?;
        grid.check_valid();

        let before_failure = grid.to_string();
        let result: Result<(), CrosswordError> = Err(CrosswordError::CellError(Location(0, 3),
                                                                               CellError::LetterMismatch('H', 'N')));
        assert_eq!(grid.place_word_in_cell(Location(0, 3), innards_word_id, 1, Direction::Down),
                   result);
        grid.check_valid();
        assert_eq!(before_failure, grid.to_string());

        let result: Result<(), CrosswordError> = Err(CrosswordError::CellError(Location(-2, 4),
                                                                               CellError::LetterMismatch('B', 'P')));
        assert_eq!(grid.place_word_in_cell(Location(-2, 2), cap_word_id, 0, Direction::Across),
                   result);
        grid.check_valid();
        assert_eq!(before_failure, grid.to_string());
        debug!("{}", grid.to_string());

        debug!("{:#?}", grid);
        grid.place_word_in_cell(Location(3, 0), innards_word_id, 0, Direction::Across)?;
        grid.check_valid();

        let mut from_file = CrosswordGridBuilder::new().from_file("tests/resources/built_up.txt");
        from_file.fit_to_size();
        debug!("{}", grid.to_string());
        assert_eq!(from_file.to_string(), grid.to_string());

        Ok(())
    }

    #[test]
    fn test_add_word_to_grid_adjacent() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/bear_button.txt");
        let button_word_id = grid.add_unplaced_word("BUTTON", "", None);
        grid.check_valid();
        let before_failure = grid.to_string();
        let actual_result = grid.place_word_in_cell(Location(3, 5), button_word_id, 2, Direction::Across);
        println!("{}", grid.to_string_with_coords());
        assert_matches!(actual_result,
                        Err(CrosswordError::AdjacentCellsNoLinkWord(Location(3, 3),
                                                                    Location(2, 3))));

        assert_eq!(before_failure, grid.to_string());
    }
}
