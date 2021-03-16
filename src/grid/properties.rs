use log::debug;

use super::CrosswordGrid;

impl CrosswordGrid {
    pub fn count_all_words(&self) -> usize {
        self.word_map.len()
    }

    pub fn count_placed_words(&self) -> usize {
        self.word_map.values().filter(|w| w.is_placed()).count()
    }

    pub fn count_unplaced_words(&self) -> usize {
        self.word_map.values().filter(|w| !w.is_placed()).count()
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

    pub fn get_grid_dimensions_with_buffer(&self) -> (usize, usize) {
        let nrows: usize = (self.bottom_right_cell_index.0 - self.top_left_cell_index.0 + 1) as usize;
        let ncols: usize = (self.bottom_right_cell_index.1 - self.top_left_cell_index.1 + 1) as usize;
        (nrows, ncols)
    }

    pub fn get_grid_dimensions(&self) -> (usize, usize) {
        let nrows: usize = (self.bottom_right_cell_index.0 - self.top_left_cell_index.0 - 1) as usize;
        let ncols: usize = (self.bottom_right_cell_index.1 - self.top_left_cell_index.1 - 1) as usize;
        (nrows, ncols)
    }

    pub fn count_filled_cells(&self) -> usize {
        self.cell_map.values().filter(|c| c.contains_letter()).count()
    }

    pub fn count_empty_cells(&self) -> usize {
        let (nrows, ncols) = self.get_grid_dimensions();
        nrows * ncols - self.count_filled_cells()
    }

    pub fn average_intersections_per_word(&self) -> f64 {
        let mut percent_intersection_per_word: Vec<f64> = vec![];
        for word in self.word_map.values() {
            if let Some((start, _end, direction)) = word.get_location() {
                let mut intersections: f64 = 0.0;
                let mut cells: f64 = 0.0;
                let mut location = start;
                debug!("{:?}", word);
                for _i in 0..word.word_text.len() {
                    let cell = self.cell_map.get(&location).unwrap();
                    debug!("{:?}", cell);
                    assert!(cell.contains_letter(), "Expected cell {:?} in word {:?} to contain letter",location, word);
                    cells += 1.0;
                    if cell.is_intersection() {
                        intersections += 1.0;
                    }
                    location = location.relative_location_directed(1, direction);
                }
                debug!("{:.0}/{:.0} = {:.2}", intersections, cells, intersections / cells);
                percent_intersection_per_word.push(intersections / cells);
            }
        }
        debug!("{:?}", percent_intersection_per_word);
        percent_intersection_per_word.iter().sum::<f64>() / (percent_intersection_per_word.len() as f64)
    }
}
