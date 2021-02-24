use log::{info,warn,debug,error};
use std::collections::{HashSet,HashMap};
use std::fs;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct CrosswordGrid {
    cell_list: Vec<Cell>,
    cell_map: HashMap<(isize, isize), usize>,
    top_left_cell_index: usize,
}

impl CrosswordGrid {
    pub fn from_file(filename: &str) -> Self {
        let contents = fs::read_to_string(filename).expect("Unable to read file");
        let characters: Vec<char> = contents.chars().collect();

        let mut row: isize = 0;
        let mut col: isize = 0;
        let mut index: usize = 0;

        let mut cell_list: Vec<Cell> = vec![];
        let mut cell_map: HashMap<(isize, isize), usize> = HashMap::new();

        for c in characters {
            if c == '\n' {
                row += 1;
                col = 0;
            } else {
                cell_list.push(Cell::new(c, (row, col)));
                cell_map.insert((row, col), index);
                col += 1;
                index += 1;
            }
        }

        CrosswordGrid {
            cell_list,
            cell_map,
            top_left_cell_index: 0,
        }
    }
}
