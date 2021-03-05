use serde::{Deserialize,Serialize};
use std::collections::{HashMap,HashSet};
use std::{cmp,fs,fmt};
use log::{info,debug};

use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::grid::CrosswordGrid;

mod stats;

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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct CrosswordGridScore {
    total_cells: f64,
    non_square_penalty: f64,
    proportion_filled: f64,
    proportion_intersections: f64,
    words_placed: f64,
    words_unplaced: f64,
    filled_cells: f64,
    num_cycles: f64,
    num_intersections: f64,
    summary: f64,
}

impl CrosswordGridScore {
    fn new(grid: &CrosswordGrid) -> Self {
        let (nrows, ncols) = grid.get_grid_dimensions();
        let total_cells = nrows * ncols;
        let non_square_penalty: usize = cmp::max(nrows, ncols).pow(2) - total_cells;
        let filled_cells: f64 = (grid.count_filled_cells() as f64);
        let proportion_filled: f64 = filled_cells / (total_cells as f64);
        let words_placed: f64 = grid.count_placed_words() as f64;
        let words_unplaced: f64 = grid.count_unplaced_words() as f64;
        let num_cycles: f64 = grid.to_graph().count_cycles() as f64;
        let num_intersections: f64 = grid.count_intersections() as f64;
        let double_counted_filled: f64 = filled_cells + num_intersections;
        let proportion_intersections: f64 = (num_intersections * 2.0) / double_counted_filled;

        let float_score: f64 = words_placed * 100.0 + proportion_filled * 20.0 + proportion_intersections * 100.0 - (non_square_penalty as f64) * 5.0 + 1000.0 * num_cycles;
        let summary = float_score * 100.0;
        CrosswordGridScore {
            total_cells: total_cells as f64,
            non_square_penalty: non_square_penalty as f64,
            proportion_filled,
            proportion_intersections,
            words_placed,
            words_unplaced,
            filled_cells,
            num_cycles,
            num_intersections,
            summary,
        }
    }
}

impl fmt::Display for CrosswordGridScore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GridScore[ summary:: {:.3} total_cells:: {:.0} filled_cells:: {:.0} \
               non_square_penalty:: {:.0} proportion_filled:: {:.3} proportion_intersections:: {:.3} \
               words_placed:: {:.0} words_unplaced:: {:.0} num_cycles:: {:.0} num_intersections:: {:.0}]",
               self.summary, self.total_cells, self.filled_cells,
               self.non_square_penalty, self.proportion_filled, self.proportion_intersections,
               self.words_placed, self.words_unplaced, self.num_cycles, self.num_intersections)
    }
}

#[derive(Debug)]
struct CrosswordGridAttempt {
    grid: CrosswordGrid,
    score: CrosswordGridScore,
    summary_score: isize,
}

impl CrosswordGridAttempt {
    fn new(grid: CrosswordGrid) -> Self {
        let score = CrosswordGridAttempt::score_grid(&grid);
        CrosswordGridAttempt {
            summary_score: score.summary as isize,
            score,
            grid,
        }
    }

    fn score_grid(grid: &CrosswordGrid) -> CrosswordGridScore {
        CrosswordGridScore::new(grid)
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
        CrosswordGeneratorSettings::new_from_hashmap(HashMap::new())
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

    pub fn new_from_hashmap(settings: HashMap<&str, usize>) -> Self {
        CrosswordGeneratorSettings::new(*settings.get("seed").unwrap_or(&13) as u64,
                                        *settings.get("moves-between-scores").unwrap_or(&4),
                                        *settings.get("num-children").unwrap_or(&10),
                                        *settings.get("num-per-gen").unwrap_or(&20),
                                        *settings.get("max-rounds").unwrap_or(&20))
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
    pub fn new_from_file_default(filename: &str) -> Self {
        CrosswordGenerator::new_from_file(filename, HashMap::new())
    }

    pub fn new_from_file(filename: &str, settings_map: HashMap<&str, usize>) -> Self {
        let contents = fs::read_to_string(filename).unwrap();
        let words: Vec<&str> = contents.split('\n').collect();
        CrosswordGenerator::new_from_singletons(words, settings_map)
    }

    pub fn new_from_singletons(words: Vec<&str>, settings_map: HashMap<&str, usize>) -> Self {
        let settings = CrosswordGeneratorSettings::new_from_hashmap(settings_map);

        let mut singletons: Vec<CrosswordGridAttempt> = vec![];

        for grid in CrosswordGrid::random_singleton_grids(words, settings.seed) {
            singletons.push(CrosswordGridAttempt::new(grid));
        }

        info!("First of first generation is {}", singletons[0].grid.to_string());

        CrosswordGenerator {
            current_generation: singletons,
            next_generation: vec![],
            round: 0,
            settings,
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
            let seed = grid_attempt.summary_score as u64;
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

        unique_children.sort_by(|a, b| b.summary_score.cmp(&a.summary_score));

        for grid_attempt in unique_children.drain(..).take(self.settings.num_per_generation) {
            debug!("Grid has score {}\n{}", grid_attempt.score, grid_attempt.grid.to_string());
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
        self.current_generation.iter().map(|x| x.summary_score).max().unwrap_or(0)
    }

    fn get_average_scores(&self) -> CrosswordGridScore {
        CrosswordGridScore::average_scores(self.current_generation.iter().map(|x| x.score).collect())
    }

    pub fn generate(&mut self) -> Vec<CrosswordGrid> {
        let mut best_score_ever: isize = 0;
        let mut best_score: isize = self.get_current_best_score();
        info!("Round {}. Average score is {}", self.round, self.get_average_scores());
        info!("Round {}. Current best score is {:?}", self.round, best_score);

        while best_score > best_score_ever && self.round < self.settings.max_rounds {
            best_score_ever = best_score;

            self.next_generation();
            best_score = self.get_current_best_score();
            info!("Round {}. Average score is {}", self.round, self.get_average_scores());
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
        let generator = CrosswordGenerator::new_from_singletons(vec!["APPLE", "PEAR", "BANANA"], HashMap::new());
        debug!("{:#?}", generator);
    }

    #[test]
    fn test_next_generation() {
        crate::logging::init_logger(true);
        let words = vec!["BEARER", "ABOVE", "HERE", "INVALUABLE", "BANANA", "ROYAL", "AROUND", "ROE"];
        let mut generator = CrosswordGenerator::new_from_singletons(words, HashMap::new());
        generator.next_generation();
        generator.next_generation();
        generator.next_generation();
    }
}
