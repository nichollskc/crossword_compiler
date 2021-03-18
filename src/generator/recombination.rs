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
        self.restrict_to_unique(partitions)
    }

    pub fn perform_recombination(&mut self, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);

        let gametes = self.generate_partitions(5, seed);
        let mut first_indices: Vec<usize> = (0..gametes.len()).choose_multiple(&mut rng, self.settings.num_per_generation);

        let mut recombined: Vec<CrosswordGridAttempt> = vec![];
        while let Some(first_index) = first_indices.pop() {
            let attempts = 20;
            let mut i = 0;
            let mut success = false;
            while i < attempts && !success {
                let second_index = rng.gen_range(0, gametes.len());
                let mut first = gametes[first_index].clone();
                let second = &gametes[second_index];
                let result = first.grid.try_merge_with_grid(&second.grid);
				success = result.0;
				let overlaps = result.1;
                if success {
                    info!("Successful recombination with\n{}\n{}",
                             second.grid.to_string(),
                             first.grid.to_string());
                    first.increment_move_count(MoveType::Recombination);
                    recombined.push(first);
                }
                i += 1;
            }
            if !success {
                info!("Failed to find grid to recombine with\n{}", gametes[first_index].grid.to_string());
            }
        }

        self.current_generation_ancestors.append(&mut recombined);
    }
}
