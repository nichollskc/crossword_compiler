use std::collections::{HashMap,HashSet};
use log::{info,warn,debug,error};

use crate::grid::CrosswordGrid;

#[derive(Debug)]
struct CrosswordGridAttempt {
    grid: CrosswordGrid,
    score: isize,
}

impl CrosswordGridAttempt {
    fn new(grid: CrosswordGrid) -> Self {
        CrosswordGridAttempt {
            score: grid.count_placed_words() as isize,
            grid,
        }
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
}

impl CrosswordGenerator {
    pub fn new_from_singletons(words: Vec<&str>) -> Self {
        let mut singletons: Vec<CrosswordGridAttempt> = vec![];
        let mut word_map: HashMap<usize, &str> = HashMap::new();
        let mut i = 0;
        for word in words.iter() {
            word_map.insert(i, word);
            i += 1;
        }

        for (word_id, word) in word_map.iter() {
            let singleton = CrosswordGrid::new_single_placed(word, *word_id, word_map.clone());
            singletons.push(CrosswordGridAttempt::new(singleton));
        }
        
        CrosswordGenerator {
            current_generation: singletons,
            next_generation: vec![],
            moves_between_scores: 5,
            num_per_generation: 20,
            num_children: 10,
            max_rounds: 20,
        }
    }

    fn next_generation(&mut self) {
        for gridAttempt in self.current_generation.iter() {
            debug!("Considering extensions of grid:\n{}", gridAttempt.grid.to_string());
            let mut children = 0;
            while children < self.num_children {
                let mut copied = gridAttempt.grid.clone();
                let mut moves = 0;
                let mut success = true;
                while success && moves < self.moves_between_scores {
                    success = copied.place_random_word();
                    moves += 1;
                }

                self.next_generation.push(CrosswordGridAttempt::new(copied));
                children += 1;
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

        unique_children.sort_by(|a, b| a.score.cmp(&b.score));

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
