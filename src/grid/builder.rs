use std::cmp;
use log::{info,warn,debug,error};
use std::collections::HashMap;

use std::fs;

use super::CrosswordGrid;
use super::Cell;
use super::Location;
use super::Word;

pub struct CrosswordGridBuilder {
    cell_map: HashMap<Location, Cell>,
    word_map: HashMap<usize, Word>,
    current_across_word_id: Option<usize>,
    current_down_word_ids: HashMap<isize, Option<usize>>,
    row: isize,
    col: isize,
    max_col: isize,
    index: usize,
    word_index: usize,
    last_location: Location,
}

impl CrosswordGridBuilder {
    pub fn new() -> Self {
        CrosswordGridBuilder {
            cell_map: HashMap::new(),
            word_map: HashMap::new(),
            current_across_word_id: None,
            current_down_word_ids: HashMap::new(),
            row: 0,
            col: 0,
            index: 0,
            max_col: 0,
            word_index: 0,
            last_location: Location(0, 0),
        }
    }

    pub fn from_file(&mut self, filename: &str) -> CrosswordGrid {
        let contents = fs::read_to_string(filename).expect("Unable to read file");
        debug!("File contents: {}", contents);
        self.from_string(&contents)
    }

    pub fn from_string(&mut self, string: &str) -> CrosswordGrid {
        let characters: Vec<char> = string.chars().collect();

        for c in characters {
            if c == '\n' {
                self.row += 1;
                self.max_col = cmp::max(self.max_col, self.col);
                self.col = 0;
            } else {
                if self.row == 0 {
                    self.current_down_word_ids.insert(self.col, None);
                }
                let location = Location(self.row, self.col);
                self.last_location = location;

                if c == ' ' {
                    // End any existing words we have
                    self.current_across_word_id = None;
                    self.current_down_word_ids.insert(self.col, None);

                    // Add empty cell to our grid
                    self.cell_map.insert(location, Cell::empty());
                } else {
                    if let Some(word_id) = self.current_across_word_id {
                        self.word_map.get_mut(&word_id).unwrap().extend_word(c);
                    } else {
                        self.word_map.insert(self.word_index, Word::new(&c.to_string(), location, true));
                        self.current_across_word_id = Some(self.word_index);
                        self.word_index += 1;
                    }
                    if let Some(word_id) = *self.current_down_word_ids.get(&self.col).unwrap() {
                        self.word_map.get_mut(&word_id).unwrap().extend_word(c);
                    } else {
                        self.word_map.insert(self.word_index, Word::new(&c.to_string(), location, false));
                        self.current_down_word_ids.insert(self.col, Some(self.word_index));
                        self.word_index += 1;
                    }

                    self.cell_map.insert(location,
                                         Cell::new(c,
                                                   self.current_across_word_id,
                                                   *self.current_down_word_ids.get(&self.col).unwrap()));
                }
                self.col += 1;
                self.index += 1;
            }
        }

        let mut grid = CrosswordGrid {
            cell_map: self.cell_map.clone(),
            word_map: self.word_map.clone(),
            top_left_cell_index: Location(0, 0),
            bottom_right_cell_index: self.last_location,
        };

        let mut singleton_word_ids: Vec<usize> = vec![];
        for (word_id, word) in grid.word_map.iter() {
            if word.word_text.len() == 1 {
                singleton_word_ids.push(*word_id);
            }
        }

        for word_id in singleton_word_ids {
            grid.remove_word(word_id);
        }

        grid.fit_to_size();
        grid
    }
}
