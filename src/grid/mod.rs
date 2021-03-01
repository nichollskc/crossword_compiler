use crate::graph::Graph;
use log::{info,warn,debug,error};
use std::collections::HashMap;

mod builder;
mod word;
mod cell;

mod add_word;
mod random;
mod spacing;
mod properties;

use word::Word;
use cell::Cell;
pub use builder::CrosswordGridBuilder;

static VALIDCHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Clone,Copy,Debug,Eq,Hash)]
pub struct Location(pub isize, pub isize);

impl PartialEq for Location {
    fn eq(&self, other: &Location) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Location {
    fn relative_location(&self, move_across: isize, move_down: isize) -> Location {
        Location(self.0 + move_across, self.1 + move_down)
    }

    fn relative_location_directed(&self, move_size: isize, to_col: bool) -> Location {
        if to_col {
            Location(self.0, self.1 + move_size)
        } else {
            Location(self.0 + move_size, self.1)
        }
    }
}

#[derive(Clone,Debug)]
pub struct CrosswordGrid {
    cell_map: HashMap<Location, Cell>,
    word_map: HashMap<usize, Word>,
    top_left_cell_index: Location,
    bottom_right_cell_index: Location,
}

impl CrosswordGrid {
    pub fn new_single_word(word: &str) -> Self {
        let mut builder = builder::CrosswordGridBuilder::new();
        builder.from_string(word)
    }

    pub fn to_graph(&self) -> Graph {
        let mut edges: Vec<(usize, usize)> = vec![];
        for cell in self.cell_map.values() {
            if cell.is_intersection() {
                edges.push((cell.get_across_word_id().unwrap(),
                            cell.get_down_word_id().unwrap()));
            }
        }
        info!("All intersections found {:#?}", edges);
        let mut graph = Graph::new_from_edges(edges);

        for (word_id, _word) in self.word_map.iter().filter(|(_id, w)| w.is_placed()) {
            graph.add_node(*word_id);
        }
        graph
    }

    pub fn to_string(&self) -> String {
        let mut string: String = String::from("");
        let mut row = self.top_left_cell_index.0 + 1;
        let mut col = self.top_left_cell_index.1 + 1;
        while row < self.bottom_right_cell_index.0 {
            while col < self.bottom_right_cell_index.1 {
                let c = self.cell_map.get(&Location(row, col)).unwrap().to_char();
                string.push(c);
                col += 1;
            }
            col = self.top_left_cell_index.1 + 1;
            row += 1;
            string.push('\n');
        }
        string
    }

    pub fn check_valid(&self) {
        assert!(self.top_left_cell_index.0 <= self.bottom_right_cell_index.0);
        assert!(self.top_left_cell_index.1 <= self.bottom_right_cell_index.1);

        let mut row = self.top_left_cell_index.0;
        let mut col = self.top_left_cell_index.1;

        while row <= self.bottom_right_cell_index.0 {
            while col <= self.bottom_right_cell_index.1 {
                let present = self.cell_map.contains_key(&Location(row, col));
                if !present {
                    panic!("Cell not present in grid {}, {}", row, col);
                }
                col += 1;
            }
            col = self.top_left_cell_index.1;
            row += 1;
        }

        for cell in self.cell_map.values() {
            if let Some(word_id) = cell.get_across_word_id() {
                assert!(self.word_map.contains_key(&word_id));
            }
            if let Some(word_id) = cell.get_down_word_id() {
                assert!(self.word_map.contains_key(&word_id));
            }
        }

        let graph = self.to_graph();
        debug!("{:#?}", graph);
        debug!("{:#?}", self.word_map);
        debug!("Checking grid connected {}", self.to_string());
        assert!(graph.is_connected());
    }

    fn find_lowest_unused_word_id(&self) -> usize {
        let mut word_id: usize = 0;
        while self.word_map.contains_key(&word_id) {
            word_id += 1;
        }
        word_id
    }

    pub fn add_unplaced_word(&mut self, word_text: &str) -> usize {
        let word = Word::new_unplaced(word_text);
        let word_id = self.find_lowest_unused_word_id();
        self.word_map.insert(word_id, word);
        word_id
    }

    pub fn remove_word(&mut self, word_id: usize) {
        self.word_map.remove(&word_id);
        for (_location, cell) in self.cell_map.iter_mut() {
            cell.remove_word(word_id);
        }
        if let Some(word) = self.word_map.get_mut(&word_id) {
            word.remove_placement();
        }
    }
}
