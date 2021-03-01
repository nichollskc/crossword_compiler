use log::{info,warn,debug,error};
use std::collections::HashMap;

use super::CrosswordGrid;
use super::Location;

use super::Word;
use super::VALIDCHARS;

#[derive(Debug,Clone)]
struct PlacementAttempt {
    word_id: usize,
    index_in_word: usize,
    location: Location,
    across: bool,
}

struct PlacementAttemptIterator {
    words: Vec<(usize, Word)>,
    current_word: Word,
    current_word_id: usize,
    current_index_in_word: usize,
    current_attempt: Option<PlacementAttempt>,
    remaining_locations: Vec<(Location, bool)>,
    letter_to_locations: HashMap<char, Vec<(Location, bool)>>,
}

impl PlacementAttemptIterator {
    fn new(grid: &CrosswordGrid) -> Self {
        let empty_word = Word::new_unplaced("");

        let mut letter_to_locations: HashMap<char, Vec<(Location, bool)>> = HashMap::new();
        for c in VALIDCHARS.chars() {
            letter_to_locations.insert(c, vec![]);
        }

        for (location, cell) in grid.cell_map.iter().filter(|(_l, c)| c.contains_letter() && !c.is_intersection()) {
            // All these cells should belong to precisely one word
            let letter = cell.to_char();
            let across = match cell.get_across_word_id() {
                Some(_w) => false,
                None => true,
            };
            letter_to_locations.get_mut(&letter).unwrap().push((*location, across));
        }

        let copied_words = grid.word_map.iter()
            .map(|(key, value)| (*key, value.clone()))
            .filter(|(key, value)| !value.is_placed())
            .collect();

        PlacementAttemptIterator {
             words: copied_words,
             current_word_id: 0,
             current_word: empty_word,
             current_index_in_word: 0,
             current_attempt: None,
             remaining_locations: vec![],
             letter_to_locations,
        }
    }

    fn get_all_locations_current(&mut self) {
        self.remaining_locations = self.letter_to_locations
            .get(&self.current_word.get_char_at_index(self.current_index_in_word))
            .unwrap()
            .to_vec();
    }

    fn move_to_next_index_in_word(&mut self) -> bool {
        let mut keep_going = true;
        self.current_index_in_word += 1;
        if self.current_index_in_word < self.current_word.len() {
            // This is a valid index
            self.get_all_locations_current();
            keep_going = true;
        } else {
            // Invalid index (end of word), move to next word
            keep_going = self.move_to_next_word();
        }
        keep_going
    }

    fn move_to_next_word(&mut self) -> bool {
        let mut keep_going = true;
        if let Some((word_id, word)) = self.words.pop() {
            self.current_word = word;
            self.current_word_id = word_id;
            self.current_index_in_word = 0;
            self.get_all_locations_current();
            keep_going = true;
        } else {
            keep_going = false;
        }
        keep_going
    }

    fn move_to_next_location(&mut self) -> bool {
        let next_location = self.remaining_locations.pop();
        let mut keep_going = true;
        if let Some((location, across)) = next_location {
            let attempt = PlacementAttempt {
                word_id: self.current_word_id,
                index_in_word: self.current_index_in_word,
                location,
                across,
            };
            self.current_attempt = Some(attempt);
            keep_going = false;
        } else {
            // We need to move to the next index_in_word
            keep_going = self.move_to_next_index_in_word();
        }
        keep_going
    }
}

impl Iterator for PlacementAttemptIterator {
    type Item = PlacementAttempt;

    /// Iterates through PlacementAttempts in the following order:
    ///
    /// For each word
    /// For each letter in the word
    /// For each location matching this letter (which will have a single valid direction)
    ///
    /// This should be initialised with a map giving the possible locations to try for each letter
    fn next(&mut self) -> Option<PlacementAttempt> {
        let mut keep_going = true;
        self.current_attempt = None;

        // Try removing current location from the list for this word and index_in_word
        while self.current_attempt.is_none() && keep_going {
            keep_going = self.move_to_next_location();
        }

        self.current_attempt.clone()
    }
}

impl CrosswordGrid {
    pub fn place_random_word(&mut self) -> bool {
        let mut success = false;
        let mut keep_going = true;
        let mut attempt_iterator = PlacementAttemptIterator::new(&self);
        while !success && keep_going {
            if let Some(attempt) = attempt_iterator.next() {
                success = self.try_place_word_in_cell(attempt.location,
                                                      attempt.word_id,
                                                      attempt.index_in_word,
                                                      attempt.across);
            } else {
                // Out of possible placements to try!
                keep_going = false;
            }
        }

        success
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::CrosswordGridBuilder;

    #[test]
    fn test_simple_iterator() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGrid::new_single_word("ALPHA");
        let mut attempts_expected = 0;
        assert_eq!(PlacementAttemptIterator::new(&grid).count(), attempts_expected);

        grid.add_unplaced_word("MOP");
        attempts_expected += 1;
        assert_eq!(PlacementAttemptIterator::new(&grid).count(), attempts_expected);

        grid.add_unplaced_word("LOOP");
        attempts_expected += 2;
        assert_eq!(PlacementAttemptIterator::new(&grid).count(), attempts_expected);

        grid.add_unplaced_word("HARICOT");
        attempts_expected += 3;
        assert_eq!(PlacementAttemptIterator::new(&grid).count(), attempts_expected);

        grid.add_unplaced_word("LOLLIPOP");
        attempts_expected += 3 + 2;
        assert_eq!(PlacementAttemptIterator::new(&grid).count(), attempts_expected);

        grid.add_unplaced_word("ABACUS");
        attempts_expected += 4;
        assert_eq!(PlacementAttemptIterator::new(&grid).count(), attempts_expected);
    }

    #[test]
    fn test_iterator() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
        grid.add_unplaced_word("ABACUS");
        assert_eq!(PlacementAttemptIterator::new(&grid).count(), 9*2 + 1);
        grid.add_unplaced_word("LOOP");
        assert_eq!(PlacementAttemptIterator::new(&grid).count(), 9*2 + 1 + 4*2 + 1);
    }

    #[test]
    fn test_use_attempts() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
        grid.add_unplaced_word("ABACUS");
        grid.add_unplaced_word("LOOP");
        grid.add_unplaced_word("BEE");
        let mut num_successes = 0;

        for attempt in PlacementAttemptIterator::new(&grid) {
            info!("Trying attempt {:?}", attempt);
            let mut grid_clone = grid.clone();
            let success = grid_clone.try_place_word_in_cell(attempt.location,
                                                            attempt.word_id,
                                                            attempt.index_in_word,
                                                            attempt.across);
            info!("Success for attempt {:?}: {}", attempt, success);
            if success {
                info!("Resulting grid\n{}", grid_clone.to_string());
                num_successes += 1;
            } else {
                assert_eq!(grid_clone.to_string(), grid.to_string());
            }
        }
        assert_eq!(num_successes, 5);
    }
}
