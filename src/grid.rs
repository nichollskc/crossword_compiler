use std::cmp;
use log::{info,warn,debug,error};
use std::collections::HashMap;
use std::fs;

use crate::graph::Graph;

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
    fn new(letter: char, location: Location, across_word_id: Option<usize>, down_word_id: Option<usize>) -> Self {
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
                        println!("Existing across word ID doesn't match new one {} {}", existing_across.unwrap(), across_word_id.unwrap());
                        success = false
                    }
                } else {
                    // We are updating down word id, so can happily keep the existing across word id
                    across_word_id = existing_across;

                    if existing_down.is_some() && existing_down != down_word_id {
                        // Existing ID this is a problem if the new id doesn't match the old ID
                        println!("Existing down word ID doesn't match new one {} {}", existing_down.unwrap(), down_word_id.unwrap());
                        success = false
                    }
                }

                if filled_cell.letter != letter {
                    println!("Existing letter doesn't match new one {} {}", filled_cell.letter, letter);
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

#[derive(Clone,Copy,Debug)]
struct WordPlacement {
    start_location: Location,
    end_location: Location,
    across: bool,
}

#[derive(Clone,Debug)]
struct Word {
    word_text: String,
    placement: Option<WordPlacement>,
}

impl Word {
    fn new(string: &str, start_location: Location, across: bool) -> Self {
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
        }
    }

    fn new_unplaced(string: &str) -> Self {
        Word {
            word_text: string.to_string(),
            placement: None,
        }
    }

    fn get_location(&self) -> Option<(Location, Location, bool)> {
        if let Some(word_placement) = &self.placement {
            Some((word_placement.start_location, word_placement.end_location, word_placement.across))
        } else {
            None
        }
    }

    fn remove_placement(&mut self) {
        self.placement = None;
    }

    fn extend_word(&mut self, character: char) -> Option<Location> {
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

    fn is_placed(&self) -> bool {
        self.get_location().is_some()
    }
}

#[derive(Debug)]
pub struct CrosswordGrid {
    cell_map: HashMap<Location, Cell>,
    word_map: HashMap<usize, Word>,
    top_left_cell_index: Location,
    bottom_right_cell_index: Location,
}

impl CrosswordGrid {
    pub fn new_single_word(word: &str) -> Self {
        let mut builder = CrosswordGridBuilder::new();
        builder.from_string(word)
    }

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
                    println!("Trying to add letter {} to cell location {:?}", letter, working_location);
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

    pub fn count_all_words(&self) -> usize {
        self.word_map.len()
    }

    pub fn count_placed_words(&self) -> usize {
        self.word_map.values().filter(|w| w.is_placed()).count()
    }

    pub fn count_intersections(&self) -> usize {
        let mut intersections: usize = 0;
        for cell in self.cell_map.values() {
            if cell.is_intersection() {
                intersections += 1
            }
        }
        intersections
    }

    pub fn to_graph(&self) -> Graph {
        let mut edges: Vec<(usize, usize)> = vec![];
        for cell in self.cell_map.values() {
            if cell.is_intersection() {
                edges.push((cell.get_across_word_id().unwrap(),
                            cell.get_down_word_id().unwrap()));
            }
        }
        println!("All intersections found {:#?}", edges);
        let mut graph = Graph::new_from_edges(edges);

        for (word_id, _word) in self.word_map.iter().filter(|(_id, w)| w.is_placed()) {
            graph.add_node(*word_id);
        }
        graph
    }

    fn expand_to_fit_cell(&mut self, location: Location) {
        while location.0 < self.top_left_cell_index.0 {
            self.add_empty_row(self.top_left_cell_index.0 - 1);
        }
        while location.0 > self.bottom_right_cell_index.0 {
            self.add_empty_row(self.bottom_right_cell_index.0 + 1);
        }
        while location.1 < self.top_left_cell_index.1 {
            self.add_empty_col(self.top_left_cell_index.1 - 1);
        }
        while location.1 > self.bottom_right_cell_index.1 {
            self.add_empty_col(self.bottom_right_cell_index.1 + 1);
        }
    }

    fn add_empty_row(&mut self, new_row: isize) {
        debug!("Adding new row at {}, top left is {:?}, bottom right is {:?}", new_row, self.top_left_cell_index, self.bottom_right_cell_index);
        let mut col = self.top_left_cell_index.1;
        while col <= self.bottom_right_cell_index.1 {
            let location = Location(new_row, col);
            self.cell_map.insert(location, Cell::empty(location));
            col += 1;
        }
        if new_row > self.bottom_right_cell_index.0 {
            self.bottom_right_cell_index = Location(new_row, self.bottom_right_cell_index.1);
        } else if new_row < self.top_left_cell_index.0 {
            self.top_left_cell_index = Location(new_row, self.top_left_cell_index.1);
        }
    }

    fn add_empty_col(&mut self, new_col: isize) {
        debug!("Adding new col at {}", new_col);
        let mut row = self.top_left_cell_index.0;
        while row <= self.bottom_right_cell_index.0 {
            let location = Location(row, new_col);
            self.cell_map.insert(location, Cell::empty(location));
            row += 1;
        }
        if new_col > self.bottom_right_cell_index.1 {
            self.bottom_right_cell_index = Location(self.bottom_right_cell_index.0, new_col);
        } else if new_col < self.top_left_cell_index.1 {
            self.top_left_cell_index = Location(self.top_left_cell_index.0, new_col);
        }
    }

    fn ensure_buffer_exists(&mut self) {
        if self.count_filled_cells_row(self.top_left_cell_index.0) > 0 {
            self.add_empty_row(self.top_left_cell_index.0 - 1);
        }
        if self.count_filled_cells_row(self.bottom_right_cell_index.0) > 0 {
            self.add_empty_row(self.bottom_right_cell_index.0 + 1);
        }
        if self.count_filled_cells_col(self.top_left_cell_index.1) > 0 {
            self.add_empty_col(self.top_left_cell_index.1 - 1);
        }
        if self.count_filled_cells_col(self.bottom_right_cell_index.1) > 0 {
            self.add_empty_col(self.bottom_right_cell_index.1 + 1);
        }
    }

    fn remove_row(&mut self, row: isize) {
        let mut col = self.top_left_cell_index.1;
        while col <= self.bottom_right_cell_index.1 {
            self.cell_map.remove(&Location(row, col));
            col += 1;
        }
        if row == self.bottom_right_cell_index.0 {
            self.bottom_right_cell_index = self.bottom_right_cell_index.relative_location(-1, 0);
        } else if row == self.top_left_cell_index.0 {
            self.top_left_cell_index = self.top_left_cell_index.relative_location(1, 0);
        }
    }

    fn remove_col(&mut self, col: isize) {
        let mut row = self.top_left_cell_index.0;
        while row <= self.bottom_right_cell_index.0 {
            self.cell_map.remove(&Location(row, col));
            row += 1;
        }
        if col == self.bottom_right_cell_index.1 {
            self.bottom_right_cell_index = self.bottom_right_cell_index.relative_location(0, -1);
        } else if col == self.top_left_cell_index.1 {
            self.top_left_cell_index = self.top_left_cell_index.relative_location(0, 1);
        }
    }

    fn remove_excess_empty(&mut self) {
        // Remove excess rows
        while self.count_filled_cells_row(self.top_left_cell_index.0 + 1) == 0 {
            self.remove_row(self.top_left_cell_index.0)
        }
        while self.count_filled_cells_row(self.bottom_right_cell_index.0 - 1) == 0 {
            self.remove_row(self.bottom_right_cell_index.0)
        }

        // Remove excess columns
        while self.count_filled_cells_col(self.top_left_cell_index.1 + 1) == 0 {
            self.remove_col(self.top_left_cell_index.1)
        }
        while self.count_filled_cells_col(self.bottom_right_cell_index.1 - 1) == 0 {
            self.remove_col(self.bottom_right_cell_index.1)
        }
    }

    /// Trim the grid so that there is exactly one row and column of empty
    /// cells on either side of the grid
    pub fn fit_to_size(&mut self) {
        self.check_valid();

        // First make sure we've got at least one buffer row and buffer column
        self.ensure_buffer_exists();

        // Then check we don't have too many empty rows or columns
        self.remove_excess_empty();
    }

    fn count_filled_cells_row(&self, row: isize) -> usize {
        let mut col = self.top_left_cell_index.1;
        let mut filled_count: usize = 0;

        while col <= self.bottom_right_cell_index.1 {
            if self.cell_map.get(&Location(row, col)).unwrap().contains_letter() {
                filled_count += 1;
            }
            col += 1;
        }
        filled_count
    }

    fn count_filled_cells_col(&self, col: isize) -> usize {
        let mut row = self.top_left_cell_index.0;
        let mut filled_count: usize = 0;

        while row <= self.bottom_right_cell_index.0 {
            if self.cell_map.get(&Location(row, col)).unwrap().contains_letter() {
                filled_count += 1;
            }
            row += 1;
        }
        filled_count
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
        println!("{:#?}", graph);
        assert!(graph.is_connected());
    }
}

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
        println!("File contents: {}", contents);
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
                    self.cell_map.insert(location, Cell::empty(location));
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
                                                   location,
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

        let mut word_ids: Vec<usize> = vec![];
        for (word_id, word) in grid.word_map.iter() {
            if word.word_text.len() == 1 {
                word_ids.push(*word_id);
            }
        }

        for word_id in word_ids {
            grid.remove_word(word_id);
        }
        grid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_black_cells() {
        crate::logging::init_logger(true);
        let mut grid = CrosswordGrid::new_single_word("ALPHA");
        println!("{:#?}", grid);
        grid.fit_to_size();
        println!("{:#?}", grid);
        grid.fill_black_cells();

        assert_eq!(grid.cell_map.values().filter(|&x| x.is_black()).count(), 2);

        assert!(grid.cell_map.get(&Location(0, -1)).unwrap().is_black());
        assert!(grid.cell_map.get(&Location(0, 5)).unwrap().is_black());

        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
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

        let grid = CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
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

        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
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

        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/blank_space.txt");
        // Number of non-empty cells shouldn't change
        grid.fit_to_size();
        println!("Grid coords {:#?} {:#?}", grid.top_left_cell_index, grid.bottom_right_cell_index);
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
        println!("{:#?}", grid);

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

        let mut grid = CrosswordGridBuilder::new().from_file("tests/resources/simple_example.txt");
        grid.fit_to_size();
        grid.fill_black_cells();
        println!("{:#?}", grid);

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
        println!("{:#?}", grid);

        assert!(grid.try_place_word_in_cell(Location(0, 0), arrival_word_id, 0, false));
        assert!(grid.try_place_word_in_cell(Location(0, 4), bear_word_id, 2, false));
        assert!(grid.try_place_word_in_cell(Location(0, 2), cup_word_id, 2, false));

        let before_failure = grid.to_string();
        assert!(!grid.try_place_word_in_cell(Location(0, 3), bear_word_id, 1, false));
        assert_eq!(before_failure, grid.to_string());

        assert!(!grid.try_place_word_in_cell(Location(-2, 2), cap_word_id, 0, true));
        assert_eq!(before_failure, grid.to_string());
        println!("GRID IS HERE");
        println!("{}", grid.to_string());

        println!("{:#?}", grid);
        debug!("TESTING");
        assert!(grid.try_place_word_in_cell(Location(3, 0), innards_word_id, 0, true));

        let mut from_file = CrosswordGridBuilder::new().from_file("tests/resources/built_up.txt");
        from_file.fit_to_size();
        debug!("{}", grid.to_string());
        assert_eq!(from_file.to_string(), grid.to_string());
    }
}
