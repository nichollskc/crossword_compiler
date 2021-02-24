use log::{info,warn,debug,error};
use std::collections::{HashSet,HashMap};
use std::fs;

#[derive(Debug)]
enum FillStatus {
    Filled(FilledCell),
    // Nothing known about cell
    Empty,
    // Must be black - just before word start or just after word end
    Black,
}

#[derive(Debug)]
struct FilledCell {
    letter: char,
    across_word_id: Option<usize>,
    down_word_id: Option<usize>,
}

#[derive(Debug)]
struct Cell {
    fill_status: FillStatus,
    location: (isize, isize),
}

impl Cell {
    fn new(letter: char, location: (isize, isize), across_word_id: Option<usize>, down_word_id: Option<usize>) -> Self {
        let filled_cell = FilledCell {
            letter,
            across_word_id,
            down_word_id,
        };
        Cell {
            fill_status: FillStatus::Filled(filled_cell),
            location,
        }
    }

    fn empty(location: (isize, isize)) -> Self {
        Cell {
            fill_status: FillStatus::Empty,
            location,
        }
    }
}

#[derive(Debug)]
struct Word {
    word_text: String,
}

#[derive(Debug)]
pub struct CrosswordGrid {
    cell_map: HashMap<(isize, isize), Cell>,
    word_map: HashMap<usize, Word>,
    top_left_cell_index: usize,
}

impl CrosswordGrid {
    fn remove_word(mut self, word_id: usize) {
        self.word_map.remove(&word_id);
    }
}

pub struct CrosswordGridBuilder {
    cell_map: HashMap<(isize, isize), Cell>,
    word_map: HashMap<usize, Word>,
    current_across_word_id: Option<usize>,
    current_down_word_ids: HashMap<isize, Option<usize>>,
    row: isize,
    col: isize,
    index: usize,
    word_index: usize,
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
            word_index: 0,
        }
    }

    pub fn from_file(mut self, filename: &str) -> CrosswordGrid {
        let contents = fs::read_to_string(filename).expect("Unable to read file");
        let characters: Vec<char> = contents.chars().collect();

        for c in characters {
            if c == '\n' {
                self.row += 1;
                self.col = 0;
            } else {
                if self.row == 0 {
                    self.current_down_word_ids.insert(self.col, None);
                }
                let location = (self.row, self.col);

                if c == ' ' {
                    // End any existing words we have
                    self.current_across_word_id = None;
                    self.current_down_word_ids.insert(self.col, None);

                    // Add empty cell to our grid
                    self.cell_map.insert(location, Cell::empty(location));
                } else {
                    if let Some(word_id) = self.current_across_word_id {
                        self.word_map.get_mut(&word_id).unwrap().word_text.push(c);
                    } else {
                        self.word_map.insert(self.word_index, Word { word_text: c.to_string() });
                        self.current_across_word_id = Some(self.word_index);
                        self.word_index += 1;
                    }
                    if let Some(word_id) = *self.current_down_word_ids.get(&self.col).unwrap() {
                        self.word_map.get_mut(&word_id).unwrap().word_text.push(c);
                    } else {
                        self.word_map.insert(self.word_index, Word { word_text: c.to_string() });
                        self.current_down_word_ids.insert(self.col, Some(self.word_index));
                        self.word_index += 1;
                    }

                    self.cell_map.insert(location,
                                         Cell::new(c,
                                                   location,
                                                   self.current_across_word_id,
                                                   *self.current_down_word_ids.get(&self.col).unwrap()));
                }
                self.col += 1;
                self.index += 1;
            }
        }

        CrosswordGrid {
            cell_map: self.cell_map,
            word_map: self.word_map,
            top_left_cell_index: 0,
        }
    }
}
