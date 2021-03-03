use std::collections::{HashMap,HashSet};
use std::fs;
use std::cmp;
use log::{info,warn,debug,error};

use rand::seq::SliceRandom;
use rand::prelude::*;
use rand::{Rng,SeedableRng};
use rand::rngs::StdRng;

use crate::grid::CrosswordGrid;

#[derive(Clone,Copy,Debug)]
enum MoveType {
    PlaceWord,
    PruneLeaves,
}

fn generate_move_types_vec(place_word_weight: usize, prune_leaves_weight: usize) -> Vec<MoveType> {
    let mut move_types = vec![];
    for _ in 0..place_word_weight {
        move_types.push(MoveType::PlaceWord);
    }
    for _ in 0..prune_leaves_weight {
        move_types.push(MoveType::PruneLeaves);
    }

    move_types
}

#[derive(Debug)]
struct CrosswordGridAttempt {
    grid: CrosswordGrid,
    score: isize,
}

impl CrosswordGridAttempt {
    fn new(grid: CrosswordGrid) -> Self {
        CrosswordGridAttempt {
            score: CrosswordGridAttempt::score_grid(&grid),
            grid,
        }
    }

    fn score_grid(grid: &CrosswordGrid) -> isize {
        let (nrows, ncols) = grid.get_grid_dimensions();
        let total_cells = nrows * ncols;
        let nonsquare_penalty: usize = cmp::max(nrows, ncols).pow(2) - total_cells;
        let proportion_filled: f64 = (grid.count_filled_cells() as f64) / (total_cells as f64);
        let num_placed: f64 = grid.count_placed_words() as f64;
        let num_cycles: f64 = grid.to_graph().count_cycles() as f64;
        info!("Num cycles: {}", num_cycles);

        let float_score: f64 = num_placed * proportion_filled * 10.0 + (nonsquare_penalty as f64) + 200.0 * num_cycles;
        (float_score * 100.0) as isize
    }
}

#[derive(Debug)]
pub struct CrosswordGenerator {
    current_generation: Vec<CrosswordGridAttempt>,
    next_generation: Vec<CrosswordGridAttempt>,
    moves_between_scores: usize,
    num_children: usize,
    num_per_generation: usize,
    max_rounds: usize,
    seed: u64,
    move_types: Vec<MoveType>,
}

impl CrosswordGenerator {
    pub fn new_from_file(filename: &str) -> Self {
        let contents = fs::read_to_string(filename).unwrap();
        let words: Vec<&str> = contents.split('\n').collect();
        CrosswordGenerator::new_from_singletons(words)
    }

    pub fn new_from_singletons(words: Vec<&str>) -> Self {
        let mut singletons: Vec<CrosswordGridAttempt> = vec![];
        let mut word_map: HashMap<usize, &str> = HashMap::new();
        for (word_id, word) in words.iter().enumerate() {
            word_map.insert(word_id, word);
        }

        for (word_id, word) in words.iter().enumerate() {
            let singleton = CrosswordGrid::new_single_placed(word, word_id, word_map.clone());
            singletons.push(CrosswordGridAttempt::new(singleton));
        }

        info!("First of first generation is {}", singletons[0].grid.to_string());

        CrosswordGenerator {
            current_generation: singletons,
            next_generation: vec![],
            moves_between_scores: 5,
            num_per_generation: 20,
            num_children: 10,
            max_rounds: 3,
            seed: 13,
            move_types: generate_move_types_vec(5, 2),
        }
    }

    fn choose_random_move_type(&self, seed: u64) -> MoveType {
        let mut rng = StdRng::seed_from_u64(self.seed + seed);
        *self.move_types.choose(&mut rng).unwrap()
    }

    fn produce_child(&self, gridAttempt: &CrosswordGridAttempt, seed: u64) -> CrosswordGridAttempt {
        let mut copied = gridAttempt.grid.clone();
        let mut moves = 0;
        let mut success = true;
        while success && moves < self.moves_between_scores {
            let extended_seed: u64 = seed + moves as u64;
            match self.choose_random_move_type(extended_seed) {
                MoveType::PlaceWord => {
                    success = copied.place_random_word(extended_seed);
                },
                MoveType::PruneLeaves => {
                    copied.remove_random_leaves(2, extended_seed);
                },
            }
            moves += 1;
        }
        CrosswordGridAttempt::new(copied)
    }

    fn next_generation(&mut self) {
        for gridAttempt in self.current_generation.iter() {
            debug!("Considering extensions of grid:\n{}", gridAttempt.grid.to_string());
            let seed = (gridAttempt.score as u64);
            for child_index in 0..self.num_children {
                let child = self.produce_child(&gridAttempt, seed + child_index as u64);
                self.next_generation.push(child);
            }
        }

        // Clear current generation, but add them to the next generation in case they
        // actually score better
        self.next_generation.append(&mut self.current_generation);

        let mut unique_children_hashes: HashSet<String> = HashSet::new();
        let mut unique_children: Vec<CrosswordGridAttempt> = vec![];

        for child in self.next_generation.drain(..) {
            let is_new_child = unique_children_hashes.insert(child.grid.to_string());
            if is_new_child {
                unique_children.push(child);
            }
        }

        unique_children.sort_by(|a, b| b.score.cmp(&a.score));

        for gridAttempt in unique_children.drain(..).take(self.num_per_generation) {
            info!("Grid has score {}:\n{}", gridAttempt.score, gridAttempt.grid.to_string());
            self.current_generation.push(gridAttempt);
        }
    }

    fn output_best(&self, num_to_output: usize) -> Vec<CrosswordGrid> {
        let mut output: Vec<CrosswordGrid> = vec![];
        for gridAttempt in self.current_generation.iter().take(num_to_output) {
            output.push(gridAttempt.grid.clone());
        }
        output
    }

    fn get_current_best_score(&self) -> isize {
        match self.current_generation.iter().map(|x| x.score).max() {
            Some(max_score) => max_score,
            None => 0,
        }
    }

    pub fn generate(&mut self) -> Vec<CrosswordGrid> {
        let mut round: usize = 0;
        let mut best_score_ever: isize = 0;
        let mut best_score: isize = self.get_current_best_score();
        info!("Round {}. Current best score is {:?}", round, best_score);

        while best_score > best_score_ever && round < self.max_rounds {
            best_score_ever = best_score;

            self.next_generation();
            best_score = self.get_current_best_score();
            info!("Round {}. Current best score is {:?}", round, best_score);

            round += 1;
        }
        if best_score <= best_score_ever {
            info!("Stopped iterating since we stopped increasing our score");
        }

        self.output_best(self.num_per_generation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_generation() {
        crate::logging::init_logger(true);
        let generator = CrosswordGenerator::new_from_singletons(vec!["APPLE", "PEAR", "BANANA"]);
        debug!("{:#?}", generator);
    }

    #[test]
    fn test_next_generation() {
        crate::logging::init_logger(true);
        let words = vec!["BEARER", "ABOVE", "HERE", "INVALUABLE", "BANANA", "ROYAL", "AROUND", "ROE"];
        let mut generator = CrosswordGenerator::new_from_singletons(words);
        generator.next_generation();
        generator.next_generation();
        generator.next_generation();
    }
}
