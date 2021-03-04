use std::collections::{HashMap,HashSet};
use std::fs;
use std::cmp;
use log::{info,debug};

use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::grid::CrosswordGrid;
use crate::grid::Direction;

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
        let num_intersections: f64 = grid.count_intersections() as f64;
        let double_counted_filled: f64 = num_placed + num_intersections;
        let proportion_intersections: f64 = (num_intersections * 2.0) / double_counted_filled;

        let float_score: f64 = num_placed * 100.0 + proportion_filled * 20.0 + proportion_intersections * 100.0 - (nonsquare_penalty as f64) * 5.0 + 1000.0 * num_cycles;
        let score = (float_score * 100.0) as isize;
        info!("Score: {}, Num cycles: {}, Placed: {}, Prop filed: {}\n{}",
              score, num_cycles, num_placed, proportion_filled, grid.to_string());
        score
    }
}

#[derive(Debug)]
pub struct CrosswordGeneratorSettings {
    seed: u64,
    moves_between_scores: usize,
    num_children: usize,
    num_per_generation: usize,
    max_rounds: usize,
    move_types: Vec<MoveType>,
}

impl CrosswordGeneratorSettings {
    pub fn default() -> Self {
        CrosswordGeneratorSettings {
            moves_between_scores: 4,
            num_per_generation: 20,
            num_children: 10,
            max_rounds: 20,
            seed: 13,
            move_types: generate_move_types_vec(10, 1),
        }
    }

    pub fn new(seed: u64,
           moves_between_scores: usize,
           num_children: usize,
           num_per_generation: usize,
           max_rounds: usize) -> Self {
        CrosswordGeneratorSettings {
            moves_between_scores,
            num_per_generation,
            num_children,
            max_rounds,
            seed,
            move_types: generate_move_types_vec(10, 1),
        }
    }
}

#[derive(Debug)]
pub struct CrosswordGenerator {
    current_generation: Vec<CrosswordGridAttempt>,
    next_generation: Vec<CrosswordGridAttempt>,
    round: usize,
    pub settings: CrosswordGeneratorSettings,
}

impl CrosswordGenerator {
    pub fn new_from_file(filename: &str) -> Self {
        let contents = fs::read_to_string(filename).unwrap();
        let words: Vec<&str> = contents.split('\n').collect();
        CrosswordGenerator::new_from_singletons(words)
    }

    pub fn new_from_singletons(words: Vec<&str>) -> Self {
        let mut singletons: Vec<CrosswordGridAttempt> = vec![];

        for grid in CrosswordGrid::random_singleton_grids(words) {
            singletons.push(CrosswordGridAttempt::new(grid));
        }

        info!("First of first generation is {}", singletons[0].grid.to_string());

        CrosswordGenerator {
            current_generation: singletons,
            next_generation: vec![],
            round: 0,
            settings: CrosswordGeneratorSettings::default(),
        }
    }

    fn choose_random_move_type(&self, seed: u64) -> MoveType {
        let mut rng = StdRng::seed_from_u64(self.settings.seed + seed);
        *self.settings.move_types.choose(&mut rng).unwrap()
    }

    fn produce_child(&self, grid_attempt: &CrosswordGridAttempt, seed: u64) -> CrosswordGridAttempt {
        let mut copied = grid_attempt.grid.clone();
        let mut moves = 0;
        let mut success = true;
        while success && moves < self.settings.moves_between_scores {
            let extended_seed: u64 = seed + moves as u64;
            let random_move = self.choose_random_move_type(extended_seed);
            debug!("Picked move {:?}", random_move);
            match random_move {
                MoveType::PlaceWord => {
                    success = copied.place_random_word(extended_seed);
                },
                MoveType::PruneLeaves => {
                    copied.remove_random_leaves(1, extended_seed);
                },
            }
            moves += 1;
        }
        CrosswordGridAttempt::new(copied)
    }

    fn next_generation(&mut self) {
        for grid_attempt in self.current_generation.iter() {
            debug!("Considering extensions of grid:\n{}", grid_attempt.grid.to_string());
            let seed = grid_attempt.score as u64;
            for child_index in 0..self.settings.num_children {
                let child = self.produce_child(&grid_attempt, seed + child_index as u64);
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

        for grid_attempt in unique_children.drain(..).take(self.settings.num_per_generation) {
            debug!("Grid has score {}:\n{}", grid_attempt.score, grid_attempt.grid.to_string());
            self.current_generation.push(grid_attempt);
        }
    }

    fn output_best(&self, num_to_output: usize) -> Vec<CrosswordGrid> {
        let mut output: Vec<CrosswordGrid> = vec![];
        for grid_attempt in self.current_generation.iter().take(num_to_output) {
            output.push(grid_attempt.grid.clone());
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
        let mut best_score_ever: isize = 0;
        let mut best_score: isize = self.get_current_best_score();
        info!("Round {}. Current best score is {:?}", self.round, best_score);

        while best_score > best_score_ever && self.round < self.settings.max_rounds {
            best_score_ever = best_score;

            self.next_generation();
            best_score = self.get_current_best_score();
            info!("Round {}. Current best score is {:?}", self.round, best_score);

            self.round += 1;
        }
        if best_score <= best_score_ever {
            info!("Stopped iterating since we stopped increasing our score");
        }

        self.output_best(self.settings.num_per_generation)
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
