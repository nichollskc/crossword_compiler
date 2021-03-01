use crate::graph::Graph;
use std::cmp;
use log::{info,warn,debug,error};
use std::collections::HashMap;

mod builder;
mod word;
mod cell;
mod add_word;
mod spacing;
mod properties;

use ::word::Word;
use crate::grid::cell::Cell;

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
        self.check_valid();

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
        info!("{:#?}", graph);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_black_cells() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGrid::new_single_word("ALPHA");
        debug!("{:#?}", grid);
        grid.fit_to_size();
        debug!("{:#?}", grid);
        grid.fill_black_cells();

        assert_eq!(grid.cell_map.values().filter(|&x| x.is_black()).count(), 2);

        assert!(grid.cell_map.get(&Location(0, -1)).unwrap().is_black());
        assert!(grid.cell_map.get(&Location(0, 5)).unwrap().is_black());

        let mut grid = builder::CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
        grid.fit_to_size();
        grid.fill_black_cells();
        assert_eq!(grid.cell_map.values().filter(|&x| x.is_black()).count(), 18);
    }

    #[test]
    fn test_count_filled_cells() {
        crate::logging::init_logger(true);
        let grid = CrosswordGrid::new_single_word("ALPHA");
        assert!(grid.cell_map.get(&Location(0, 0)).unwrap().contains_letter());

        for i in 0..4 {
            assert_eq!(grid.count_filled_cells_col(i), 1);
        }
        assert_eq!(grid.count_filled_cells_row(0), 5);

        let grid = builder::CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
        let row_counts: Vec<usize> = vec![6, 2, 9, 3, 6, 3, 10, 2, 1];
        let col_counts: Vec<usize> = vec![2, 6, 5, 4, 4, 7, 3, 4, 5, 2];

        for i in 0..9 {
            assert_eq!(grid.count_filled_cells_row(i as isize), row_counts[i]);
        }
        for i in 0..10 {
            assert_eq!(grid.count_filled_cells_col(i as isize), col_counts[i]);
        }
    }

    #[test]
    fn test_fit_to_size() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGrid::new_single_word("ALPHA");
        grid.fit_to_size();
        assert_eq!(grid.cell_map.len(), 7*3);
        // Shouldn't change size on second call of function
        grid.fit_to_size();
        assert_eq!(grid.cell_map.len(), 7*3);

        let mut grid = builder::CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
        // Number of non-empty cells shouldn't change
        grid.fit_to_size();
        let row_counts: Vec<usize> = vec![6, 2, 9, 3, 6, 3, 10, 2, 1];
        let col_counts: Vec<usize> = vec![2, 6, 5, 4, 4, 7, 3, 4, 5, 2];

        for i in 0..9 {
            assert_eq!(grid.count_filled_cells_row(i as isize), row_counts[i]);
        }
        for i in 0..10 {
            assert_eq!(grid.count_filled_cells_col(i as isize), col_counts[i]);
        }

        let mut grid = builder::CrosswordGridBuilder::new().from_file("tests/resources/blank_space.txt");
        // Number of non-empty cells shouldn't change
        grid.fit_to_size();
        info!("Grid coords {:#?} {:#?}", grid.top_left_cell_index, grid.bottom_right_cell_index);
        assert_eq!(grid.cell_map.len(), 12*11);
        let row_counts: Vec<usize> = vec![6, 2, 9, 3, 6, 3, 10, 2, 1];
        let col_counts: Vec<usize> = vec![2, 6, 5, 4, 4, 7, 3, 4, 5, 2];

        for i in 0..9 {
            assert_eq!(grid.count_filled_cells_row(i as isize + 4), row_counts[i]);
        }
        for i in 0..10 {
            assert_eq!(grid.count_filled_cells_col(i as isize + 4), col_counts[i]);
        }
    }

    #[test]
    fn test_open_cells() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGrid::new_single_word("ALPHA");
        grid.fit_to_size();
        grid.fill_black_cells();
        debug!("{:#?}", grid);

        for i in -1..5 {
            assert!(!grid.cell_is_open_across(Location(0, i)), "Cell (0, {}) should not be open across", i);
            assert!(!grid.cell_is_open_across(Location(-1, i)), "Cell (0, {}) should not be open across", i);
            assert!(!grid.cell_is_open_across(Location(1, i)), "Cell (0, {}) should not be open across", i);
            assert!(!grid.cell_is_open_down(Location(-1, i)), "Cell (0, {}) should not be open down", i);
            assert!(!grid.cell_is_open_down(Location(1, i)), "Cell (0, {}) should not be open down", i);
        }

        for i in 0..4 {
            assert!(grid.cell_is_open_down(Location(0, i)), "Cell (0, {}) should be open down", i);
        }

        assert!(!grid.cell_is_open_down(Location(0, -1)));
        assert!(!grid.cell_is_open_down(Location(0, 5)));

        let mut grid = builder::CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
        grid.fit_to_size();
        grid.fill_black_cells();
        debug!("{:#?}", grid);

        assert!(grid.cell_is_open_down(Location(2, 0)));
        assert!(!grid.cell_is_open_down(Location(2, 1)));
        assert!(!grid.cell_is_open_down(Location(2, 2)));
        assert!(grid.cell_is_open_down(Location(2, 3)));

        assert!(!grid.cell_is_open_across(Location(3, 0)));
        assert!(grid.cell_is_open_across(Location(3, 1)));
        assert!(!grid.cell_is_open_across(Location(3, 2)));
        assert!(!grid.cell_is_open_across(Location(3, 3)));
        assert!(grid.cell_is_open_across(Location(3, 5)));
    }

    #[test]
    fn add_word_to_grid() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGrid::new_single_word("ALPHA");
        grid.fit_to_size();
        grid.fill_black_cells();

        let arrival_word_id = grid.add_unplaced_word("ARRIVAL");
        let bear_word_id = grid.add_unplaced_word("BEARER");
        let innards_word_id = grid.add_unplaced_word("INNARDS");
        let cup_word_id = grid.add_unplaced_word("CUP");
        let cap_word_id = grid.add_unplaced_word("CAP");
        debug!("{:#?}", grid);

        assert!(grid.try_place_word_in_cell(Location(0, 0), arrival_word_id, 0, false));
        assert!(grid.try_place_word_in_cell(Location(0, 4), bear_word_id, 2, false));
        assert!(grid.try_place_word_in_cell(Location(0, 2), cup_word_id, 2, false));

        let before_failure = grid.to_string();
        assert!(!grid.try_place_word_in_cell(Location(0, 3), bear_word_id, 1, false));
        assert_eq!(before_failure, grid.to_string());

        assert!(!grid.try_place_word_in_cell(Location(-2, 2), cap_word_id, 0, true));
        assert_eq!(before_failure, grid.to_string());
        info!("{}", grid.to_string());

        debug!("{:#?}", grid);
        debug!("TESTING");
        assert!(grid.try_place_word_in_cell(Location(3, 0), innards_word_id, 0, true));

        let mut from_file = builder::CrosswordGridBuilder::new().from_file("tests/resources/built_up.txt");
        from_file.fit_to_size();
        debug!("{}", grid.to_string());
        assert_eq!(from_file.to_string(), grid.to_string());
    }
}
