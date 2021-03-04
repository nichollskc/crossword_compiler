use crate::graph::Graph;
use log::debug;
use std::collections::HashMap;
use std::fmt;

mod builder;
mod word;
mod cell;

mod add_word;
mod random;
mod spacing;
mod properties;
mod pdf_conversion;

use word::Word;
use cell::Cell;
pub use builder::CrosswordGridBuilder;
pub use pdf_conversion::CrosswordPrinter;

static VALIDCHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Clone,Copy,Debug,PartialEq,Eq,Ord,PartialOrd,Hash)]
pub enum Direction {
    Across,
    Down,
}

impl Direction {
    fn rotate(&self) -> Self {
        match self {
            Direction::Across => Direction::Down,
            Direction::Down => Direction::Across,
        }
    }
}

#[derive(Clone,Copy,Eq,Hash)]
pub struct Location(pub isize, pub isize);

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Location({}, {})", self.0, self.1)
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Location) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Location {
    fn relative_location(&self, move_across: isize, move_down: isize) -> Location {
        Location(self.0 + move_across, self.1 + move_down)
    }

    fn relative_location_directed(&self, move_size: isize, direction: Direction) -> Location {
        match direction {
            Direction::Across => Location(self.0, self.1 + move_size),
            Direction::Down => Location(self.0 + move_size, self.1),
        }
    }
}

#[derive(Clone)]
pub struct CrosswordGrid {
    cell_map: HashMap<Location, Cell>,
    word_map: HashMap<usize, Word>,
    top_left_cell_index: Location,
    bottom_right_cell_index: Location,
}

impl fmt::Debug for CrosswordGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut words: Vec<(&usize, &Word)> = self.word_map.iter().collect();
        words.sort_by_key(|a| *a.0);
        let word_strs: Vec<String> = words.iter().map(|x| format!("{:?}: {:?}", x.0, x.1)).collect();

        let mut cells: Vec<(&Location, &Cell)> = self.cell_map.iter().collect();
        cells.sort_by_key(|a| (a.0.0, a.0.1));
        let cell_strs: Vec<String> = cells.iter().map(|x| format!("{:?}: {:?}", x.0, x.1)).collect();

        write!(f, "CrosswordGrid(\nIndices: Top left {:?} Bottom right {:?}\nWords:{:#?}\nCells:{:#?}\n))",
               self.top_left_cell_index, self.bottom_right_cell_index, word_strs, cell_strs)
    }
}

impl CrosswordGrid {
    pub fn new_single_word(word: &str) -> Self {
        let mut builder = builder::CrosswordGridBuilder::new();
        builder.from_string(word)
    }

    fn new_from_wordmap_single_placed(word_id: usize,
                                      direction: Direction,
                                      mut word_map: HashMap<usize, Word>) -> Self {
        let mut location = Location(0, 0);
        let across_id: Option<usize>;
        let down_id: Option<usize>;
        let mut cell_map: HashMap<Location, Cell> = HashMap::new();

        match direction {
            Direction::Across => {
                across_id = Some(word_id);
                down_id = None;
            },
            Direction::Down => {
                across_id = None;
                down_id = Some(word_id);
            },
        };
        let mut word = word_map.get_mut(&word_id).unwrap();
        word.update_location(location, direction);
        for c in word.word_text.chars() {
            cell_map.insert(location, Cell::new(c, across_id, down_id));
            location = location.relative_location_directed(1, direction);
        }

        let mut grid = CrosswordGrid {
            cell_map,
            word_map,
            top_left_cell_index: Location(0, 0),
            bottom_right_cell_index: location.relative_location_directed(-1, direction),
        };

        grid.fit_to_size();
        grid
    }

    pub fn to_graph(&self) -> Graph {
        let mut edges: Vec<(usize, usize)> = vec![];
        for cell in self.cell_map.values() {
            if cell.is_intersection() {
                edges.push((cell.get_across_word_id().unwrap(),
                            cell.get_down_word_id().unwrap()));
            }
        }
        edges.sort();
        debug!("All intersections found {:#?}", edges);
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

    pub fn add_unplaced_word_at_id(&mut self, word_text: &str, word_id: usize, required_direction: Option<Direction>) {
        let word = Word::new_unplaced(word_text, required_direction);
        self.word_map.insert(word_id, word);
    }

    pub fn add_unplaced_word(&mut self, word_text: &str, required_direction: Option<Direction>) -> usize {
        let word_id = self.find_lowest_unused_word_id();
        self.add_unplaced_word_at_id(word_text, word_id, required_direction);
        word_id
    }

    pub fn update_word_id(&mut self, old_word_id: usize, new_word_id: usize) {
        // Move in hashmap
        let word: Word = self.word_map.remove(&old_word_id).unwrap();
        self.word_map.insert(new_word_id, word);

        for (_location, cell) in self.cell_map.iter_mut() {
            cell.update_word_id(old_word_id, new_word_id);
        }
    }

    pub fn delete_word(&mut self, word_id:usize) {
        self.unplace_word(word_id);
        self.word_map.remove(&word_id);
    }

    pub fn unplace_word(&mut self, word_id: usize) {
        for (_location, cell) in self.cell_map.iter_mut() {
            cell.remove_word(word_id);
        }
        if let Some(word) = self.word_map.get_mut(&word_id) {
            word.remove_placement();
        }
        debug!("Now have {} words in grid", self.word_map.len());
    }
}
