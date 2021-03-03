use super::Location;

#[derive(Clone,Copy,Debug)]
struct WordPlacement {
    start_location: Location,
    end_location: Location,
    across: bool,
}

#[derive(Clone,Debug)]
pub(super) struct Word {
    pub word_text: String,
    placement: Option<WordPlacement>,
    pub clue: String,
}

impl Word {
    pub fn new(string: &str, start_location: Location, across: bool) -> Self {
        let mut end_location = start_location.clone();
        if across {
            end_location.1 += string.len() as isize - 1;
        } else {
            end_location.0 += string.len() as isize - 1;
        }
        let placement = WordPlacement {
            start_location,
            end_location,
            across,
        };
        Word {
            word_text: string.to_string(),
            placement: Some(placement),
            clue: "Bla bla bla (6)".to_string(),
        }
    }

    pub fn new_unplaced(string: &str) -> Self {
        Word {
            word_text: string.to_string(),
            placement: None,
            clue: "Bla bla bla (6)".to_string(),
        }
    }

    pub fn get_location(&self) -> Option<(Location, Location, bool)> {
        if let Some(word_placement) = &self.placement {
            Some((word_placement.start_location, word_placement.end_location, word_placement.across))
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
            new_word_placement.end_location = word_placement.end_location.relative_location_directed(1, word_placement.across);
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
}
