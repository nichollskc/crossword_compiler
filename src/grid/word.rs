use super::Location;
use super::Direction;

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
    pub fn new(string: &str, start_location: Location, direction: Direction) -> Self {
        Word {
            word_text: string.to_string(),
            placement: Some(WordPlacement::new(string, start_location, direction)),
            clue: "Bla bla bla (6)".to_string(),
            required_direction: None,
        }
    }

    pub fn new_unplaced(string: &str) -> Self {
        Word {
            word_text: string.to_string(),
            placement: None,
            clue: "Bla bla bla (6)".to_string(),
            required_direction: None,
        }
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
        self.placement = Some(WordPlacement::new(&self.word_text, start_location, direction));
    }
}
