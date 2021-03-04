use log::warn;

use super::Location;
use super::Direction;
use super::{VALID_CLUECHARS,VALID_ANSWERCHARS};

use crate::sanitise_string;

fn parse_clue_string(string: &str) -> (String, String, Option<Direction>) {
    let mut components = string.split("::");

    let word_text: &str = components.next().unwrap();
    let sanitised_word: String = sanitise_string(word_text, VALID_ANSWERCHARS);
    let clue: &str = match components.next() {
        Some(clue_text) => clue_text,
        None => "",
    };
    let sanitised_clue: String = sanitise_string(clue, VALID_CLUECHARS);
    let required_direction: Option<Direction> = match components.next() {
        Some(x) if x.to_uppercase() == "ACROSS" => Some(Direction::Across),
        Some(x) if x.to_uppercase() == "DOWN" => Some(Direction::Down),
        Some(x) => {
            warn!("Unexpected word at end of clue, expected 'ACROSS', 'DOWN' or empty. Parsed as if it were empty. {}", x);
            None
        },
        None => None,
    };
    (sanitised_word, sanitised_clue, required_direction)
}

#[derive(Clone,Copy,Debug)]
struct WordPlacement {
    start_location: Location,
    end_location: Location,
    direction: Direction,
}

impl WordPlacement {
    pub fn new(string: &str, start_location: Location, direction: Direction) -> Self {
        let mut end_location = start_location.clone();
        match direction {
            Direction::Across => { end_location.1 += string.len() as isize - 1; },
            Direction::Down => { end_location.0 += string.len() as isize - 1; },
        }
        WordPlacement {
            start_location,
            end_location,
            direction,
        }
    }
}

#[derive(Clone,Debug)]
pub(super) struct Word {
    pub word_text: String,
    placement: Option<WordPlacement>,
    pub clue: String,
    required_direction: Option<Direction>,
}

impl Word {
    pub fn new(string: &str, start_location: Location, direction: Direction, required_direction: Option<Direction>) -> Self {
        Word {
            word_text: string.to_string(),
            placement: Some(WordPlacement::new(string, start_location, direction)),
            clue: "Bla bla bla (6)".to_string(),
            required_direction,
        }
    }

    pub fn new_unplaced(word_text: &str, clue: &str, required_direction: Option<Direction>) -> Self {
        Word {
            word_text: word_text.to_string(),
            placement: None,
            clue: clue.to_string(),
            required_direction,
        }
    }

    pub fn new_parsed(string: &str) -> Self {
        let (word, clue, required_direction) = parse_clue_string(string);
        Word::new_unplaced(&word, &clue, required_direction)
    }

    pub fn get_location(&self) -> Option<(Location, Location, Direction)> {
        if let Some(word_placement) = &self.placement {
            Some((word_placement.start_location, word_placement.end_location, word_placement.direction))
        } else {
            None
        }
    }

    pub fn remove_placement(&mut self) {
        self.placement = None;
    }

    pub fn extend_word(&mut self, character: char) -> Option<Location> {
        self.word_text.push(character);
        if let Some(word_placement) = &self.placement {
            let mut new_word_placement = word_placement.clone();
            new_word_placement.end_location = word_placement.end_location.relative_location_directed(1, word_placement.direction);
            self.placement = Some(new_word_placement);
            Some(new_word_placement.end_location)
        } else {
            None
        }
    }

    pub fn is_placed(&self) -> bool {
        self.get_location().is_some()
    }

    pub fn len(&self) -> usize {
        self.word_text.len()
    }

    pub fn get_char_at_index(&self, index: usize) -> char {
        self.word_text.chars().nth(index).unwrap()
    }

    pub fn allowed_in_direction(&self, direction: Direction) -> bool {
        match self.required_direction {
            // If no requirements, anything is allowed
            None => true,
            // If there is a requirement, only allowed if the directions match
            Some(dir) => dir == direction,
        }
    }

    pub fn update_location(&mut self, start_location: Location, direction: Direction) {
        assert!(self.allowed_in_direction(direction),
                "Attempted to add word with invalid direction {:?}: {:?}", direction, self);
        self.placement = Some(WordPlacement::new(&self.word_text, start_location, direction));
    }

    pub fn update_required_direction(&mut self, required_direction: Option<Direction>) {
        self.required_direction = required_direction;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::info;

    fn parse_clue_test_helper(clue_string: &str, word: &str, clue: &str, required_direction: Option<Direction>) {
        assert_eq!(parse_clue_string(clue_string),
                   (word.to_string(), clue.to_string(), required_direction));
    }
    fn parse_clue_test_helper_across(clue_string: &str, word: &str, clue: &str) {
        parse_clue_test_helper(clue_string, word, clue, Some(Direction::Across));
    }
    fn parse_clue_test_helper_down(clue_string: &str, word: &str, clue: &str) {
        parse_clue_test_helper(clue_string, word, clue, Some(Direction::Down));
    }
    fn parse_clue_test_helper_none(clue_string: &str, word: &str, clue: &str) {
        parse_clue_test_helper(clue_string, word, clue, None);
    }

    #[test]
    fn test_parse_clue_string() {
        parse_clue_test_helper_across("WORD::clue::ACROSS", "WORD", "clue");
        parse_clue_test_helper_across("WORD::clue::across", "WORD", "clue");
        parse_clue_test_helper_across("WORD::clue::Across", "WORD", "clue");
        parse_clue_test_helper_down("WORD::clue::DOWN", "WORD", "clue");
        parse_clue_test_helper_down("WORD::clue::down", "WORD", "clue");
        parse_clue_test_helper_none("WORD::clue::", "WORD", "clue");
        parse_clue_test_helper_none("WORD::clue::blabla", "WORD", "clue");
        parse_clue_test_helper_none("WORD::clue", "WORD", "clue");
        parse_clue_test_helper_none("BELLY FLOP::clue", "BELLYFLOP", "clue");
        parse_clue_test_helper_none("WORD", "WORD", "");
        parse_clue_test_helper_across("WORD::clue with multiple words::ACROSS",
                                      "WORD", "clue with multiple words");
        parse_clue_test_helper_none("TEA-TIME::clue", "TEATIME", "clue");
        parse_clue_test_helper_none("ANOXIC::Gripped by sudden fear, topless opponents in game lacking vital element (6)::",
                                    "ANOXIC", "Gripped by sudden fear, topless opponents in game lacking vital element (6)");
        parse_clue_test_helper_across("ANOXIC::Gripped by sudden fear, topless opponents in game lacking vital element (6)::ACROSS",
                                    "ANOXIC", "Gripped by sudden fear, topless opponents in game lacking vital element (6)");
        parse_clue_test_helper_down("SONNET::Lines up outside No 10 — speech just beginning (6)::DOWN",
                                    "SONNET", "Lines up outside No 10 — speech just beginning (6)");
    }
}
