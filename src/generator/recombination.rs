use log::info;

use rand::SeedableRng;
use rand::seq::IteratorRandom;
use rand::rngs::StdRng;
use rand::Rng;

use super::CrosswordGridAttempt;
use super::CrosswordGenerator;
use super::MoveType;

impl CrosswordGenerator {
    fn generate_partitions(&self,
                           partitions_per_parent: usize,
                           seed: u64) -> Vec<CrosswordGridAttempt> {
        let mut partitions: Vec<CrosswordGridAttempt> = vec![];
        for parent in self.current_generation_ancestors.iter() {
            let parent_seed: u64 = seed.wrapping_add(parent.summary_score as u64);
            for i in 0..partitions_per_parent {
                let extended_seed: u64 = parent_seed.wrapping_add(i as u64);
                let mut copied = parent.clone();
                if let Some(other_half) = self.attempt_partition(&mut copied,
                                                                 extended_seed) {
                    partitions.push(other_half);
                    partitions.push(copied);
                }
            }
        }
        self.pick_best_varied(partitions, self.settings.num_per_generation * 2)
    }

    pub fn perform_recombination(&mut self, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);

        let gametes = self.generate_partitions(10, seed);
        let mut first_index = 0;

        let mut recombined: Vec<CrosswordGridAttempt> = vec![];
        while first_index < gametes.len() {
            let mut second_index = 0;
            let mut success = false;
            let mut min_overlaps = 1;
            while second_index < first_index {
                let mut first = gametes[first_index].clone();
                let second = &gametes[second_index];
                let success = first.grid.try_merge_with_grid(&second.grid, min_overlaps);
                if success {
                    info!("Successful recombination with at least {} overlaps \n{}\n{}",
                          min_overlaps,
                          second.grid.to_string(),
                          first.grid.to_string());
                    first.increment_move_count(MoveType::Recombination);
                    recombined.push(first);
                    min_overlaps += 1;
                }
                second_index += 1;
            }
            if min_overlaps == 1 {
                info!("Failed to find grid to recombine with\n{}", gametes[first_index].grid.to_string());
            }
            first_index += 1;
        }

        self.current_generation_ancestors.append(&mut recombined);
    }
}
