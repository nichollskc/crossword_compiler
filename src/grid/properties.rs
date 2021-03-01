use crate::graph::Graph;
use std::cmp;
use log::{info,warn,debug,error};
use std::collections::HashMap;

use super::CrosswordGrid;

impl CrosswordGrid {
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
}
