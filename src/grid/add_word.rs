use crate::graph::Graph;
use std::cmp;
use log::{info,warn,debug,error};
use std::collections::HashMap;

use super::CrosswordGrid;

impl CrosswordGrid {
    fn fill_black_cells(&mut self) {
        // Clear black cells before starting
        for (_location, cell) in self.cell_map.iter_mut() {
            if let FillStatus::Black = cell.fill_status {
                cell.fill_status = FillStatus::Empty;
            }
        }

        for word in self.word_map.values() {
            if let Some((start_location, end_location, across)) = word.get_location() {
                let mut black_cells: Vec<Location> = vec![];
                black_cells.push(start_location.relative_location_directed(-1, across));
                black_cells.push(end_location.relative_location_directed(1, across));

                for cell_location in black_cells {
                    if let Some(cell) = self.cell_map.get_mut(&cell_location) {
                        cell.set_black();
                    } else {
                        panic!("Cell doesn't exist! {:#?}, {:#?}", cell_location, word);
                    }
                }
            }
        }
    }

    fn neighbouring_cells_empty(&self, location: Location, neighbour_moves: Vec<(isize, isize)>) -> bool {
        if !self.cell_map.get(&location).unwrap().contains_letter() {
            // If the cell is empty, it cannot be added to - it is not an open cell
            false
        } else {
            let mut result = false;
            for relative_move in neighbour_moves {
                if self.cell_map.get(&location.relative_location(relative_move.0, relative_move.1)).unwrap().is_empty() {
                    result = true;
                }
            }
            result
        }
    }

    fn cell_is_open_across(&self, location: Location) -> bool {
        // If there is already an across word for this cell, can't place another across word here
        if self.cell_map.get(&location).unwrap().get_across_word_id().is_some() {
            false
        } else {
            let across_relative_moves: Vec<(isize, isize)> = vec![(0, -1), (0, 1)];
            self.neighbouring_cells_empty(location, across_relative_moves)
        }
    }

    fn cell_is_open_down(&self, location: Location) -> bool {
        // If there is already an down word for this cell, can't place another down word here
        if self.cell_map.get(&location).unwrap().get_down_word_id().is_some() {
            false
        } else {
            let down_relative_moves: Vec<(isize, isize)> = vec![(-1, 0), (1, 0)];
            self.neighbouring_cells_empty(location, down_relative_moves)
        }
    }

    fn cell_is_open(&self, location: Location, across: bool) -> bool {
        if across {
            self.cell_is_open_across(location)
        } else {
            self.cell_is_open_down(location)
        }
    }

    pub fn try_place_word_in_cell(&mut self,
                              location: Location,
                              word_id: usize,
                              index_in_word: usize,
                              across: bool) -> bool {
        debug!("Trying to add word");
        self.fit_to_size();
        self.fill_black_cells();

        let mut success = true;
        let mut start_location = location;
        let word = self.word_map.get(&word_id).unwrap().clone();
        if self.cell_is_open(location, across) {
            let cells_before_this = - (index_in_word as isize);
            let cells_after_this = (word.word_text.len() as isize) - (index_in_word as isize);
            start_location = location.relative_location_directed(cells_before_this, across);
            let end_location: Location = location.relative_location_directed(cells_after_this, across);
            self.expand_to_fit_cell(start_location);
            self.expand_to_fit_cell(end_location);

            let mut updated_locations: Vec<Location> = vec![];

            let mut working_location = start_location.clone();
            for letter in word.word_text.chars() {
                if success {
                    debug!("Trying to add letter {} to cell location {:?}", letter, working_location);
                    let cell = self.cell_map.get_mut(&working_location).unwrap();
                    success = cell.add_word(word_id, letter, across);
                    updated_locations.push(working_location);
                    working_location = working_location.relative_location_directed(1, across);
                }
            }

            if !success {
                for updated_location in updated_locations {
                    let cell = self.cell_map.get_mut(&updated_location).unwrap();
                    cell.remove_word(word_id);
                }
            }
        }
        if success {
            self.word_map.insert(word_id, Word::new(&word.word_text, start_location, across));
        }
        self.fit_to_size();
        self.fill_black_cells();
        success
    }

    fn remove_word(&mut self, word_id: usize) {
        self.word_map.remove(&word_id);
        for (_location, cell) in self.cell_map.iter_mut() {
            cell.remove_word(word_id);
        }
        if let Some(word) = self.word_map.get_mut(&word_id) {
            word.remove_placement();
        }
    }
}
