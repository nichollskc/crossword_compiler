use std::collections::HashMap;

fn mean(values: &Vec<f64>) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

use super::CrosswordGridScore;

impl CrosswordGridScore {
    fn average_scores(scores: Vec<Self>) -> Self {
        let hashmaps: Vec<HashMap<String, f64>> = scores.iter().map(to_hashmap).collect();
        let mut means: HashMap<String, f64> = HashMap::new();

        for key in hashmaps[0].keys() {
            let all_values: Vec<f64> = hashmaps.iter().map(|h| *h.get(key).unwrap()).collect();
            means.insert(key.to_string(), mean(&all_values));
        }
        let v = serde_json::to_value(means).unwrap();
        serde_json::from_value(v).unwrap()
    }

    fn to_hashmap(&self) -> HashMap<String, f64> {
        let v = serde_json::to_value(self).unwrap();
        println!("{}", v);
        serde_json::from_value(v).unwrap()
    }
}
