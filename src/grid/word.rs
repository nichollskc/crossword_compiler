use log::warn;
use thiserror::Error;

use super::Location;
use super::Direction;
use super::{VALID_CLUECHARS,VALID_ANSWERCHARS};

use crate::sanitise_string;

use regex::Regex;

#[derive(Error,Debug)]
pub enum ParseError {
    #[error("Invalid character '{0}' found in supplied answer: '{1}'")]
    InvalidAnswerChar(char, String),

    #[error("Supplied answer is empty: '{0}'")]
    EmptyAnswer(String)
}

fn parse_answer_string(string: &str) -> Result<(String, String), ParseError> {
    let mut word_lengths = String::from("(");
    let mut word = String::new();
    let mut current_word_len = 0;
    for c in string.chars() {
        match c {
            '-' => {
                word_lengths.push_str(&format!("{}-", current_word_len));
                current_word_len = 0;
            },
            ' ' => {
                word_lengths.push_str(&format!("{},", current_word_len));
                current_word_len = 0;
            },
            'A'..='z' => {
                word.push(c.to_ascii_uppercase());
                current_word_len += 1;
            },
            _ => Err(ParseError::InvalidAnswerChar(c, string.to_string()))?,
        }
    }
    word_lengths.push_str(&format!("{})", current_word_len));

    match word.len() {
        0 => Err(ParseError::EmptyAnswer(string.to_string())),
        _ => Ok((word, word_lengths)),
    }
}

fn clue_contains_word_lengths(string: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\([-,\d]+\)").unwrap();
    }
    RE.is_match(string)
}

fn parse_clue_string(string: &str) -> Result<(String, String, Option<Direction>), ParseError> {
    let mut components = string.split("::");

    let word_text: &str = components.next().unwrap();
    let (sanitised_word, word_lengths) = parse_answer_string(word_text)?;
    let clue: &str = match components.next() {
        Some(clue_text) => clue_text,
        None => "",
    };
    let mut sanitised_clue: String = sanitise_string(clue, VALID_CLUECHARS);
    if !clue_contains_word_lengths(&sanitised_clue) {
        sanitised_clue.push_str(" ");
        sanitised_clue.push_str(&word_lengths);
    }

    let required_direction: Option<Direction> = match components.next() {
        Some(x) if x.to_uppercase() == "ACROSS" => Some(Direction::Across),
        Some(x) if x.to_uppercase() == "DOWN" => Some(Direction::Down),
        Some(x) => {
            warn!("Unexpected word at end of clue, expected 'ACROSS', 'DOWN' or empty. Parsed as if it were empty. {}", x);
            None
        },
        None => None,
    };
    Ok((sanitised_word, sanitised_clue, required_direction))
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

    pub fn new_parsed(string: &str) -> Result<Self, ParseError> {
        let (word, clue, required_direction) = parse_clue_string(string)?;
        Ok(Word::new_unplaced(&word, &clue, required_direction))
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

    pub fn get_required_direction(&self) -> Option<Direction> {
        self.required_direction
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
}

#[cfg(test)]
mod tests {
	use rstest::rstest;
    use super::*;

    #[rstest(clue_string, word, clue, required_direction,
      case("WORD::clue::ACROSS", "WORD", "clue (4)", Some(Direction::Across)),
      case("WORD::clue::across", "WORD", "clue (4)", Some(Direction::Across)),
      case("WORD::clue::Across", "WORD", "clue (4)", Some(Direction::Across)),
      case("WORD::clue with multiple words::ACROSS",
           "WORD", "clue with multiple words (4)", Some(Direction::Across)),
      case("ANOXIC::Gripped by sudden fear, topless opponents in game lacking vital element (6)::ACROSS",
           "ANOXIC", "Gripped by sudden fear, topless opponents in game lacking vital element (6)", Some(Direction::Across)),
      case("WORD::clue::DOWN", "WORD", "clue (4)", Some(Direction::Down)),
      case("WORD::clue::down", "WORD", "clue (4)", Some(Direction::Down)),
      case("SONNET::Lines up outside No 10 — speech just beginning (6)::DOWN",
           "SONNET", "Lines up outside No 10 — speech just beginning (6)", Some(Direction::Down)),
      case("WORD::clue::", "WORD", "clue (4)", None),
      case("WORD::clue::blabla", "WORD", "clue (4)", None),
      case("WORD::clue", "WORD", "clue (4)", None),
      case("BELLY FLOP::clue", "BELLYFLOP", "clue (5,4)", None),
      case("WORD", "WORD", " (4)", None),
      case("TEA-TIME::clue", "TEATIME", "clue (3-4)", None),
      case("ANOXIC::Gripped by sudden fear, topless opponents in game lacking vital element (6)::",
           "ANOXIC", "Gripped by sudden fear, topless opponents in game lacking vital element (6)", None),
      )]
    fn test_parse_clue_string(clue_string: &str, word: &str, clue: &str, required_direction: Option<Direction>) -> Result<(), ParseError> {
        assert_eq!(parse_clue_string(clue_string)?,
                   (word.to_string(), clue.to_string(), required_direction));
        Ok(())
    }

    #[rstest(string, word, word_lengths,
      case("TEA-TIME", "TEATIME", "(3-4)"),
      case("BILBO BAGGINS", "BILBOBAGGINS", "(5,7)"),
      case("tea-time", "TEATIME", "(3-4)"),
      case("tea-TIME", "TEATIME", "(3-4)"),
      )]
    fn test_parse_answer_string(string: &str, word: &str, word_lengths: &str) -> Result<(), ParseError> {
        crate::logging::init_logger(true);
        assert_eq!(parse_answer_string(string)?,
                   (word.to_string(), word_lengths.to_string()));
        Ok(())
    }

    #[rstest(input, expected,
      case("Lines up outside No 10 — speech just beginning (6)", true),
	  case("Lines up outside No 10 — speech just beginning (3-4)", true),
	  case("Lines up outside No 10 — speech just beginning (3,4)", true),
	  case("Lines up outside No 10 — speech just beginning (10,1,1)", true),
	  case("Lines up outside No 10 — speech just beginning", false),
	  case("Lines up outside No 10 — (speech just beginning)", false),
      )]
    fn test_check_word_lengths_in_string(input: &str, expected: bool) {
        crate::logging::init_logger(true);
        assert_eq!(expected, clue_contains_word_lengths(input))
    }
}
