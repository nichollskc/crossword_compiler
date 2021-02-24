use log::{info,warn,debug,error};
use std::collections::{HashSet,HashMap};
use std::fs;

struct Cell {
    letter: char,
    location: (isize, isize),
}

impl Cell {
    fn new(letter: char, location: (isize, isize)) -> Self {
        Cell {
            letter,
            location,
        }
    }
}

pub struct CrosswordGrid {
    cell_list: Vec<Cell>,
    cell_map: HashMap<(isize, isize), usize>,
    top_left_cell_index: usize,
}

impl CrosswordGrid {
    pub fn from_file(filename: &str) -> Self {
        let data = fs::read_to_string(filename).expect("Unable to read file");
        println!("{}", data);

        CrosswordGrid {
            cell_list: vec![],
            cell_map: HashMap::new(),
            top_left_cell_index: 0,
        }
    }
}
