use crate::graph::Graph;
use std::cmp;
use log::{info,warn,debug,error};
use std::collections::HashMap;

use super::Location;

#[derive(Clone,Copy,Debug)]
enum FillStatus {
    Filled(FilledCell),
    // Nothing known about cell
    Empty,
    // Must be black - just before word start or just after word end
    Black,
}

#[derive(Clone,Copy,Debug)]
struct FilledCell {
    letter: char,
    across_word_id: Option<usize>,
    down_word_id: Option<usize>,
}

impl FilledCell {
    fn new(letter: char, across_word_id: Option<usize>, down_word_id: Option<usize>) -> Self {
        FilledCell {
            letter,
            across_word_id,
            down_word_id,
        }
    }
}

#[derive(Clone,Copy,Debug)]
struct Cell {
    fill_status: FillStatus,
    location: Location,
}

impl Cell {
    pub fn new(letter: char, location: Location, across_word_id: Option<usize>, down_word_id: Option<usize>) -> Self {
        Cell {
            fill_status: FillStatus::Filled(FilledCell::new(letter, across_word_id, down_word_id)),
            location,
        }
    }

    fn empty(location: Location) -> Self {
        Cell {
            fill_status: FillStatus::Empty,
            location,
        }
    }

    fn remove_word(&mut self, word_id: usize) {
        if let FillStatus::Filled(filled_cell) = self.fill_status {
            let mut across_word_id = self.get_across_word_id();
            let mut down_word_id = self.get_down_word_id();
            if across_word_id == Some(word_id) {
                across_word_id = None;
            }
            if down_word_id == Some(word_id) {
                down_word_id = None;
            }
            if across_word_id.is_none() && down_word_id.is_none() {
                self.fill_status = FillStatus::Empty;
            } else {
                self.fill_status = FillStatus::Filled(FilledCell::new(filled_cell.letter,
                                                                      across_word_id,
                                                                      down_word_id));
            }
        }
    }

    fn add_word(&mut self, word_id: usize, letter: char, across: bool) -> bool {
        let mut success = true;

        let mut across_word_id: Option<usize> = None;
        let mut down_word_id: Option<usize> = None;
        if across {
            across_word_id = Some(word_id);
        } else {
            down_word_id = Some(word_id);
        }

        match self.fill_status {
            FillStatus::Filled(filled_cell) => {
                let existing_across = filled_cell.across_word_id;
                let existing_down = filled_cell.down_word_id;

                if across {
                    // We are updating across word id, so can happily keep the existing down word id
                    down_word_id = existing_down;
                    if existing_across.is_some() && existing_across != across_word_id {
                        // Existing ID this is a problem if the new id doesn't match the old ID
                        warn!("Existing across word ID doesn't match new one {} {}", existing_across.unwrap(), across_word_id.unwrap());
                        success = false
                    }
                } else {
                    // We are updating down word id, so can happily keep the existing across word id
                    across_word_id = existing_across;

                    if existing_down.is_some() && existing_down != down_word_id {
                        // Existing ID this is a problem if the new id doesn't match the old ID
                        warn!("Existing down word ID doesn't match new one {} {}", existing_down.unwrap(), down_word_id.unwrap());
                        success = false
                    }
                }

                if filled_cell.letter != letter {
                    info!("Existing letter doesn't match new one {} {}", filled_cell.letter, letter);
                    success = false;
                }
            },
            FillStatus::Empty => {},
            FillStatus::Black => {
                success = false
            },
        }

        if success {
            self.fill_status = FillStatus::Filled(FilledCell::new(letter,
                                                                  across_word_id,
                                                                  down_word_id));
        }
        success
    }

    fn get_down_word_id(&self) -> Option<usize> {
        if let FillStatus::Filled(filled_cell) = self.fill_status {
            filled_cell.down_word_id
        } else {
            None
        }
    }

    fn get_across_word_id(&self) -> Option<usize> {
        if let FillStatus::Filled(filled_cell) = self.fill_status {
            filled_cell.across_word_id
        } else {
            None
        }
    }

    fn is_intersection(&self) -> bool {
        if self.get_across_word_id().is_some() && self.get_down_word_id().is_some() {
            true
        } else {
            false
        }
    }

    fn set_black(&mut self) {
        self.fill_status = FillStatus::Black;
    }

    fn contains_letter(&self) -> bool {
        if let FillStatus::Filled(_filled_cell) = self.fill_status {
            true
        } else {
            false
        }
    }

    fn to_char(&self) -> char {
        if let FillStatus::Filled(filled_cell) = self.fill_status {
            filled_cell.letter
        } else {
            ' '
        }
    }

    fn is_empty(&self) -> bool {
        if let FillStatus::Empty = self.fill_status {
            true
        } else {
            false
        }
    }

    fn is_black(&self) -> bool {
        if let FillStatus::Black = self.fill_status {
            true
        } else {
            false
        }
    }
}
