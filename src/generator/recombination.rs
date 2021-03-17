use super::CrosswordGridAttempt;
use super::CrosswordGenerator;

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
        let gametes = self.generate_partitions(5, seed);
        let mut recombined: Vec<CrosswordGridAttempt> = vec![];
        for i in 0..gametes.len() {
            for j in 0..gametes.len() {
                let mut first = gametes[i].clone();
                let second = &gametes[j];
                let success = first.grid.try_merge_with_grid(&second.grid);
                if success {
                    println!("Successful recombination with {}\n{}",
                             second.grid.to_string(),
                             first.grid.to_string());
                    recombined.push(first);
                }
            }
        }

//        let gamete_strings: Vec<String> = gametes.iter().map(|x| x.grid.to_string()).collect();
//        println!("Gametes\n{}", gamete_strings.join("\n\n"));
    }
}
