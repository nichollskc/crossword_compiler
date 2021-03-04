use log::debug;
use std::collections::HashMap;

use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;

use super::CrosswordGrid;
use super::Location;
use super::Direction;

use super::Word;
use super::VALIDCHARS;

#[derive(Debug,Clone)]
struct PlacementAttempt {
    word_id: usize,
    index_in_word: usize,
    location: Location,
    direction: Direction,
}

struct PlacementAttemptIterator {
    words: Vec<(usize, Word)>,
    current_word: Word,
    current_word_id: usize,
    current_index_in_word: usize,
    current_attempt: Option<PlacementAttempt>,
    remaining_locations: Vec<(Location, Direction)>,
    letter_to_locations: HashMap<char, Vec<(Location, Direction)>>,
}

impl PlacementAttemptIterator {
    fn new(grid: &CrosswordGrid, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let empty_word = Word::new_unplaced("");

        let mut letter_to_locations: HashMap<char, Vec<(Location, Direction)>> = HashMap::new();
        for c in VALIDCHARS.chars() {
            letter_to_locations.insert(c, vec![]);
        }

        for (location, cell) in grid.cell_map.iter().filter(|(_l, c)| c.contains_letter() && !c.is_intersection()) {
            // All these cells should belong to precisely one word
            let letter = cell.to_char();
            let empty_direction = match cell.get_across_word_id() {
                Some(_w) => Direction::Down,
                None => Direction::Across,
            };
            letter_to_locations.get_mut(&letter).unwrap().push((*location, empty_direction));
        }

        for c in VALIDCHARS.chars() {
            letter_to_locations.get_mut(&c).unwrap().sort_by_key(|a| (a.1, a.0.0, a.0.1));
            letter_to_locations.get_mut(&c).unwrap().shuffle(&mut rng);
        }

        let mut copied_words: Vec<(usize, Word)> = grid.word_map.iter()
            .map(|(key, value)| (*key, value.clone()))
            .filter(|(_key, value)| !value.is_placed())
            .collect();
        // Determinstically shuffle the word list. The order is currently
        // arbitrary, so first sort by word_id and then shuffle using the seeded RNG
        copied_words.sort_by(|a, b| a.0.cmp(&b.0));
        copied_words.shuffle(&mut rng);

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
        let keep_going: bool;
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
        let keep_going: bool;
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
        let keep_going: bool;
        if let Some((location, direction)) = next_location {
            let attempt = PlacementAttempt {
                word_id: self.current_word_id,
                index_in_word: self.current_index_in_word,
                location,
                direction,
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
    pub fn place_random_word(&mut self, seed: u64) -> bool {
        let mut success = false;
        let mut keep_going = true;
        let mut attempt_iterator = PlacementAttemptIterator::new(&self, seed);
        while !success && keep_going {
            if let Some(attempt) = attempt_iterator.next() {
                success = self.try_place_word_in_cell(attempt.location,
                                                      attempt.word_id,
                                                      attempt.index_in_word,
                                                      attempt.direction);
            } else {
                // Out of possible placements to try!
                keep_going = false;
            }
        }

        success
    }

    pub fn remove_random_leaves(&mut self, num_leaves: usize, seed: u64) {
        let mut leaves: Vec<usize> = self.to_graph().find_leaves();
        let mut rng = StdRng::seed_from_u64(seed);
        leaves.sort();
        leaves.shuffle(&mut rng);

        debug!("Attempting to remove {} leaves", num_leaves);

        let mut count: usize = 0;
        while count < num_leaves && self.count_placed_words() > 1 {
            if let Some(word_id) = leaves.pop() {
                debug!("Removing leaf word {}", word_id);
                self.unplace_word(word_id);
            }
            count += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::CrosswordGridBuilder;
    use log::info;

    #[test]
    fn test_simple_iterator() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGrid::new_single_word("ALPHA");
        let mut attempts_expected = 0;
        assert_eq!(PlacementAttemptIterator::new(&grid, 13).count(), attempts_expected);

        grid.add_unplaced_word("MOP");
        attempts_expected += 1;
        assert_eq!(PlacementAttemptIterator::new(&grid, 13).count(), attempts_expected);

        grid.add_unplaced_word("LOOP");
        attempts_expected += 2;
        assert_eq!(PlacementAttemptIterator::new(&grid, 13).count(), attempts_expected);

        grid.add_unplaced_word("HARICOT");
        attempts_expected += 3;
        assert_eq!(PlacementAttemptIterator::new(&grid, 13).count(), attempts_expected);

        grid.add_unplaced_word("LOLLIPOP");
        attempts_expected += 3 + 2;
        assert_eq!(PlacementAttemptIterator::new(&grid, 13).count(), attempts_expected);

        grid.add_unplaced_word("ABACUS");
        attempts_expected += 4;
        assert_eq!(PlacementAttemptIterator::new(&grid, 13).count(), attempts_expected);
    }

    #[test]
    fn test_iterator() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
        grid.add_unplaced_word("ABACUS");
        assert_eq!(PlacementAttemptIterator::new(&grid, 13).count(), 9*2 + 1);
        grid.add_unplaced_word("LOOP");
        assert_eq!(PlacementAttemptIterator::new(&grid, 13).count(), 9*2 + 1 + 4*2 + 1);
    }

    #[test]
    fn test_use_attempts() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
        grid.add_unplaced_word("ABACUS");
        grid.add_unplaced_word("LOOP");
        grid.add_unplaced_word("BEE");
        //assert_eq!(count_successful_attempts(&grid), 5);

        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/everyman_starter.txt");
        grid.add_unplaced_word("PROBONO");
        grid.add_unplaced_word("PASTURE");
        grid.add_unplaced_word("VETO");
        grid.add_unplaced_word("EROS");
        // Note that whenever a valid word placement crosses multiple open cells, you will get a
        // success starting from each of the open cells
        assert_eq!(count_successful_attempts(&grid), 2 + 5 + 3 + 5);
    }

    fn count_successful_attempts(grid: &CrosswordGrid) -> usize {
        let mut num_successes = 0;
        for attempt in PlacementAttemptIterator::new(grid, 13) {
            info!("Trying attempt {:?}", attempt);
            let mut grid_clone = grid.clone();
            let success = grid_clone.try_place_word_in_cell(attempt.location,
                                                            attempt.word_id,
                                                            attempt.index_in_word,
                                                            attempt.direction);
            info!("Success for attempt {:?}: {}", attempt, success);
            if success {
                info!("Resulting grid\n{}", grid_clone.to_string());
                num_successes += 1;
            } else {
                assert_eq!(grid_clone.to_string(), grid.to_string());
            }
        }
        num_successes
    }
}
