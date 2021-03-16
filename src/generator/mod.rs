use serde::{Deserialize,Serialize};
use std::collections::{HashMap,HashSet};
use std::{cmp,fs,fmt};
use log::{info,debug};

use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;

use ndarray::Array2;

use crate::grid::CrosswordGrid;
use crate::custom_hashmap_format;

mod stats;

#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq)]
enum MoveType {
    Partition,
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

fn calculate_similarity(adj1: &Array2<u8>, adj2: &Array2<u8>) -> f64 {
    let union = (adj1 + adj2).iter().filter(|x| **x > 0).count() as f64;
    let intersection = (adj1 * adj2).sum() as f64;
    let similarity = intersection / union;
    similarity
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
    average_intersections: f64,
    summary: f64,
}

impl CrosswordGridScore {
    fn new(grid: &CrosswordGrid, settings: &CrosswordGeneratorSettings) -> Self {
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
        let average_intersections: f64 = grid.average_intersections_per_word();

        let summary: f64 = - (non_square_penalty as f64) * (settings.weight_non_square as f64)
                + proportion_filled * (settings.weight_prop_filled as f64)
                + proportion_intersections * (settings.weight_prop_intersect as f64)
                + num_cycles * (settings.weight_num_cycles as f64)
                + num_intersections * (settings.weight_num_intersect as f64)
                + average_intersections * (settings.weight_avg_intersect as f64)
                + words_placed * (settings.weight_words_placed as f64);
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
            average_intersections,
            summary,
        }
    }
}

impl fmt::Display for CrosswordGridScore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GridScore[ summary:: {:.3} total_cells:: {:.0} filled_cells:: {:.0} \
               non_square_penalty:: {:.0} proportion_filled:: {:.3} proportion_intersections:: {:.3} average_intersections:: {:.3} \
               words_placed:: {:.0} words_unplaced:: {:.0} num_cycles:: {:.0} num_intersections:: {:.0}]",
               self.summary, self.total_cells, self.filled_cells,
               self.non_square_penalty, self.proportion_filled, self.proportion_intersections, self.average_intersections,
               self.words_placed, self.words_unplaced, self.num_cycles, self.num_intersections)
    }
}

#[derive(Clone,Debug)]
struct CrosswordGridAttempt {
    grid: CrosswordGrid,
    score: CrosswordGridScore,
    move_counts: HashMap<MoveType, f64>,
    summary_score: isize,
}

impl CrosswordGridAttempt {
    fn new(grid: CrosswordGrid, settings: &CrosswordGeneratorSettings) -> Self {
        let score = CrosswordGridAttempt::score_grid(&grid, settings);
        let mut move_counts: HashMap<MoveType, f64> = HashMap::new();
        move_counts.insert(MoveType::PlaceWord, 0.0);
        move_counts.insert(MoveType::PruneLeaves, 0.0);
        move_counts.insert(MoveType::Partition, 0.0);
        CrosswordGridAttempt {
            summary_score: score.summary as isize,
            score,
            grid,
            move_counts,
        }
    }

    fn score_grid(grid: &CrosswordGrid, settings: &CrosswordGeneratorSettings) -> CrosswordGridScore {
        CrosswordGridScore::new(grid, settings)
    }

    fn increment_move_count(&mut self, move_type: MoveType) {
        *self.move_counts.get_mut(&move_type).unwrap() += 1.0;
    }

    fn update_score(&mut self, settings: &CrosswordGeneratorSettings) {
        let score = CrosswordGridAttempt::score_grid(&self.grid, settings);
        self.score = score;
        self.summary_score = score.summary as isize;
    }
}

#[derive(Debug)]
pub struct CrosswordGeneratorSettings {
    seed: u64,
    moves_between_scores: usize,
    num_children: usize,
    num_per_generation: usize,
    max_rounds: usize,
    min_rounds: usize,
    move_types: Vec<MoveType>,
    weight_non_square: usize,
    weight_prop_filled: usize,
    weight_prop_intersect: usize,
    weight_num_cycles: usize,
    weight_num_intersect: usize,
    weight_avg_intersect: usize,
    weight_words_placed: usize,
}

impl CrosswordGeneratorSettings {
    pub fn default() -> Self {
        CrosswordGeneratorSettings::new_from_hashmap(HashMap::new())
    }

    pub fn new_from_hashmap(settings: HashMap<&str, usize>) -> Self {
        CrosswordGeneratorSettings {
            seed: *settings.get("seed").unwrap_or(&13) as u64,
            moves_between_scores: *settings.get("moves-between-scores").unwrap_or(&4),
            num_children: *settings.get("num-children").unwrap_or(&15),
            num_per_generation: *settings.get("num-per-gen").unwrap_or(&15),
            max_rounds: *settings.get("max-rounds").unwrap_or(&20),
            min_rounds: *settings.get("min-rounds").unwrap_or(&10),
            weight_non_square: *settings.get("weight-non-square").unwrap_or(&2),
            weight_prop_filled: *settings.get("weight-prop-filled").unwrap_or(&10),
            weight_prop_intersect: *settings.get("weight-prop-intersect").unwrap_or(&500),
            weight_num_cycles: *settings.get("weight-num-cycles").unwrap_or(&1000),
            weight_num_intersect: *settings.get("weight-num-intersect").unwrap_or(&100),
            weight_avg_intersect: *settings.get("weight-avg-intersect").unwrap_or(&5000),
            weight_words_placed: *settings.get("weight-words-placed").unwrap_or(&10),
            move_types: generate_move_types_vec(3, 1),
        }
    }
}

#[derive(Debug)]
pub struct CrosswordGenerator {
    current_generation_complete: Vec<CrosswordGridAttempt>,
    next_generation_complete: Vec<CrosswordGridAttempt>,
    current_generation_ancestors: Vec<CrosswordGridAttempt>,
    next_generation_ancestors: Vec<CrosswordGridAttempt>,
    round: usize,
    pub settings: CrosswordGeneratorSettings,
}

impl CrosswordGenerator {
    pub fn new_from_file_default(filename: &str) -> Self {
        CrosswordGenerator::new_from_file(filename, HashMap::new())
    }

    pub fn new_from_file_contents(contents: &str, settings_map: HashMap<&str, usize>) -> Self {
        let words: Vec<&str> = contents.split('\n').collect();
        CrosswordGenerator::new_from_singletons(words, settings_map)
    }

    pub fn new_from_file(filename: &str, settings_map: HashMap<&str, usize>) -> Self {
        let contents = fs::read_to_string(filename).unwrap();
        CrosswordGenerator::new_from_file_contents(&contents, settings_map)
    }

    pub fn new_from_singletons(words: Vec<&str>, settings_map: HashMap<&str, usize>) -> Self {
        let settings = CrosswordGeneratorSettings::new_from_hashmap(settings_map);

        let mut singletons: Vec<CrosswordGridAttempt> = vec![];

        for grid in CrosswordGrid::random_singleton_grids(words, settings.seed) {
            singletons.push(CrosswordGridAttempt::new(grid, &settings));
        }

        info!("First of first generation is {}", singletons[0].grid.to_string());

        CrosswordGenerator {
            current_generation_ancestors: singletons,
            current_generation_complete: vec![],
            next_generation_ancestors: vec![],
            next_generation_complete: vec![],
            round: 0,
            settings,
        }
    }

    fn choose_random_move_type(&self, seed: u64) -> MoveType {
        let mut rng = StdRng::seed_from_u64(self.settings.seed.wrapping_add(seed));
        *self.settings.move_types.choose(&mut rng).unwrap()
    }

    fn produce_child(&self, grid_attempt: &CrosswordGridAttempt, seed: u64) -> CrosswordGridAttempt {
        let mut copied = grid_attempt.clone();
        let mut moves = 0;
        let mut success = true;
        while success && moves < self.settings.moves_between_scores {
            let extended_seed: u64 = seed.wrapping_add(moves as u64);
            let random_move = self.choose_random_move_type(extended_seed);
            debug!("Picked move {:?}", random_move);
            match random_move {
                MoveType::PlaceWord => {
                    success = copied.grid.place_random_word(extended_seed);
                    if success {
                        copied.increment_move_count(MoveType::PlaceWord);
                    }
                },
                MoveType::PruneLeaves => {
                    copied.grid.remove_random_leaves(1, extended_seed);
                    if success {
                        copied.increment_move_count(MoveType::PruneLeaves);
                    }
                },
                MoveType::Partition => {
                    panic!("Not expecting to choose partition");
                }
            }
            moves += 1;
        }
        copied.update_score(&self.settings);
        copied
    }

    fn fill_grid(&self, grid_attempt: &CrosswordGridAttempt, seed: u64) -> CrosswordGridAttempt {
        let mut copied = grid_attempt.clone();
        let mut moves = 0;
        let mut success = true;
        while success {
            let extended_seed: u64 = seed.wrapping_add(moves as u64);
            success = copied.grid.place_random_word(extended_seed);
            if success {
                copied.increment_move_count(MoveType::PlaceWord);
            }
            moves += 1;
        }
        copied.update_score(&self.settings);
        copied
    }

    fn next_generation(&mut self) {
        info!("START. Current_ancestors: {}, current_complete: {}, next_ancestors: {}, next_complete: {}",
              self.current_generation_ancestors.len(), self.current_generation_complete.len(),
              self.next_generation_ancestors.len(), self.next_generation_complete.len());
        for grid_attempt in self.current_generation_ancestors.iter() {
            debug!("Considering extensions of grid:\n{}", grid_attempt.grid.to_string());
            let seed = (grid_attempt.summary_score as u64).wrapping_add(self.round as u64);
            for child_index in 0..self.settings.num_children {
                let child = self.produce_child(&grid_attempt, seed.wrapping_add(child_index as u64));
                self.next_generation_ancestors.push(child);
            }

            for i in 0..self.settings.num_children {
                let mut copied = grid_attempt.clone();
                if copied.grid.count_placed_words() > 1 {
                    let other_half_grid = copied.grid.random_partition(seed);
                    let mut other_half = grid_attempt.clone();
                    other_half.grid = other_half_grid;
                    debug!("Partitioned graph {}\n{}\n{}\nPartitioned graph over",
                            grid_attempt.grid.to_string(), copied.grid.to_string(), other_half.grid.to_string());
                    copied.increment_move_count(MoveType::Partition);
                    other_half.increment_move_count(MoveType::Partition);
                    copied.update_score(&self.settings);
                    other_half.update_score(&self.settings);
                    self.next_generation_ancestors.push(copied);
                    self.next_generation_ancestors.push(other_half);
                }
            }
        }
        info!("GENERATED ANCESTORS. Current_ancestors: {}, current_complete: {}, next_ancestors: {}, next_complete: {}",
              self.current_generation_ancestors.len(), self.current_generation_complete.len(),
              self.next_generation_ancestors.len(), self.next_generation_complete.len());

        // Clear current generation, but add them to the next generation in case they
        // actually score better
        self.next_generation_ancestors.append(&mut self.current_generation_ancestors);
        info!("APPENDED OLD ANCESTORS. Current_ancestors: {}, current_complete: {}, next_ancestors: {}, next_complete: {}",
              self.current_generation_ancestors.len(), self.current_generation_complete.len(),
              self.next_generation_ancestors.len(), self.next_generation_complete.len());

        let new_ancestors = self.next_generation_ancestors.drain(..).collect();
        self.current_generation_ancestors = self.pick_best_varied(new_ancestors,
                                                                  self.settings.num_per_generation);

        for grid_attempt in self.current_generation_ancestors.iter() {
            let seed = grid_attempt.summary_score as u64;
            for child_index in 0..self.settings.num_children {
                let child = self.fill_grid(&grid_attempt, seed.wrapping_add(child_index as u64));
                self.next_generation_complete.push(child);
            }
        }

        info!("MADE ANCESTORS COMPLETE. Current_ancestors: {}, current_complete: {}, next_ancestors: {}, next_complete: {}",
              self.current_generation_ancestors.len(), self.current_generation_complete.len(),
              self.next_generation_ancestors.len(), self.next_generation_complete.len());

        // Clear current generation, but add them to the next generation in case they
        // actually score better
        self.next_generation_complete.append(&mut self.current_generation_complete);
        info!("EXTENDED COMPLETE. Current_ancestors: {}, current_complete: {}, next_ancestors: {}, next_complete: {}",
              self.current_generation_ancestors.len(), self.current_generation_complete.len(),
              self.next_generation_ancestors.len(), self.next_generation_complete.len());

        let new_complete = self.next_generation_complete.drain(..).collect();
        self.current_generation_complete = self.pick_best_varied(new_complete,
                                                                 self.settings.num_per_generation);
        info!("UPDATED CURRENT COMPLETE. Current_ancestors: {}, current_complete: {}, next_ancestors: {}, next_complete: {}",
              self.current_generation_ancestors.len(), self.current_generation_complete.len(),
              self.next_generation_ancestors.len(), self.next_generation_complete.len());
    }

    fn pick_best_varied(&self, grid_attempts: Vec<CrosswordGridAttempt>, num_to_pick: usize) -> Vec<CrosswordGridAttempt> {
        let mut best_attempts: Vec<CrosswordGridAttempt> = vec![];

        let mut unique_children_hashes: HashSet<String> = HashSet::new();
        let mut unique_children: Vec<CrosswordGridAttempt> = vec![];

        for child in grid_attempts {
            let is_new_child = unique_children_hashes.insert(child.grid.to_string());
            if is_new_child {
                unique_children.push(child);
            }
        }
        let mut unique_children_summaries: Vec<isize> = unique_children.iter().map(|x| x.summary_score).collect();
        let mut unique_children_adjacencies: Vec<Array2<u8>> = unique_children.iter().map(|x| x.grid.to_graph_adjacency_matrix()).collect();
        let mut unique_children_adjusted_scores: Vec<isize> = unique_children_summaries.iter().cloned().collect();

        while best_attempts.len() < num_to_pick {
            debug!("Raw scores:\n{:?}", unique_children_summaries);
            debug!("Adjusted scores:\n{:?}", unique_children_adjusted_scores);
            let best_index: usize = unique_children_adjusted_scores.iter().enumerate().max_by_key(|(_i, &s)| s).map(|(i, _s)| i).unwrap();

            let best_attempt = unique_children.remove(best_index);
            unique_children_summaries.remove(best_index);
            unique_children_adjacencies.remove(best_index);
            unique_children_adjusted_scores.remove(best_index);

            debug!("Grid has score {}\n{}", best_attempt.score, best_attempt.grid.to_string());
            let best_adjacency = &best_attempt.grid.to_graph_adjacency_matrix();
            let existing_grid_strings: Vec<String> = best_attempts.iter().map(|x| x.grid.to_string()).collect();
            debug!("Existing grids:\n{}", existing_grid_strings.join("\n\n"));

            for i in 0..unique_children.len() {
                let adjacency = &unique_children_adjacencies[i];
                let similarity = calculate_similarity(adjacency, best_adjacency);
                let adjusted = ((unique_children_summaries[i] as f64) * (1.0 - similarity)) as isize;
                if adjusted < unique_children_adjusted_scores[i] {
                    unique_children_adjusted_scores[i] = adjusted;
                }
                debug!("This grid raw score {}, similarity {}, adjusted {}:\n{}\n{:?}",
                      unique_children_summaries[i],
                      similarity,
                      unique_children_adjusted_scores[i],
                      unique_children[i].grid.to_string(),
                      unique_children[i].score);
            }

            best_attempts.push(best_attempt);
        }
        best_attempts
    }

    fn output_best(&self, num_to_output: usize) -> Vec<CrosswordGrid> {
        let mut output: Vec<CrosswordGrid> = vec![];
        for grid_attempt in self.current_generation_complete.iter().take(num_to_output) {
            output.push(grid_attempt.grid.clone());
        }
        output
    }

    fn get_current_best_score(&self) -> isize {
        self.current_generation_complete.iter().map(|x| x.summary_score).max().unwrap_or(0)
    }

    fn get_average_scores(&self) -> CrosswordGridScore {
        if self.current_generation_complete.len() > 0 {
            CrosswordGridScore::average_scores(self.current_generation_complete.iter().map(|x| x.score).collect())
        } else {
            panic!("Called when no results!");
        }
    }

    fn stringified_output(&self) -> String {
        let mut stringified: String = String::from("");
        for grid_attempt in self.current_generation_ancestors.iter().chain(self.current_generation_complete.iter()) {
            stringified.push_str(&grid_attempt.grid.to_string());
            stringified.push_str("\n\n");
        }
        stringified
    }

    fn average_move_counts_ancestors(&self) -> HashMap<MoveType, f64> {
        let all_move_counts = self.current_generation_ancestors.iter().map(|x| x.move_counts.clone()).collect();
        stats::mean_of_hashmaps(all_move_counts)
    }

    fn average_move_counts(&self) -> HashMap<MoveType, f64> {
        let all_move_counts = self.current_generation_complete.iter().map(|x| x.move_counts.clone()).collect();
        stats::mean_of_hashmaps(all_move_counts)
    }

    pub fn generate(&mut self) -> Vec<CrosswordGrid> {
        let mut best_score: isize = self.get_current_best_score();
        let mut reached_convergence: bool = false;
        let mut last_generation_stringified = self.stringified_output();
        info!("Round {}. Current best score is {:?}", self.round, best_score);

        while !reached_convergence && self.round < self.settings.max_rounds {
            self.next_generation();
            best_score = self.get_current_best_score();
            info!("Round {}. Average score is {}", self.round, self.get_average_scores());
            info!("Round {}. Current best score is {:?}", self.round, best_score);
            info!("Round {}. Move counts of best is {}", self.round, custom_hashmap_format(&self.current_generation_complete[0].move_counts,
                                                                                           "best_count",
                                                                                           ":: "));
            info!("Round {}. Average move counts {}", self.round, custom_hashmap_format(&self.average_move_counts(),
                                                                                        "average_count",
                                                                                        ":: "));
            info!("Round {}. Average move counts {}", self.round, custom_hashmap_format(&self.average_move_counts_ancestors(),
                                                                                        "average_ancestor_count",
                                                                                        ":: "));

            let this_generation_stringified = self.stringified_output();
            info!("This generation:\n{}", this_generation_stringified);
            if self.round > self.settings.min_rounds {
                info!("Checking for convergence");
                reached_convergence = this_generation_stringified == last_generation_stringified;
            }
            last_generation_stringified = this_generation_stringified;
            self.round += 1;
        }
        if reached_convergence {
            info!("Stopped iterating since we stopped increasing our score");
        }

        info!("Best final score is: {}", self.current_generation_complete[0].score);
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
    #[ignore]
    fn test_next_generation() {
        crate::logging::init_logger(true);
        let words = vec!["BEARER", "ABOVE", "HERE", "INVALUABLE", "BANANA", "ROYAL", "AROUND", "ROE"];
        let mut generator = CrosswordGenerator::new_from_singletons(words, HashMap::new());
        generator.next_generation();
        generator.next_generation();
        generator.next_generation();
    }
}
